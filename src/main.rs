mod range_iter;
mod resume;
mod opt;
mod download;
mod unfinish_json;
mod thread;

use crate::resume::UnfinishFiles;
use crate::download::Info;
use crate::unfinish_json::Json;

use self::opt::Opt;
use std::str::FromStr;
use structopt::StructOpt;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opt = Opt::from_args();
    let url = opt.url;
    let thread = opt.job;
    let force = opt.force;

    let file_name = url.split('/').rev().next().unwrap();
    let thread_count = u64::from_str(thread.split('j').rev().next().unwrap())?;
    let length = Info::get_length(&url).await.unwrap();
    let mut info = Info::new(url.to_string(), file_name.to_string(), thread_count, force, length);
    let json = Json::new("unfinish.json");

    let mut is_resume = false;
    if Path::new(&json.path).exists() {
        let unfinish_files: UnfinishFiles = json.get_info();
        for mut files in unfinish_files.files {
            if files.file.name.eq(&file_name) && files.file.size.eq(&length) {
                is_resume = true;
                files.file.resume_download(&url).await?;
            }
        }
    }

    if !is_resume {
        info.start_download().await.unwrap();
    }

    Ok(())
}