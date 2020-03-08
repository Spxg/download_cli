use structopt_derive::*;
use std::path::PathBuf;

#[derive(StructOpt)]
#[structopt(name = "download_cli")]
pub struct Opt {
    #[structopt(subcommand)]
    pub cmd: Command
}

#[derive(StructOpt)]
pub enum Command {
    #[structopt(help = "Download")]
    Download {
        #[structopt(help = "Url")]
        url: String,
        #[structopt(help = "Output dir, default dir is current dir", short = "o", long = "output", parse(from_os_str), default_value = ".")]
        target_dir: PathBuf,
        #[structopt(help = "Rename", short, default_value = "")]
        rename: String,
        #[structopt(help = "Task number", short, default_value = "1")]
        job: u64,
        #[structopt(help = "Force download", short = "f", long = "focre")]
        is_force: bool,
        #[structopt(help = "Cover file", short = "c", long = "cover")]
        is_cover: bool,
    },
    #[structopt(help = "List unfinish file")]
    List,
    #[structopt(help = "Delete unfinish file")]
    Delete {
        #[structopt(help = "file id", short)]
        id: usize,
    },
    Resume {
        #[structopt(help = "file id", short)]
        id: usize,
    },
}