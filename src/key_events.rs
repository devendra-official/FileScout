use std::{
    io::{Error, ErrorKind},
    sync::{Arc, MutexGuard},
    thread,
};

use crate::{
    constant::COLORS,
    crypto_handler::AesEncryptor,
    explorer::FileStruct,
    ui::{FileScout, ViewMode},
};
use crossterm::event::{KeyCode, KeyModifiers};
use tokio::sync::mpsc::Sender;

pub fn handle_events(
    file: &mut FileScout,
    code: KeyCode,
    tx: Sender<String>,
    modifier: KeyModifiers,
) {
    let file_clone = Arc::clone(&file.files);
    let mut file_struct = file_clone.lock().unwrap();

    match file.mode {
        ViewMode::FileEdit => handle_file_edit(code, file, file_struct, modifier),
        ViewMode::Create | ViewMode::Rename => handle_file_name(code, file, file_struct),
        _ => match code {
            KeyCode::Char('q') | KeyCode::Char('Q') => file.exit = true,
            KeyCode::Char('r') | KeyCode::Char('R') => {
                if let Some(path) = &file_struct.current_path {
                    file.input.content = path.file_name().unwrap().to_str().unwrap().to_string();
                    file.mode = ViewMode::Rename
                }
            }
            KeyCode::Char('n') | KeyCode::Char('N') => file.mode = ViewMode::Create,
            KeyCode::Char('o') | KeyCode::Char('O') => {
                match file_struct.file_read() {
                    Ok(content) => {
                        file.input.total_lines = content.lines().count();
                        file.input.total_letter = content.lines().next().unwrap_or_default().len();
                        file.input.content = content;
                        file.mode = ViewMode::FileEdit;
                    }
                    Err(error) => file_struct.error = Some(error),
                };
            }
            KeyCode::Char('e') | KeyCode::Char('E') => {
                if let Some(index) = file_struct.current_state.selected() {
                    let path = file_struct.current_dir[index].to_path_buf();
                    if path.is_file() {
                        if let Some(file_name) = path.file_name() {
                            let file_name = file_name.to_str().unwrap();
                            let file_name = format!("{}.enc", file_name);
                            let mut pwd = file_struct.pwd.to_path_buf();
                            pwd.push(file_name);
                            let message_clone = Arc::clone(&file.files);
                            thread::spawn(move || {
                                match AesEncryptor::new().encrypt_file(&path, &pwd) {
                                    Ok(()) => {
                                        let mut msg = message_clone.lock().unwrap();
                                        let pwd = msg.pwd.to_path_buf();
                                        if let Some(index) = msg.current_state.selected() {
                                            msg.present_dir_fn(&pwd, Some(index));
                                        }
                                        if tx
                                            .try_send(String::from("File Encryption completed"))
                                            .is_err()
                                        {
                                            msg.error = Some(Error::new(
                                                ErrorKind::Other,
                                                "Failed to refresh",
                                            ))
                                        }
                                    }
                                    Err(error) => {
                                        let mut msg = message_clone.lock().unwrap();
                                        msg.error = Some(error);
                                        if tx
                                            .try_send(String::from("Failed to Encrypt file"))
                                            .is_err()
                                        {
                                            msg.error = Some(Error::new(
                                                ErrorKind::Other,
                                                "Failed to refresh",
                                            ))
                                        }
                                    }
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
                            let message_clone = Arc::clone(&file.files);
                            thread::spawn(move || {
                                match AesEncryptor::new().decrypt_file(&path, &output_path) {
                                    Ok(()) => {
                                        let mut msg = message_clone.lock().unwrap();
                                        let pwd = msg.pwd.to_path_buf();
                                        if let Some(index) = msg.current_state.selected() {
                                            msg.present_dir_fn(&pwd, Some(index));
                                        }
                                        if tx
                                            .try_send(String::from("File Decryption completed"))
                                            .is_err()
                                        {
                                            msg.error = Some(Error::new(
                                                ErrorKind::Other,
                                                "Failed to refresh",
                                            ))
                                        }
                                    }
                                    Err(error) => {
                                        let mut msg = message_clone.lock().unwrap();
                                        msg.error = Some(error);
                                        if tx
                                            .try_send(String::from("Failed to Decrypt file"))
                                            .is_err()
                                        {
                                            msg.error = Some(Error::new(
                                                ErrorKind::Other,
                                                "Failed to refresh",
                                            ))
                                        }
                                    }
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
                    FileStruct::delete(path.as_path(), &mut file_struct);
                    let path = file_struct.pwd.to_path_buf();
                    let index = if index == 0 { 0 } else { index - 1 };
                    file_struct.present_dir_fn(path.as_path(), Some(index));
                }
            }
            KeyCode::Tab => match file.mode {
                ViewMode::ContentView => {
                    file.mode = ViewMode::ListView;
                    file.text_scroll_x = 0;
                    file.text_scroll_y = 0;
                }
                ViewMode::ListView => {
                    if let Some(index) = file_struct.current_state.selected() {
                        if file_struct.current_dir[index].is_file() {
                            file.mode = ViewMode::ContentView
                        }
                    }
                }
                _ => {}
            },
            KeyCode::Down => match file.mode {
                ViewMode::ListView => {
                    file_struct.error = None;
                    file_struct.current_state.select_next();
                    if let Some(index) = file_struct.current_state.selected() {
                        if file_struct.current_dir.len() > index
                            && file_struct.current_dir[index].is_dir()
                        {
                            let path = file_struct.current_dir[index].to_path_buf();
                            file_struct.next_dir_fn(path.as_path());
                        } else if file_struct.current_dir.len() > index {
                            file_struct.content = String::new();
                            let file_path = file_struct.current_dir[index].to_path_buf();
                            let file = Arc::clone(&file.files);
                            tokio::spawn(async move {
                                let mut file_str = file.lock().unwrap();
                                file_str.read_file(file_path);
                                if tx.try_send(String::new()).is_err() {
                                    file_str.error =
                                        Some(Error::new(ErrorKind::Other, "something went wrong"))
                                }
                            });
                        } else {
                            return;
                        }
                        file_struct.current_path =
                            Some(file_struct.current_dir[index].to_path_buf());
                    }
                }
                ViewMode::ContentView => {
                    if file.text_scroll_y < file_struct.line_count.saturating_sub(1) {
                        file.text_scroll_y = file.text_scroll_y.saturating_add(1)
                    }
                }
                _ => {}
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
                            file_struct.next_dir_fn(path.as_path());
                        } else if file_struct.current_dir.len() > index {
                            file_struct.content = String::new();
                            let file_path = file_struct.current_dir[index].to_path_buf();
                            let file = Arc::clone(&file.files);
                            tokio::spawn(async move {
                                let mut file_str = file.lock().unwrap();
                                file_str.read_file(file_path);
                                if tx.try_send(String::new()).is_err() {
                                    file_str.error =
                                        Some(Error::new(ErrorKind::Other, "something went wrong"))
                                }
                            });
                        }
                        file_struct.current_path =
                            Some(file_struct.current_dir[index].to_path_buf());
                    }
                }
                ViewMode::ContentView => file.text_scroll_y = file.text_scroll_y.saturating_sub(1),
                _ => {}
            },
            KeyCode::Right => match file.mode {
                ViewMode::ListView => {
                    if let Some(index) = file_struct.current_state.selected() {
                        if file_struct.current_dir.len() > index
                            && file_struct.current_dir[index].is_dir()
                        {
                            let path = file_struct.current_dir[index].to_path_buf();
                            file_struct.present_dir_fn(path.as_path(), None);
                        }
                    }
                }
                ViewMode::ContentView => file.text_scroll_x = file.text_scroll_x.saturating_add(1),
                _ => {}
            },
            KeyCode::Left => match file.mode {
                ViewMode::ListView => {
                    if let Some(index) = file_struct.parent_state.selected() {
                        let path = file_struct.parent.to_path_buf();
                        file_struct.present_dir_fn(path.as_path(), Some(index));
                    }
                }
                ViewMode::ContentView => file.text_scroll_x = file.text_scroll_x.saturating_sub(1),
                _ => {}
            },
            _ => {}
        },
    }
}

fn handle_file_edit(
    code: KeyCode,
    file: &mut FileScout,
    mut file_struct: MutexGuard<FileStruct>,
    modifier: KeyModifiers,
) {
    let mut line = file.input.content.lines();
    match (code, modifier) {
        (KeyCode::Char('s'), KeyModifiers::CONTROL) => {
            file_struct.file_write(file.input.content.clone());
            let pwd = file_struct.pwd.to_path_buf();
            reset_mode(file);
            if let Some(index) = file_struct.current_state.selected() {
                file_struct.present_dir_fn(&pwd, Some(index));
            }
            file.text_scroll_y = 0;
            file.text_scroll_x = 0;
        }
        (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
            reset_mode(file);
            file.text_scroll_y = 0;
            file.text_scroll_x = 0;
        }
        (KeyCode::Char(ch), _) => {
            edit_content(file, ch);
            file.text_scroll_x = file.text_scroll_x.saturating_add(1)
        }
        (KeyCode::Enter, _) => {
            edit_content(file, '\n');
            file.text_scroll_y = file.text_scroll_y.saturating_add(1);
            file.text_scroll_x = 0;
            file.input.content.push_str("");
        }
        (KeyCode::Backspace, _) => {
            remove_at(file);
            file.text_scroll_x = file.text_scroll_x.saturating_sub(1);
        }
        (KeyCode::Tab, _) => {
            edit_content(file, ' ');
            file.text_scroll_x = file.text_scroll_x.saturating_add(1);
        }
        (KeyCode::Down, _) => {
            let mut lines = file.input.content.lines();
            if file.text_scroll_y < lines.clone().count().saturating_sub(1) {
                file.text_scroll_y = file.text_scroll_y.saturating_add(1);
                file.input.total_letter = lines.nth(file.text_scroll_y).unwrap_or_default().len();
                if file.text_scroll_x > file.input.total_letter {
                    file.text_scroll_x = file.input.total_letter
                }
            }
        }
        (KeyCode::Up, _) => {
            file.text_scroll_y = file.text_scroll_y.saturating_sub(1);
            file.input.total_letter = line.nth(file.text_scroll_y).unwrap_or_default().len();
            if file.text_scroll_x > file.input.total_letter {
                file.text_scroll_x = file.input.total_letter
            }
        }
        (KeyCode::Left, _) => {
            if file.text_scroll_x == 0 && file.text_scroll_y != 0 {
                file.text_scroll_y = file.text_scroll_y.saturating_sub(1);
                file.input.total_letter = line.nth(file.text_scroll_y).unwrap_or_default().len();
                file.text_scroll_x = file.input.total_letter;
                return;
            }
            file.text_scroll_x = file.text_scroll_x.saturating_sub(1);
        }
        (KeyCode::Right, _) => {
            if file.text_scroll_x == file.input.total_letter {
                file.text_scroll_x = 0;
                file.text_scroll_y = file.text_scroll_y.saturating_add(1);
                file.input.total_letter = line.nth(file.text_scroll_y).unwrap_or_default().len();
                return;
            }
            file.text_scroll_x = file.text_scroll_x.saturating_add(1);
        }
        _ => {}
    }
}

fn edit_content(file: &mut FileScout, ch: char) {
    let lines = file.input.content.lines();
    if file.input.content.lines().count() == 0 {
        file.input.content.insert(0, ch);
    } else {
        let mut list: Vec<String> = lines.map(|ln| ln.to_string()).collect();
        if list.len() > file.text_scroll_y {
            list[file.text_scroll_y].insert(file.text_scroll_x, ch);
            file.input.content = list.join("\n");
        } else {
            file.input.content.push(ch);
        }
    }
}

fn remove_at(file: &mut FileScout) {
    let lines = file.input.content.lines();
    if file.input.content.lines().count() != 0 {
        let mut list: Vec<String> = lines.map(|ln| ln.to_string()).collect();
        if list.len() > file.text_scroll_y {
            if list[file.text_scroll_y].len() >= file.text_scroll_x {
                list[file.text_scroll_y].remove(file.text_scroll_x.saturating_sub(1));
                file.input.content = list.join("\n");
            }
        } else {
            file.input.content.pop();
            file.text_scroll_y = file.text_scroll_y.saturating_sub(1)
        }
    }
}

fn handle_file_name(code: KeyCode, file: &mut FileScout, mut file_struct: MutexGuard<FileStruct>) {
    match code {
        KeyCode::Char(c) => file.input.content.push(c),
        KeyCode::Backspace => {
            file.input.content.pop();
        }
        KeyCode::Enter => {
            if file.mode == ViewMode::Create {
                match file_struct.create_file(&file.input.content) {
                    Ok(()) => {}
                    Err(error) => file_struct.error = Some(error),
                }
            } else if file.mode == ViewMode::Rename {
                file_struct.rename(&file.input.content);
            }
            reset_mode(file);
            let pwd = file_struct.pwd.to_path_buf();
            if let Some(index) = file_struct.current_state.selected() {
                file_struct.present_dir_fn(&pwd, Some(index));
            }
        }
        KeyCode::Esc => reset_mode(file),
        _ => {}
    }
}

fn reset_mode(file: &mut FileScout) {
    file.input.content.clear();
    file.mode = ViewMode::ListView;
}
