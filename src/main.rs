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
use crate::opt::Command;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opt = Opt::from_args();
    let mut exe_dir = std::env::current_exe().unwrap();
    exe_dir.pop();
    let json = Json::new(exe_dir);

    match opt.cmd {
        Command::Download {
            url, target_dir, rename, job, is_force, is_cover,
        } => {
            let url = url;
            let task_count = job;
            let is_force = is_force;
            let is_cover = is_cover;
            let target_dir = target_dir;
            let mut file_name = rename;

            if file_name.is_empty() {
                file_name = url.split('/').rev().next().unwrap().to_string();
            }

            let length = Info::get_length(&url).await?;
            let mut target = target_dir.clone();
            target.push(&file_name);

            let mut is_resume = false;
            if Path::new(&json.path).exists() {
                let unfinish_files: UnfinishFiles = json.get_info();
                for mut files in unfinish_files.files {
                    if files.file.target.eq(&target) && files.file.size.eq(&length) {
                        is_resume = true;
                        files.file.resume_from_breakpoint().await?;
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

                info.start_download().await?;
            }
        }
        Command::List => {
            if json.exist() {
                let unfinish_files = json.get_info();
                println!("id    file_name      size     target");
                for (i, unfinish_file) in unfinish_files.files.iter().enumerate() {
                    println!("{} {}   {}   {:?}", i, unfinish_file.file.name, unfinish_file.file.size, unfinish_file.file.target);
                }
            } else {
                println!("no unfinish file");
            }
        }
        Command::Delete { id } => {
            if json.exist() {
                let mut unfinish_files = json.get_info();
                std::fs::remove_file(&unfinish_files.files.get(id).unwrap().file.target).unwrap();
                unfinish_files.files.remove(id);

                if unfinish_files.files.len() == 0 {
                    std::fs::remove_file(&json.path).unwrap();
                } else {
                    json.write(&unfinish_files);
                }

                println!("delete successfully");
            } else {
                println!("no unfinish file");
            }
        }
        Command::Resume { id } => {
            if json.exist() {
                let mut unfinish_files = json.get_info();
                let unfinish_file = unfinish_files.files.get_mut(id).unwrap();
                unfinish_file.file.resume_from_breakpoint().await?;
            } else {
                println!("no unfinish file");
            }
        }
    }
    Ok(())
}