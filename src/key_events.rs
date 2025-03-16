use std::io::{Error, ErrorKind};

use crate::{
    constant::COLORS,
    ui::{FileScout, ViewMode},
};
use crossterm::event::{self, KeyCode, KeyEvent, KeyEventKind};

pub fn handle_events(file: &mut FileScout) {
    if let Ok(event::Event::Key(KeyEvent { code, kind, .. })) = event::read() {
        if kind == KeyEventKind::Press {
            match code {
                KeyCode::Char('q') | KeyCode::Char('Q') => file.exit = true,
                KeyCode::Char('e') | KeyCode::Char('E') => {
                    if let Some(index) = file.files.current_state.selected() {
                        let path = file.files.current_dir[index].to_path_buf();
                        if path.is_file() {
                            file.files.encrypt_file(path.as_path());
                            file.files.present_dir_fn(
                                file.files.pwd.to_path_buf().as_path(),
                                Some(index),
                            );
                        } else {
                            file.files.error = Some(Error::new(
                                ErrorKind::IsADirectory,
                                "can't encrypt directory",
                            ))
                        }
                    }
                }
                KeyCode::Char('d') | KeyCode::Char('D') => {
                    if let Some(index) = file.files.current_state.selected() {
                        let path = file.files.current_dir[index].to_path_buf();
                        if path.is_file() {
                            file.files.decrypt_file(&path);
                            file.files.present_dir_fn(
                                file.files.pwd.to_path_buf().as_path(),
                                Some(index),
                            );
                        } else {
                            file.files.error =
                                Some(Error::new(ErrorKind::IsADirectory, "Not allowed!"))
                        }
                    }
                }
                KeyCode::Char('c') | KeyCode::Char('C') => {
                    file.color_index = (file.color_index + 1) % COLORS.len()
                }
                KeyCode::Delete => {
                    if let Some(index) = file.files.current_state.selected() {
                        let path = file.files.current_dir[index].to_path_buf();
                        file.files.delete(&path.as_path());
                        let path = file.files.pwd.to_path_buf();
                        let index = if index == 0 { 0 } else { index - 1 };
                        file.files.present_dir_fn(path.as_path(), Some(index));
                        file.files.current_state.select(Some(index));
                    }
                }
                KeyCode::Tab => match file.mode {
                    ViewMode::ContentView => {
                        file.mode = ViewMode::ListView;
                        file.text_scroll_x = 0;
                        file.text_scroll_y = 0;
                    }
                    ViewMode::ListView => match file.files.current_state.selected() {
                        Some(index) => {
                            if file.files.current_dir[index].is_file() {
                                file.mode = ViewMode::ContentView
                            }
                        }
                        None => {}
                    },
                },
                KeyCode::Down => match file.mode {
                    ViewMode::ListView => {
                        file.files.current_state.select_next();
                        file.files.error = None;
                        if let Some(index) = file.files.current_state.selected() {
                            if file.files.current_dir.len() > index
                                && file.files.current_dir[index].is_dir()
                            {
                                let path = file.files.current_dir[index].to_path_buf();
                                file.files.next_dir_fn(&path.as_path());
                            } else if file.files.current_dir.len() > index {
                                let file_path = file.files.current_dir[index].to_path_buf();
                                file.files.read_file(file_path);
                            }
                        }
                    }
                    ViewMode::ContentView => {
                        if file.text_scroll_y < file.files.line_count.saturating_sub(1) as u16 {
                            file.text_scroll_y = file.text_scroll_y.saturating_add(1)
                        }
                    }
                },
                KeyCode::Up => match file.mode {
                    ViewMode::ListView => {
                        file.files.current_state.select_previous();
                        file.files.error = None;

                        if let Some(index) = file.files.current_state.selected() {
                            if file.files.current_dir.len() > index
                                && file.files.current_dir[index].is_dir()
                            {
                                let path = file.files.current_dir[index].to_path_buf();
                                file.files.next_dir_fn(&path.as_path());
                            } else if file.files.current_dir.len() > index {
                                let file_path = file.files.current_dir[index].to_path_buf();
                                file.files.read_file(file_path);
                            }
                        }
                    }
                    ViewMode::ContentView => {
                        file.text_scroll_y = file.text_scroll_y.saturating_sub(1)
                    }
                },
                KeyCode::Right => match file.mode {
                    ViewMode::ListView => {
                        if let Some(index) = file.files.current_state.selected() {
                            if file.files.current_dir.len() > index
                                && file.files.current_dir[index].is_dir()
                            {
                                let path = file.files.current_dir[index].to_path_buf();
                                file.files.present_dir_fn(path.as_path(), None);
                            } else if file.files.current_dir.len() > index {
                                let file_path = file.files.current_dir[index].to_path_buf();
                                file.files.read_file(file_path);
                            }
                        }
                    }
                    ViewMode::ContentView => {
                        file.text_scroll_x = file.text_scroll_x.saturating_add(1)
                    }
                },
                KeyCode::Left => match file.mode {
                    ViewMode::ListView => {
                        if let Some(index) = file.files.parent_state.selected() {
                            let path = file.files.parent.to_path_buf();
                            file.files.present_dir_fn(&path.as_path(), Some(index));
                            file.files.current_state.select(Some(index));
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
