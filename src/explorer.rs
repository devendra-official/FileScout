use ratatui::widgets::ListState;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::{
    fs::{self},
    io::Error,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

#[derive(Default)]
pub struct FileEX {
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
    pub message: Option<String>,
}

#[derive(Default)]
pub struct FileStruct {
    pub file: Arc<Mutex<FileEX>>,
}

trait FileFun {
    fn get_dirs_and_files(path: &Path) -> Vec<PathBuf>;
    fn parent_dir_fn(file_struct: &mut FileEX);
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

    fn parent_dir_fn(file_struct: &mut FileEX) {
        let mut files: Vec<PathBuf> = vec![];
        if let Some(parent) = file_struct.pwd.parent() {
            files = FileStruct::get_dirs_and_files(parent);
        }
        file_struct.parent_dir = files;
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
        let file_struct_clone = Arc::clone(&self.file);
        let mut file_struct = file_struct_clone.lock().unwrap();

        let pwd = fs::canonicalize(path).unwrap();
        file_struct.error = None;
        file_struct.pwd = pwd.to_path_buf();
        if let Some(parent) = file_struct.pwd.parent() {
            file_struct.parent = parent.to_path_buf()
        }
        let files = FileStruct::get_dirs_and_files(pwd.as_path());

        let index = match index {
            Some(idx) => idx,
            None => 0,
        };
        file_struct.current_state.select(Some(index));

        if !files.is_empty() {
            if files[index].is_dir() {
                FileStruct::next_dir_fn(files[index].as_path(), &mut file_struct);
            } else if files[index].is_file() {
                FileStruct::read_file(files[index].to_path_buf(), &mut file_struct);
            } else {
                #[cfg(unix)]
                FileStruct::file_permission(files[index].as_path(), &mut file_struct);
                file_struct.next_dir.clear();
            }
        }

        file_struct.current_dir = files;
        FileStruct::parent_dir_fn(&mut file_struct);
    }

    pub fn next_dir_fn(path: &Path, file_struct: &mut FileEX) {
        #[cfg(unix)]
        FileStruct::file_permission(path, file_struct);
        let files = FileStruct::get_dirs_and_files(path);
        file_struct.next = path.to_path_buf();
        file_struct.next_dir = files;
    }

    pub fn read_file(path: PathBuf, file_struct: &mut FileEX) {
        #[cfg(unix)]
        FileStruct::file_permission(path.as_path(), file_struct);
        let line = fs::read_to_string(path).unwrap_or_else(|error| {
            file_struct.error = Some(error);
            String::new()
        });
        file_struct.line_count = line.lines().count();
        file_struct.content = line;
    }

    #[cfg(unix)]
    pub fn file_permission(path: &Path, file_struct: &mut FileEX) {
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

                file_struct.permission = permission;
            }
            Err(error) => {
                file_struct.error = Some(error);
            }
        }
    }

    pub fn delete(path: &Path, file_struct: &mut FileEX) {
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
