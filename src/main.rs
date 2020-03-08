mod range_iter;
mod resume;
mod opt;
mod download;
mod unfinish_json;
mod task;
mod progress_show;

use crate::download::Info;
use crate::unfinish_json::{Json, UnfinishFiles};

use self::opt::Opt;
use structopt::StructOpt;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opt = Opt::from_args();
    let url = opt.url;
    let task_count = opt.job;
    let is_force = opt.is_force;
    let is_cover = opt.is_cover;
    let target_dir = opt.target_dir;
    let mut file_name = opt.rename;

    if file_name.is_empty() {
        file_name = url.split('/').rev().next().unwrap().to_string();
    }

    let length = Info::get_length(&url).await.unwrap();
    let mut target = target_dir.clone();
    target.push(&file_name);

    let mut exe_dir  = std::env::current_exe().unwrap();
    exe_dir.pop();

    let json  = Json::new(exe_dir);
    let mut is_resume = false;
    if Path::new(&json.path).exists() {
        let unfinish_files: UnfinishFiles = json.get_info();
        for mut files in unfinish_files.files {
            if files.file.target.eq(&target) && files.file.size.eq(&length) {
                is_resume = true;
                files.file.resume_from_breakpoint(&url, files.file.target.clone()).await?;
                break;
            }
        }
    }

    if !is_resume {
        let mut info = Info::new()
            .url(&url)
            .file_name(&file_name)
            .force(is_force)
            .cover(is_cover)
            .task_count(task_count)
            .length(length)
            .target_dir(target_dir.clone())
            .build();

        info.start_download().await.unwrap();
    }

    Ok(())
}