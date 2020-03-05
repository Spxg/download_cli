use crate::resume::{FileInfo, BreakPoint};
use crate::range_iter::RangeIter;

use std::sync::{Arc, Mutex};
use std::fs::File;
use std::sync::atomic::{AtomicBool, Ordering, AtomicUsize};
use reqwest::header::RANGE;
use positioned_io_preview::WriteAt;
use tokio::task::JoinHandle;
use tokio::stream::StreamExt;

pub struct Thread {
    url: String,
    file: Arc<Mutex<File>>,
    msg: Arc<AtomicBool>,
    file_info: Arc<Mutex<FileInfo>>,
}

impl Thread {
    pub fn new(url: String,
               file: Arc<Mutex<File>>,
               msg: Arc<AtomicBool>,
               file_info: Arc<Mutex<FileInfo>>)
               -> Thread {
        Thread {
            url,
            file,
            msg,
            file_info,
        }
    }

    pub async fn init(&mut self, start: u64, end: u64, buffer_size: u64, check: bool, finish_count: Arc<AtomicUsize>) -> Vec<JoinHandle<()>> {
        let mut threads = Vec::new();

        for (range, start, end) in RangeIter::new(start, end, buffer_size, check) {
            let copy_url = self.url.as_str().to_string();
            let clone_file = self.file.clone();
            let clone_msg = self.msg.clone();
            let clone_file_info = self.file_info.clone();
            let clone_count = finish_count.clone();

            let thread = tokio::spawn(async move {
                let mut start_at = start;
                let end = end;

                let mut stream = reqwest::Client::new().get(&copy_url)
                    .header(RANGE, range)
                    .send().await.unwrap()
                    .bytes_stream();

                while let Some(item) = stream.next().await {
                    let byte = item.unwrap();
                    if clone_msg.load(Ordering::SeqCst) {
                        clone_file.lock().unwrap().write_at(start_at, &byte).unwrap();
                        start_at += byte.len() as u64;
                    } else {
                        let point = BreakPoint { start: start_at, end };
                        clone_file_info.lock().unwrap().break_point.push(point);
                        break;
                    }
                }

                clone_count.fetch_add(1, Ordering::SeqCst);
            });
            threads.push(thread);
        }

        threads
    }
}
