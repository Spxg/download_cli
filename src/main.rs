mod range_iter;
mod opt;

use self::opt::Opt;
use crate::range_iter::RangeIter;
use tokio::stream::StreamExt;
use reqwest::header::{CONTENT_LENGTH, RANGE, ACCEPT_RANGES};
use std::str::FromStr;
use std::fs::File;
use std::io::stdin;
use structopt::StructOpt;
use std::process::exit;
use positioned_io_preview::WriteAt;
use std::sync::{Arc, Mutex};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opt = Opt::from_args();
    let url = opt.url;
    let thread = opt.job;
    let force = opt.force;

    let file_name = url.split('/').rev().next().unwrap();
    let mut thread_count = u64::from_str(thread.split('j').rev().next().unwrap())?;

    let client = reqwest::Client::new();
    let info = client.head(url.trim()).send().await?;

    let length = info.headers().get(CONTENT_LENGTH).unwrap();
    let length = u64::from_str(length.to_str()?)?;

    let accept_range = match info.headers().get(ACCEPT_RANGES) {
        Some(i) => {
            if i.to_str().unwrap().eq("none") { false } else { true }
        }
        _ => { false }
    };

    if !accept_range && thread_count > 1 && !force {
        println!("the url server may not accept range, \
        if force continue, you can start program with '-f' command");
        println!("1) force continue  2) use single thread to download");
        println!("3) exit");

        let mut input = String::new();
        stdin().read_line(&mut input).unwrap();

        match i32::from_str(input.trim()) {
            Ok(i) => {
                if i == 1 {} else if i == 2 { thread_count = 1; } else { exit(0) };
            }
            Err(_) => exit(1)
        }
    }

    println!("Downloading, size {}", length);
    let mut count = 1;
    let file = Arc::new(Mutex::new(File::create(file_name)?));
    let mut threads = Vec::new();

    for (range, start) in RangeIter::new(0, length - 1, length / thread_count) {
        let copy_url = url.as_str().to_string();
        let clone_file = file.clone();

        let thread = tokio::task::spawn(async move {
            let mut start_at = start;
            println!("thread {} start at {:?}", count, range);

            let mut stream = reqwest::Client::new().get(&copy_url)
                .header(RANGE, range)
                .send().await.unwrap()
                .bytes_stream();

            while let Some(item) = stream.next().await {
                let byte = item.unwrap();
                clone_file.lock().unwrap().write_at(start_at, &byte).unwrap();
                start_at += byte.len() as u64;
            }

            println!("thread {} download successfully", count);
        });

        threads.push(thread);
        count += 1;
    }

    for thread in threads {
        thread.await?;
    }

    println!();
    println!("file download successfully");
    Ok(())
}