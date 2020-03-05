use crate::resume::FileInfo;
use crate::thread::Thread;
use crate::unfinish_json::Json;

use std::sync::{Arc, Mutex};
use std::fs::File;
use std::sync::atomic::{AtomicBool, Ordering, AtomicUsize};
use std::str::FromStr;
use reqwest::header::{CONTENT_LENGTH, ACCEPT_RANGES};
use std::io::stdin;
use std::process::exit;
use reqwest::{redirect, Client};

pub struct Info {
    url: String,
    file_name: String,
    thread_count: u64,
    force: bool,
    length: u64,
}

impl Info {
    pub fn new(url: String,
               file_name: String,
               thread_count: u64,
               force: bool,
               length: u64) -> Info {
        Info {
            url,
            file_name,
            thread_count,
            force,
            length,
        }
    }

    pub async fn start_download(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let file = Arc::new(Mutex::new(File::create(&self.file_name)?));
        let file_info = Arc::new(Mutex::new(FileInfo {
            name: self.file_name.as_str().to_string(),
            size: self.length,
            break_point: Vec::new(),
        }));
        let finish_count = Arc::new(AtomicUsize::new(0));
        let running = Arc::new(AtomicBool::new(true));
        let msg = running.clone();

        ctrlc::set_handler(move || {
            msg.store(false, Ordering::SeqCst);
        }).expect("Error setting Ctrl-C handler");

        self.check().await.unwrap();
        println!("Downloading, size {}", self.length);

        let mut thread = Thread::new(self.url.as_str().to_string(),
                                     file.clone(),
                                     running.clone(),
                                     file_info.clone());

        let end = self.length - 1;
        let buffer_size = self.length / self.thread_count;
        let threads = thread.init(0, end, buffer_size,
                                  end / buffer_size < self.thread_count,
                                  finish_count.clone()).await;

        for thread in threads {
            thread.await?;
        }

        loop {
            if finish_count.load(Ordering::SeqCst) == self.thread_count as usize {
                let json = Json::new("unfinish.json");
                if !running.load(Ordering::SeqCst) {
                    json.save_point(file_info);
                } else {
                    println!("file download successfully");
                }
            }
            break;
        }

        Ok(())
    }

    pub async fn build_client() -> Client {
        let custom = redirect::Policy::custom(|attempt| {
            if attempt.previous().len() > 5 {
                attempt.error("too many redirects")
            } else {
                attempt.follow()
            }
        });

        let client = reqwest::Client::builder()
            .redirect(custom)
            .build().unwrap();

        client
    }

    pub async fn get_length(url: &str) -> Result<u64, Box<dyn std::error::Error>> {
        let client = Info::build_client().await;
        let info = client.get(url).send().await?;
        let length = info.headers().get(CONTENT_LENGTH).unwrap();
        let length = u64::from_str(length.to_str()?)?;
        Ok(length)
    }

    pub async fn check(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let client = Info::build_client().await;
        let info = client.get(&self.url).send().await?;
        let code = info.status();
        let code = code.as_u16();

        println!("Status {}", code);
        let accept_range = match info.headers().get(ACCEPT_RANGES) {
            Some(i) => {
                if i.to_str().unwrap().eq("none") { false } else { true }
            }
            _ => { false }
        };

        let pass = self.thread_count == 1 && self.force;
        if !accept_range && !pass || code == 206 && !pass {
            println!("the url server may not accept range or limit the range, \
        if force continue, you can start program with '-f' command");
            println!("1) force continue  2) use single thread to download");
            println!("3) exit");

            let mut input = String::new();
            stdin().read_line(&mut input).unwrap();

            match i32::from_str(input.trim()) {
                Ok(i) => {
                    if i == 1 {} else if i == 2 { self.thread_count = 1; } else { exit(0) };
                }
                Err(_) => exit(1)
            }
        }
        Ok(())
    }
}