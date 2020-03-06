use indicatif::{ProgressBar, ProgressStyle};
use std::sync::mpsc::Receiver;
use std::thread;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

pub fn show_progress(start: u64, end: u64, rx: Receiver<u64>, start_msg: Arc<AtomicBool>) -> ProgressBar {
    let pb = ProgressBar::new(end - start);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
        .progress_chars("#>-"));
    let pb_clone = pb.clone();

    thread::spawn(move || {
        loop {
            if start_msg.load(Ordering::SeqCst) {
                pb_clone.set_position(0);
                let total_bytes = end - start;
                let mut bytes = 0;
                while total_bytes >= bytes {
                    match rx.recv() {
                        Ok(i) => {
                            let byte_len = i;
                            pb_clone.inc(byte_len);
                            bytes += byte_len;
                        },
                        Err(_) => break,
                    };
                }
                pb_clone.finish_at_current_pos();
                break;
            }
        }
    });

    pb
}