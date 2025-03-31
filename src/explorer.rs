use ratatui::widgets::ListState;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::{
    fs::{self},
    io::Error,
    path::{Path, PathBuf},
};

#[derive(Default)]
pub struct FileStruct {
    pub pwd: PathBuf,
    pub parent: PathBuf,
    pub next: PathBuf,
    pub line_count: usize,
    pub current_dir: Vec<PathBuf>,
    pub current_state: ListState,
    pub parent_dir: Vec<PathBuf>,
    pub parent_state: ListState,
    pub next_dir: Vec<PathBuf>,
    pub error: Option<Error>,
    pub content: String,
    pub permission: String,
}

trait FileFun {
    fn get_dirs_and_files(path: &Path) -> Vec<PathBuf>;
    fn parent_dir_fn(&mut self);
    #[cfg(unix)]
    fn format_permissions(mode: u32) -> String;
}

impl FileFun for FileStruct {
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

    fn parent_dir_fn(&mut self) {
        let mut files: Vec<PathBuf> = vec![];
        if let Some(parent) = self.pwd.parent() {
            files = FileStruct::get_dirs_and_files(parent);
        }
        self.parent_dir = files;
    }

    #[cfg(unix)]
    fn format_permissions(mode: u32) -> String {
        let mut permissions = String::new();

        let permission_masks = [
            0o400, 0o200, 0o100, 0o040, 0o020, 0o010, 0o004, 0o002, 0o001,
        ];
        let permission_chars = ['r', 'w', 'x'];
        for i in 0..3 {
            for j in 0..3 {
                let mask = permission_masks[i * 3 + j];
                permissions.push(if mode & mask != 0 {
                    permission_chars[j]
                } else {
                    '-'
                });
            }
        }
        permissions
    }
}

impl FileStruct {
    pub fn present_dir_fn(&mut self, path: &Path, index: Option<usize>) {
        let pwd = fs::canonicalize(path).unwrap();
        self.error = None;
        self.pwd = pwd.to_path_buf();
        if let Some(parent) = self.pwd.parent() {
            self.parent = parent.to_path_buf()
        }
        let files = FileStruct::get_dirs_and_files(pwd.as_path());

        let index = match index {
            Some(idx) => idx,
            None => 0,
        };
        self.current_state.select(Some(index));

        if !files.is_empty() {
            if files[index].is_dir() {
                self.next_dir_fn(files[index].as_path());
            } else if files[index].is_file() {
                self.read_file(files[index].to_path_buf());
            } else {
                #[cfg(unix)]
                self.file_permission(files[index].as_path());
                self.next_dir.clear();
            }
        }

        self.current_dir = files;
        self.parent_dir_fn();
    }

    pub fn next_dir_fn(&mut self, path: &Path) {
        #[cfg(unix)]
        self.file_permission(path);
        let files = FileStruct::get_dirs_and_files(path);
        self.next = path.to_path_buf();
        self.next_dir = files;
    }

    pub fn read_file(&mut self, path: PathBuf) {
        #[cfg(unix)]
        self.file_permission(path.as_path());
        let line = fs::read_to_string(path).unwrap_or_else(|error| {
            self.error = Some(error);
            String::new()
        });
        self.line_count = line.lines().count();
        self.content = line;
    }

    #[cfg(unix)]
    fn file_permission(&mut self, path: &Path) {
        match fs::metadata(path) {
            Ok(metadata) => {
                let permissions = metadata.permissions();
                let mode = permissions.mode();

                let file_type = if metadata.is_dir() {
                    'd'
                } else if metadata.file_type().is_symlink() {
                    'l'
                } else {
                    '-'
                };
                let mut permission = String::new();
                permission.push(file_type);
                let f_permission = FileStruct::format_permissions(mode);
                permission.push_str(&f_permission);

                self.permission = permission;
            }
            Err(error) => {
                self.error = Some(error);
            }
        }
    }

    pub fn delete(path: &Path, file_struct: &mut FileStruct) {
        if path.is_dir() {
            match fs::remove_dir_all(path) {
                Ok(_) => {}
                Err(error) => file_struct.error = Some(error),
            }
        } else {
            match fs::remove_file(path) {
                Ok(_) => {}
                Err(error) => file_struct.error = Some(error),
            }
        }
    }
}
