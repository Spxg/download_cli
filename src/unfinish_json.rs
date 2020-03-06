extern crate serde_json;
extern crate serde_derive;
extern crate serde;

use serde_derive::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::Read;
use positioned_io_preview::WriteAt;
use std::sync::{Arc, Mutex};
use std::borrow::BorrowMut;

#[derive(Serialize, Deserialize)]
pub struct UnfinishFiles {
    pub files: Vec<UnfinishFile>
}

#[derive(Serialize, Deserialize)]
pub struct UnfinishFile {
    pub file: FileInfo,
}

#[derive(Serialize, Deserialize)]
pub struct FileInfo {
    pub name: String,
    pub size: u64,
    pub break_point: Vec<BreakPoint>,
}

#[derive(Serialize, Deserialize)]
pub struct BreakPoint {
    pub start: u64,
    pub end: u64,
}

pub struct Json {
    pub path: PathBuf
}

impl Json {
    pub fn new(mut path: PathBuf) -> Self {
        path.push("unfinish.json");
        Json {
            path
        }
    }

    pub fn get_info(&self) -> UnfinishFiles {
        let mut info = String::new();
        let mut json = File::open(&self.path).unwrap();
        json.read_to_string(&mut info).unwrap();
        serde_json::from_str(&info).unwrap()
    }

    pub fn save_point(&self, file_info: Arc<Mutex<FileInfo>>) {
        let path = &self.path;
        let mut file_info = file_info.lock().unwrap();
        let name = file_info.name.as_str().to_string();
        let size = file_info.size;

        let mut break_point = Vec::new();
        break_point.append(file_info.break_point.borrow_mut());

        let file_info = FileInfo {
            name,
            size,
            break_point,
        };

        let unfinish_file = UnfinishFile {
            file: file_info
        };

        let mut file = vec![unfinish_file];

        if Path::new(path).exists() {
            let mut unfinish_files: UnfinishFiles = self.get_info();
            unfinish_files.files.append(&mut file);
            self.write(&unfinish_files);
        } else {
            let unfinish_files = UnfinishFiles {
                files: file
            };
            self.write(&unfinish_files);
        }

        println!("the points have been saved, next time download, the file will resume");
    }

    pub fn write(&self, unfinish_files: &UnfinishFiles) {
        let mut json = File::create(&self.path).unwrap();
        let info = serde_json::to_string_pretty(unfinish_files).unwrap();
        json.write_at(0, info.as_bytes()).unwrap();
    }

    pub fn delete_earlier(&self, name: &str) {
        let mut unfinish_files = self.get_info();
        if unfinish_files.files.len() == 1 {
            std::fs::remove_file(&self.path).unwrap();
        } else {
            let i = self.search(name);
            unfinish_files.files.remove(i.unwrap());
            self.write(&unfinish_files);
        }
    }

    fn search(&self, name: &str) -> Option<usize> {
        let unfinish_files = self.get_info();

        for (i, value) in unfinish_files.files.iter().enumerate() {
            if value.file.name.eq(name) {
                return Some(i);
            }
        }
        None
    }
}