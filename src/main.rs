mod range_iter;
use reqwest::header::{CONTENT_LENGTH, RANGE};
use std::str::FromStr;
use crate::range_iter::RangeIter;
use std::fs::File;
use std::io::{Write, Read, stdin};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    eprint!("download link: ");
    let mut url = String::new();
    stdin().read_line(&mut url)?;

    eprint!("download thread: ");
    let mut thread = String::new();
    stdin().read_line(&mut thread)?;
    println!();

    let url = url.trim().to_string();
    let file_name = url.split('/').rev().next().unwrap();
    let thread = u64::from_str(thread.trim())?;

    let client = reqwest::Client::new();
    let info = client.head(url.trim()).send().await?;
    let length = info.headers().get(CONTENT_LENGTH).unwrap();
    let length = u64::from_str(length.to_str()?)?;
    let mut v = Vec::new();
    let mut num = 0;

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