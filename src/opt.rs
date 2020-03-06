use structopt_derive::*;
use std::path::PathBuf;

#[derive(StructOpt, Debug)]
#[structopt(name = "thread_download")]
pub struct Opt {
    #[structopt(help = "Url")]
    pub url: String,
    #[structopt(help = "Output dir, default dir is current dir", short = "o", long = "output", parse(from_os_str), default_value = ".")]
    pub target_dir: PathBuf,
    #[structopt(help = "Rename", short, default_value = "")]
    pub rename: String,
    #[structopt(help = "Task number", short, default_value = "1")]
    pub job: u64,
    #[structopt(help = "Force download", short = "f", long = "focre")]
    pub is_force: bool,
}