mod range_iter;
mod opt;

use reqwest::header::{CONTENT_LENGTH, RANGE, ACCEPT_RANGES};
use std::str::FromStr;
use crate::range_iter::RangeIter;
use std::fs::File;
use std::io::{Write, Read, stdin};
use structopt::StructOpt;
use self::opt::Opt;
use std::process::exit;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opt = Opt::from_args();
    let url = opt.url;
    let thread = opt.job;
    let force = opt.force;

    let file_name = url.split('/').rev().next().unwrap();
    let mut thread = u64::from_str(thread.split('j').rev().next().unwrap())?;

    let client = reqwest::Client::new();
    let info = client.head(url.trim()).send().await?;

    let length = info.headers().get(CONTENT_LENGTH).unwrap();
    let accept_range = match info.headers().get(ACCEPT_RANGES) {
        Some(i) => {
            if i.to_str().unwrap().eq("none") { false } else { true }
        }
        _ => { false }
    };

    let length = u64::from_str(length.to_str()?)?;

    let mut v = Vec::new();
    let mut num = 0;

    if !accept_range && thread > 1 && !force {
        println!("the url server may not accept range, \
        if force, you can start program with '-f' command");
        println!("1) force continue  2) use single thread to download");
        println!("3) exit");

        let mut input = String::new();
        stdin().read_line(&mut input).unwrap();

        match i32::from_str(input.trim()) {
            Ok(i) => {
                if i == 1 {} else if i == 2 { thread = 1; } else { exit(0) };
            }
            Err(_) => exit(1)
        }
    }

    println!("Downloading, size {}", length);
    for range in RangeIter::new(0, length - 1, length / thread) {
        let copy_url = url.as_str().to_string();
        let thread = tokio::task::spawn(async move {
            println!("thread {} start at {:?}", num, range);
            let file_name = format!("temp{}", num);
            let mut file = File::create(file_name).unwrap();
            let response = reqwest::Client::new().get(&copy_url).header(RANGE, range)
                .send().await.unwrap()
                .bytes().await.unwrap();
            file.write(&response).unwrap();
            println!("thread {} download successfully", num);
        });

        v.push(thread);
        num += 1;
    }

    for i in v {
        i.await?;
    }

    println!();
    let mut file = File::create(file_name)?;
    println!("start merge");

    for i in 0..num {
        let mut buf = Vec::new();
        let path = format!("temp{}", i);
        File::open(path)?.read_to_end(&mut buf)?;
        file.write(&buf)?;
        println!("merge {} successfully", i + 1);
    }

    for i in 0..num {
        let path = format!("temp{}", i);
        std::fs::remove_file(path)?;
    }
    println!();
    println!("file download successfully");
    Ok(())
}