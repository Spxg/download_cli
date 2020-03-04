use structopt_derive::*;

#[derive(StructOpt, Debug)]
#[structopt(name = "thread_download")]
pub struct Opt {
    #[structopt(help = "Url")]
    pub url: String,
    #[structopt(help = "Thread Number", short)]
    pub job: String,
    #[structopt(help = "Force Download", short)]
    pub force: bool,
}