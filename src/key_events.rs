use crate::ui::FileScout;
use crossterm::event::{self, KeyCode, KeyEvent};

pub fn handle_events(file: &mut FileScout) {
    if let Ok(event::Event::Key(KeyEvent { code, .. })) = event::read() {
        match code {
            KeyCode::Char('q') => file.exit = true,
            KeyCode::Down => {
                file.files.current_state.select_next();
                match file.files.current_state.selected() {
                    Some(index) => {
                        if file.files.current_dir.len() > index
                            && file.files.current_dir[index].is_dir()
                        {
                            let path = file.files.current_dir[index].to_path_buf();
                            file.files.next_dir_fn(&path.as_path());
                        } else {
                            file.files.next_dir.clear();
                        }
                    }
                    None => {}
                }
            }
            KeyCode::Up => {
                file.files.current_state.select_previous();
                match file.files.current_state.selected() {
                    Some(index) => {
                        if file.files.current_dir.len() > index
                            && file.files.current_dir[index].is_dir()
                        {
                            let path = file.files.current_dir[index].to_path_buf();
                            file.files.next_dir_fn(&path.as_path());
                        } else {
                            file.files.next_dir.clear();
                        }
                    }
                    None => {}
                }
            }
            KeyCode::Right => match file.files.current_state.selected() {
                Some(index) => {
                    if file.files.current_dir.len() > index
                        && file.files.current_dir[index].is_dir()
                    {
                        let path = file.files.current_dir[index].to_path_buf();
                        file.files.present_dir_fn(path.as_path());
                    }
                }
                None => {}
            },
            KeyCode::Left => match file.files.parent_state.selected() {
                Some(index) => {
                    let path = file.files.parent.to_path_buf();
                    file.files.present_dir_fn(&path.as_path());
                    file.files.current_state.select(Some(index))
                }
                None => {}
            },
            _ => {}
        }
    }
}
