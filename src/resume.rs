extern crate serde_json;
extern crate serde_derive;
extern crate serde;

use crate::thread::Thread;
use crate::unfinish_json::Json;

use serde_derive::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::fs::OpenOptions;
use std::sync::atomic::{Ordering, AtomicBool, AtomicUsize};


#[derive(Serialize, Deserialize)]
pub struct UnfinishFiles {
    pub files: Vec<UnfinishFile>
}

#[derive(Serialize, Deserialize)]
pub struct UnfinishFile {
    pub file: FileInfo,
}

#[derive(Serialize, Deserialize)]
pub struct FileInfo {
    pub name: String,
    pub size: u64,
    pub break_point: Vec<BreakPoint>,
}

#[derive(Serialize, Deserialize)]
pub struct BreakPoint {
    pub start: u64,
    pub end: u64,
}

impl FileInfo {
    pub async fn resume_download(&mut self, url: &String) -> Result<(), Box<dyn std::error::Error>> {
        let file = Arc::new(Mutex::new(OpenOptions::new().write(true)
            .open(&self.name).unwrap()));
        let file_info = Arc::new(Mutex::new(FileInfo {
            name: self.name.as_str().to_string(),
            size: self.size,
            break_point: Vec::new(),
        }));
        let finish_count = Arc::new(AtomicUsize::new(0));
        let running = Arc::new(AtomicBool::new(true));
        let msg = running.clone();

        println!("Continue Download...");
        let mut thread = Thread::new(url.as_str().to_string(),
                                     file.clone(),
                                     running.clone(),
                                     file_info.clone());

        ctrlc::set_handler(move || {
            msg.store(false, Ordering::SeqCst);
        }).expect("Error setting Ctrl-C handler");

        let mut threads = Vec::new();
        for point in &self.break_point {
            let buffer_size = point.end - point.start + 1;
            threads = thread.init(point.start,
                                  point.end,
                                  buffer_size,
                                  false,
                                  finish_count.clone()).await;
        }

        for thread in threads {
            thread.await?;
        }

        loop {
            if finish_count.load(Ordering::SeqCst) == self.break_point.len() {
                let json = Json::new("unfinish.json");
                json.delete_earlier(&file_info.lock().unwrap().name);
                if !running.load(Ordering::SeqCst) {
                    json.save_point(file_info.clone());
                } else {
                    println!("file download successfully");
                }
                break;
            }
        }
        Ok(())
    }
}
