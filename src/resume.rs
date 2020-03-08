use crate::task::Task;
use crate::unfinish_json::{Json, FileInfo};

use std::sync::{Arc, Mutex};
use std::fs::OpenOptions;
use std::sync::atomic::{Ordering, AtomicBool, AtomicUsize};
use std::path::PathBuf;
use indicatif::MultiProgress;

impl FileInfo {
    pub async fn resume_from_breakpoint(&mut self, url: &String, path: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        println!("Download to {:?}", path);
        let file = Arc::new(Mutex::new(OpenOptions::new().write(true)
            .open(&path).unwrap()));
        let file_info = Arc::new(Mutex::new(FileInfo {
            name: self.name.as_str().to_string(),
            size: self.size,
            target: self.target.clone(),
            break_point: Vec::new(),
        }));
        let finish_count = Arc::new(AtomicUsize::new(0));
        let ctrl_c_msg = Arc::new(AtomicBool::new(true));
        let clone_ctrl_c_msg = ctrl_c_msg.clone();
        let start_msg = Arc::new(AtomicBool::new(false));

        println!("Continue Download...");
        let mut task = Task::new(url.as_str().to_string(),
                                 file.clone(),
                                 ctrl_c_msg.clone(),
                                 start_msg.clone(),
                                 file_info.clone());

        ctrlc::set_handler(move || {
            clone_ctrl_c_msg.store(false, Ordering::SeqCst);
        }).expect("Error setting Ctrl-C handler");

        let mut tasks = Vec::new();
        let mut pbs = Vec::new();
        for point in &self.break_point {
            let (mut task, mut pb) = task.init(point.start,
                                               point.end,
                                               1,
                                               finish_count.clone()).await;
            tasks.append(&mut task);
            pbs.append(&mut pb);
        }

        let m = MultiProgress::new();
        for pb in pbs {
            m.add(pb);
        }
        m.join().unwrap();

        for task in tasks {
            task.await?;
        }

        loop {
            if finish_count.load(Ordering::SeqCst) == self.break_point.len() {
                let mut exe_dir  = std::env::current_exe().unwrap();
                exe_dir.pop();
                let json = Json::new(exe_dir);
                json.delete_earlier(file_info.lock().unwrap().target.clone());
                if !ctrl_c_msg.load(Ordering::SeqCst) {
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
