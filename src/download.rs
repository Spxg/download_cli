use crate::task::Task;
use crate::unfinish_json::{Json, FileInfo};

use std::sync::{Arc, Mutex};
use std::fs::File;
use std::sync::atomic::{AtomicBool, Ordering, AtomicUsize};
use std::str::FromStr;
use reqwest::header::{CONTENT_LENGTH, ACCEPT_RANGES};
use std::io::stdin;
use std::process::exit;
use reqwest::{redirect, Client};
use std::path::PathBuf;
use indicatif::MultiProgress;

pub struct Info {
    url: String,
    file_name: String,
    task_count: u64,
    force: bool,
    length: u64,
    target_dir: PathBuf,
    target: PathBuf,
}

pub struct InfoBuilder {
    url: String,
    file_name: String,
    task_count: u64,
    force: bool,
    length: u64,
    target_dir: PathBuf,
}

impl InfoBuilder {
    pub fn url(self, url: &str) -> InfoBuilder {
        InfoBuilder {
            url: url.to_string(),
            ..self
        }
    }

    pub fn file_name(self, name: &str) -> InfoBuilder {
        InfoBuilder {
            file_name: name.to_string(),
            ..self
        }
    }

    pub fn task_count(self, count: u64) -> InfoBuilder {
        InfoBuilder {
            task_count: count,
            ..self
        }
    }

    pub fn force(self, force: bool) -> InfoBuilder {
        InfoBuilder {
            force,
            ..self
        }
    }

    pub fn length(self, length: u64) -> InfoBuilder {
        InfoBuilder {
            length,
            ..self
        }
    }

    pub fn target_dir(self, path: PathBuf) -> InfoBuilder {
        InfoBuilder {
            target_dir: path,
            ..self
        }
    }

    pub fn build(self) -> Info {
        let mut target = self.target_dir.clone();
        target.push(&self.file_name);
        Info {
            url: self.url,
            file_name: self.file_name,
            task_count: self.task_count,
            force: self.force,
            length: self.length,
            target_dir: self.target_dir,
            target,
        }
    }
}

impl Info {
    pub fn new() -> InfoBuilder {
        InfoBuilder {
            url: String::default(),
            file_name: String::default(),
            task_count: 0,
            force: false,
            length: 0,
            target_dir: PathBuf::default(),
        }
    }

    pub async fn start_download(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.check().await.unwrap();
        let file = Arc::new(Mutex::new(File::create(&self.target)?));
        let file_info = Arc::new(Mutex::new(FileInfo {
            name: self.file_name.as_str().to_string(),
            size: self.length,
            break_point: Vec::new(),
        }));
        let finish_count = Arc::new(AtomicUsize::new(0));
        let ctrl_c_msg = Arc::new(AtomicBool::new(true));
        let clone_ctrl_c_msg = ctrl_c_msg.clone();
        let start_msg = Arc::new(AtomicBool::new(false));

        ctrlc::set_handler(move || {
            clone_ctrl_c_msg.store(false, Ordering::SeqCst);
        }).expect("Error setting Ctrl-C handler");
        println!("Downloading, size {}", self.length);

        let mut task = Task::new(self.url.as_str().to_string(),
                                 file.clone(),
                                 ctrl_c_msg.clone(),
                                 start_msg.clone(),
                                 file_info.clone());

        let end = self.length - 1;
        let (tasks, pbs) = task.init(0, end,
                              self.task_count,
                              finish_count.clone()).await;

        let m = MultiProgress::new();
        for pb in pbs {
            m.add(pb);
        }
        m.join().unwrap();

        for task in tasks {
            task.await?;
        }

        loop {
            if finish_count.load(Ordering::SeqCst) == self.task_count as usize {
                let json = Json::new(self.target_dir.clone());
                if !ctrl_c_msg.load(Ordering::SeqCst) {
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

        let pass = self.task_count == 1 || self.force;
        if !accept_range && !pass || code == 206 && !pass {
            println!("the url server may not accept range or limit the range, \
        if force continue, you can start program with '-f' command");
            println!("1) force continue  2) use single task to download");
            println!("3) exit");

            let mut input = String::new();
            stdin().read_line(&mut input).unwrap();

            match i32::from_str(input.trim()) {
                Ok(i) => {
                    if i == 1 {} else if i == 2 { self.task_count = 1; } else { exit(0) };
                }
                Err(_) => exit(1)
            }
        }

        if self.target.exists() {
            println!("the file is exists, rename the file you download or cover previous file");
            println!("1) customize name         2) rename like 'file_name(number)'");
            println!("3) cover previous file");

            let mut input = String::new();
            stdin().read_line(&mut input).unwrap();

            match i32::from_str(input.trim()) {
                Ok(i) => {
                    if i == 1 {
                        println!("input name");
                        let mut name = String::new();
                        stdin().read_line(&mut name).unwrap();
                        let mut target = self.target_dir.clone();
                        target.push(name.trim());
                        self.target = target;
                    } else if i == 2 {
                        let prev_name = self.file_name.as_str().to_string();
                        let mut i = 1;
                        let mut name = format!("{}({})", &prev_name, i);
                        let mut target = self.target_dir.clone();
                        target.push(&name);

                        while target.exists() {
                            i += 1;
                            name = format!("{}({})", prev_name, i);
                            target.pop();
                            target.push(&name);
                        }

                        self.target = target;
                    } else if i == 3 {
                        std::fs::remove_file(&self.target).unwrap();
                    } else {
                        exit(0)
                    };
                }
                Err(_) => exit(1)
            }
        }

        Ok(())
    }
}
