use std::{
    fs::{self, File},
    io::{Error, ErrorKind, Read},
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
        }

        dirs.append(&mut files);
        dirs
    }

    pub fn present_dir_fn(&mut self, path: &Path) {
        let pwd = fs::canonicalize(path).unwrap();

        self.pwd = pwd.to_path_buf();
        match self.pwd.parent() {
            Some(parent) => self.parent = parent.to_path_buf(),
            None => self.error = Some(Error::new(ErrorKind::NotFound, "oh no!")),
        }

        let files = FileStruct::get_dirs_and_files(pwd.as_path());

        if !files.is_empty() && files[0].is_dir() {
            self.current_state.select(Some(0));
            self.next_dir_fn(files[0].as_path());
        } else {
            self.next_dir.clear();
            self.read_file();
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

    fn read_file(&mut self) {
        match self.current_state.selected() {
            Some(index) => {
                let path = self.current_dir[index].to_path_buf();
                let mut file = File::open(path).unwrap();
                let mut content = String::new();
                file.read_to_string(&mut content).unwrap();
                self.content = Some(content);
            }
            None => {}
        }
    }
}
