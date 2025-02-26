use std::{
    fs::{self, File},
    io::{Error, Read},
    path::{Path, PathBuf},
};

use ratatui::widgets::ListState;

pub struct FileStruct {
    pub pwd: PathBuf,
    pub parent: PathBuf,
    pub next: PathBuf,
    pub current_dir: Vec<PathBuf>,
    pub current_state: ListState,
    pub parent_dir: Vec<PathBuf>,
    pub parent_state: ListState,
    pub next_dir: Vec<PathBuf>,
    pub error: Option<Error>,
    pub content: Option<String>,
}

impl FileStruct {
    pub fn default() -> Self {
        Self {
            pwd: PathBuf::default(),
            parent: PathBuf::default(),
            current_dir: Vec::new(),
            current_state: ListState::default(),
            parent_dir: Vec::new(),
            parent_state: ListState::default(),
            next_dir: Vec::new(),
            next: PathBuf::default(),
            error: None,
            content: None,
        }
    }

    fn get_dirs_and_files(path: &Path) -> Vec<PathBuf> {
        let mut dirs = Vec::new();
        let mut files = Vec::new();

        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.filter_map(Result::ok) {
                let entry_path = entry.path();
                if entry_path.is_dir() {
                    dirs.push(entry.path());
                } else {
                    files.push(entry.path());
                }
            }
            dirs.append(&mut files);
        }

        dirs
    }

    pub fn present_dir_fn(&mut self, path: &Path, index: Option<usize>) {
        let pwd = fs::canonicalize(path).unwrap();
        self.error = None;
        self.pwd = pwd.to_path_buf();
        if let Some(parent) = self.pwd.parent() {
            self.parent = parent.to_path_buf()
        }
        let files = FileStruct::get_dirs_and_files(pwd.as_path());

        self.current_state.select(Some(0));
        if !files.is_empty() && files[0].is_dir() {
            match index {
                Some(idx) => {
                    self.next_dir_fn(files[idx].as_path());
                }
                None => self.next_dir_fn(files[0].as_path()),
            }
        } else if !files.is_empty() && files[0].is_file() {
            self.read_file(files[0].to_path_buf());
        } else {
            self.next_dir.clear();
        }

        self.current_dir = files;
        self.parent_dir_fn();
    }

    pub fn next_dir_fn(&mut self, path: &Path) {
        let files = FileStruct::get_dirs_and_files(path);
        self.next = path.to_path_buf();
        self.next_dir = files;
    }

    fn parent_dir_fn(&mut self) {
        let mut files: Vec<PathBuf> = vec![];
        if let Some(parent) = self.pwd.parent() {
            files = FileStruct::get_dirs_and_files(parent);
        }
        self.parent_dir = files;
    }

    pub fn read_file(&mut self, path: PathBuf) {
        let mut file = match File::open(&path) {
            Ok(file) => file,
            Err(error) => {
                self.error = Some(error);
                return;
            }
        };
        let mut content = String::new();
        file.read_to_string(&mut content).unwrap_or_else(|error| {
            self.error = Some(error);
            0
        });
        self.content = Some(content);
    }

    pub fn delete(&mut self, path: &Path) {
        if path.is_dir() {
            match fs::remove_dir_all(path) {
                Ok(_) => {}
                Err(error) => self.error = Some(error),
            }
        } else {
            match fs::remove_file(path) {
                Ok(_) => {}
                Err(error) => self.error = Some(error),
            }
        }
    }
}
