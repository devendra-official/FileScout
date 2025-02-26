use crate::ui::FileScout;
use crossterm::event::{self, KeyCode, KeyEvent};

pub fn handle_events(file: &mut FileScout) {
    if let Ok(event::Event::Key(KeyEvent { code, .. })) = event::read() {
        match code {
            KeyCode::Char('q') => file.exit = true,
            KeyCode::Down => {
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
            KeyCode::Up => {
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
            KeyCode::Right => {
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
            KeyCode::Left => {
                if let Some(index) = file.files.parent_state.selected() {
                    let path = file.files.parent.to_path_buf();
                    file.files.present_dir_fn(&path.as_path(), Some(index));
                    file.files.current_state.select(Some(index));
                }
            }
            _ => {}
        }
    }
}
