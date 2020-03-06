use crate::range_iter::RangeIter;
use crate::unfinish_json::{FileInfo, BreakPoint};
use crate::progress_show::show_progress;

use std::sync::{Arc, Mutex};
use std::fs::File;
use std::sync::atomic::{AtomicBool, Ordering, AtomicUsize};
use reqwest::header::RANGE;
use positioned_io_preview::WriteAt;
use tokio::task::JoinHandle;
use tokio::stream::StreamExt;
use std::sync::mpsc;
use indicatif:: ProgressBar;

pub struct Task {
    url: String,
    file: Arc<Mutex<File>>,
    ctrl_c_msg: Arc<AtomicBool>,
    start_msg: Arc<AtomicBool>,
    file_info: Arc<Mutex<FileInfo>>,
}

impl Task {
    pub fn new(url: String,
               file: Arc<Mutex<File>>,
               ctrl_c_msg: Arc<AtomicBool>,
               start_msg: Arc<AtomicBool>,
               file_info: Arc<Mutex<FileInfo>>)
               -> Task {
        Task {
            url,
            file,
            ctrl_c_msg,
            start_msg,
            file_info,
        }
    }

    pub async fn init(&mut self, start: u64, end: u64, buffer_size: u64, check: bool, finish_count: Arc<AtomicUsize>) -> (Vec<JoinHandle<()>>, Vec<ProgressBar>) {
        let mut tasks = Vec::new();
        let mut pbs = Vec::new();

        for (range, start, end) in RangeIter::new(start, end, buffer_size, check) {
            let (tx, rx) = mpsc::channel();
            let copy_url = self.url.as_str().to_string();
            let clone_file = self.file.clone();
            let clone_ctrl_c_msg = self.ctrl_c_msg.clone();
            let clone_start_msg = self.start_msg.clone();
            let clone_file_info = self.file_info.clone();
            let clone_count = finish_count.clone();

            let pb = show_progress(start, end, rx, self.start_msg.clone());

            let task = tokio::spawn(async move {
                let mut start_at = start;
                let end = end;

                let mut stream = reqwest::Client::new().get(&copy_url)
                    .header(RANGE, range)
                    .send().await.unwrap()
                    .bytes_stream();

                while let Some(item) = stream.next().await {
                    clone_start_msg.store(true, Ordering::SeqCst);
                    let byte = item.unwrap();
                    if clone_ctrl_c_msg.load(Ordering::SeqCst) {
                        let byte_len = byte.len();
                        clone_file.lock().unwrap().write_at(start_at, &byte).unwrap();
                        start_at += byte_len as u64;
                        tx.send(byte_len as u64).unwrap();
                    } else {
                        let point = BreakPoint { start: start_at, end };
                        clone_file_info.lock().unwrap().break_point.push(point);
                        break;
                    }
                }
                clone_count.fetch_add(1, Ordering::SeqCst);
            });

            tasks.push(task);
            pbs.push(pb);
        }
        (tasks, pbs)
    }
}
