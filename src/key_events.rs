use std::{
    io::{Error, ErrorKind},
    sync::Arc,
    thread,
};

use crate::{
    constant::COLORS,
    explorer::FileStruct,
    crypto_handler::{AesEncryptor, FileCipher},
    ui::{FileScout, ViewMode},
};
use crossterm::event::{self, KeyCode, KeyEvent, KeyEventKind};

pub fn handle_events(file: &mut FileScout) {
    let file_clone = Arc::clone(&file.files.file);
    let mut file_struct = file_clone.lock().unwrap();
    if let Ok(event::Event::Key(KeyEvent { code, kind, .. })) = event::read() {
        if kind == KeyEventKind::Press {
            match code {
                KeyCode::Char('q') | KeyCode::Char('Q') => file.exit = true,
                KeyCode::Char('e') | KeyCode::Char('E') => {
                    if let Some(index) = file_struct.current_state.selected() {
                        let path = file_struct.current_dir[index].to_path_buf();
                        if path.is_file() {
                            if let Some(file_name) = path.file_name() {
                                let file_name = file_name.to_str().unwrap();
                                let file_name = format!("{}.enc", file_name);
                                let mut pwd = file_struct.pwd.to_path_buf();
                                pwd.push(file_name);
                                let message_clone = Arc::clone(&file.files.file);
                                thread::spawn(move || {
                                    {
                                        let mut msg = message_clone.lock().unwrap();
                                        msg.message = Some("Encrypting File...".to_string());
                                    }
                                    AesEncryptor::initialize().encrypt_file(&path, &pwd);
                                    {
                                        let mut msg = message_clone.lock().unwrap();
                                        msg.message =
                                            Some("File Encryption completed...".to_string());
                                    }
                                });
                            }
                        } else {
                            file_struct.error = Some(Error::new(
                                ErrorKind::IsADirectory,
                                "can't encrypt directory",
                            ))
                        }
                    }
                }
                KeyCode::Char('d') | KeyCode::Char('D') => {
                    if let Some(index) = file_struct.current_state.selected() {
                        let path = file_struct.current_dir[index].to_path_buf();
                        if path.is_file() {
                            if let Some(file_name) = path.file_name() {
                                let file_name = file_name
                                    .to_str()
                                    .unwrap()
                                    .strip_suffix(".enc")
                                    .unwrap()
                                    .to_string();
                                let mut output_path = file_struct.pwd.to_path_buf();
                                output_path.push(file_name);
                                let message_clone = Arc::clone(&file.files.file);
                                thread::spawn(move || {
                                    {
                                        let mut msg = message_clone.lock().unwrap();
                                        msg.message = Some("Decrypting File...".to_string());
                                    }
                                    AesEncryptor::initialize().decrypt_file(&path, &output_path);
                                    {
                                        let mut msg = message_clone.lock().unwrap();
                                        msg.message = Some("Completed file Decryption".to_string());
                                    }
                                });
                            }
                        } else {
                            file_struct.error =
                                Some(Error::new(ErrorKind::IsADirectory, "Not allowed!"))
                        }
                    }
                }
                KeyCode::Char('c') | KeyCode::Char('C') => {
                    file.color_index = (file.color_index + 1) % COLORS.len()
                }
                KeyCode::Delete => {
                    if let Some(index) = file_struct.current_state.selected() {
                        let path = file_struct.current_dir[index].to_path_buf();
                        FileStruct::delete(&path.as_path(), &mut file_struct);
                        let path = file_struct.pwd.to_path_buf();
                        let index = if index == 0 { 0 } else { index - 1 };
                        drop(file_struct);
                        file.files.present_dir_fn(path.as_path(), Some(index));
                    }
                }
                KeyCode::Tab => match file.mode {
                    ViewMode::ContentView => {
                        file.mode = ViewMode::ListView;
                        file.text_scroll_x = 0;
                        file.text_scroll_y = 0;
                    }
                    ViewMode::ListView => match file_struct.current_state.selected() {
                        Some(index) => {
                            if file_struct.current_dir[index].is_file() {
                                file.mode = ViewMode::ContentView
                            }
                        }
                        None => {}
                    },
                },
                KeyCode::Down => match file.mode {
                    ViewMode::ListView => {
                        file_struct.current_state.select_next();
                        file_struct.error = None;
                        if let Some(index) = file_struct.current_state.selected() {
                            if file_struct.current_dir.len() > index
                                && file_struct.current_dir[index].is_dir()
                            {
                                let path = file_struct.current_dir[index].to_path_buf();
                                FileStruct::next_dir_fn(&path.as_path(), &mut file_struct);
                            } else if file_struct.current_dir.len() > index {
                                let file_path = file_struct.current_dir[index].to_path_buf();
                                FileStruct::read_file(file_path, &mut file_struct);
                            }
                        }
                    }
                    ViewMode::ContentView => {
                        if file.text_scroll_y < file_struct.line_count.saturating_sub(1) as u16 {
                            file.text_scroll_y = file.text_scroll_y.saturating_add(1)
                        }
                    }
                },
                KeyCode::Up => match file.mode {
                    ViewMode::ListView => {
                        file_struct.current_state.select_previous();
                        file_struct.error = None;

                        if let Some(index) = file_struct.current_state.selected() {
                            if file_struct.current_dir.len() > index
                                && file_struct.current_dir[index].is_dir()
                            {
                                let path = file_struct.current_dir[index].to_path_buf();
                                FileStruct::next_dir_fn(&path.as_path(), &mut file_struct);
                            } else if file_struct.current_dir.len() > index {
                                let file_path = file_struct.current_dir[index].to_path_buf();
                                FileStruct::read_file(file_path, &mut file_struct);
                            }
                        }
                    }
                    ViewMode::ContentView => {
                        file.text_scroll_y = file.text_scroll_y.saturating_sub(1)
                    }
                },
                KeyCode::Right => match file.mode {
                    ViewMode::ListView => {
                        if let Some(index) = file_struct.current_state.selected() {
                            if file_struct.current_dir.len() > index
                                && file_struct.current_dir[index].is_dir()
                            {
                                let path = file_struct.current_dir[index].to_path_buf();
                                drop(file_struct);
                                file.files.present_dir_fn(path.as_path(), None);
                            } else if file_struct.current_dir.len() > index {
                                let file_path = file_struct.current_dir[index].to_path_buf();
                                FileStruct::read_file(file_path, &mut file_struct);
                            }
                        }
                    }
                    ViewMode::ContentView => {
                        file.text_scroll_x = file.text_scroll_x.saturating_add(1)
                    }
                },
                KeyCode::Left => match file.mode {
                    ViewMode::ListView => {
                        if let Some(index) = file_struct.parent_state.selected() {
                            let path = file_struct.parent.to_path_buf();
                            drop(file_struct);
                            file.files.present_dir_fn(&path.as_path(), Some(index));
                        }
                    }
                    ViewMode::ContentView => {
                        file.text_scroll_x = file.text_scroll_x.saturating_sub(1)
                    }
                },
                _ => {}
            }
        }
    }
}
