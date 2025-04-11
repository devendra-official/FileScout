use std::{
    io::Result,
    sync::{Arc, Mutex},
};

use crossterm::event::{Event, EventStream, KeyEvent, KeyEventKind};
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Flex, Layout, Margin, Position, Rect},
    style::{Style, Stylize},
    text::{Line, Text},
    widgets::{Block, Borders, Clear as Clean, List, ListItem, Paragraph, StatefulWidget, Widget},
    DefaultTerminal, Frame,
};
use tokio::sync::mpsc;
use tokio_stream::StreamExt;

use crate::{constant::COLORS, explorer::FileStruct};

#[derive(PartialEq)]
pub enum ViewMode {
    ListView,
    ContentView,
    Rename,
    Create,
    FileEdit,
}

#[derive(Default)]
pub struct Input {
    pub content: String,
    pub total_lines: usize,
    pub total_letter: usize,
}

pub struct FileScout {
    pub files: Arc<Mutex<FileStruct>>,
    pub input: Input,
    pub text_scroll_y: usize,
    pub text_scroll_x: usize,
    pub color_index: usize,
    pub mode: ViewMode,
    pub exit: bool,
}

impl FileScout {
    pub fn new(files: FileStruct) -> Self {
        Self {
            files: Arc::new(Mutex::new(files)),
            mode: ViewMode::ListView,
            input: Input::default(),
            text_scroll_y: 0,
            text_scroll_x: 0,
            color_index: 0,
            exit: false,
        }
    }

    pub async fn run(mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        use crate::key_events::handle_events;
        let mut reader = EventStream::new();
        let (tx, mut rx) = mpsc::channel::<String>(1);
        while !self.exit {
            terminal.draw(|frame| {
                if self.mode == ViewMode::FileEdit {
                    self.handle_curson(frame);
                }
                frame.render_widget(&mut self, frame.area());
            })?;
            tokio::select! {
                Some(Ok(event)) = reader.next() => {
                    match event {
                        Event::Key(KeyEvent { code, modifiers,kind: KeyEventKind::Press, .. }) => {
                            handle_events(&mut self, code, tx.clone(),modifiers);
                        }
                        Event::Resize(_, _) => continue,
                        _ => {}
                    }
                }
                Some(_) = rx.recv() => continue,
            }
        }
        Ok(())
    }

    fn handle_curson(&self, frame: &mut Frame) {
        frame.set_cursor_position(Position::new(
            (self.text_scroll_x + 1) as u16,
            (self.text_scroll_y + 1) as u16,
        ));
    }
}

impl Widget for &mut FileScout {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let files_struct_clone = Arc::clone(&self.files);
        let mut file_ex = files_struct_clone.lock().unwrap();

        if self.mode == ViewMode::FileEdit {
            self.render_file_content(area, buf, &mut file_ex);
            return;
        }

        let [pwd_area, files_area, message] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .areas(area);

        let [parent_dir, current_dir, files] = Layout::horizontal([
            Constraint::Percentage(15),
            Constraint::Fill(1),
            Constraint::Fill(1),
        ])
        .areas(files_area);

        self.render_pwd(pwd_area, buf, &mut file_ex);
        self.render_current(current_dir, buf, &mut file_ex);
        self.render_parent(parent_dir, buf, &mut file_ex);

        if !file_ex.current_dir.is_empty() {
            if let Some(index) = file_ex.current_state.selected() {
                if file_ex.current_dir[index].is_file() {
                    self.render_content(files, buf, &mut file_ex);
                } else {
                    self.render_sub(files, buf, &mut file_ex);
                }
            }
        }
        self.render_message(message, buf, &mut file_ex);
    }
}

impl FileScout {
    fn render_file_content(&self, area: Rect, buf: &mut Buffer, _file_struct: &mut FileStruct) {
        let (sel_color, ..) = COLORS[self.color_index];
        let instruction = Line::from(vec![
            " Save ".into(),
            " <Ctrl+S> ".blue().bold().fg(sel_color),
            " Quit ".into(),
            " <Ctrl+C> ".blue().bold().fg(sel_color),
        ]);

        let block = Block::bordered().title_bottom(instruction.centered());
        let text = Text::from(self.input.content.to_string());
        Paragraph::new(text).block(block).render(area, buf);
    }

    fn render_pwd(&self, area: Rect, buf: &mut Buffer, file_struct: &mut FileStruct) {
        let (sel_color, ..) = COLORS[self.color_index];
        Paragraph::new(Text::from(file_struct.pwd.to_str().unwrap().fg(sel_color)))
            .render(area, buf);
    }

    fn render_parent(&mut self, area: Rect, buf: &mut Buffer, file_struct: &mut FileStruct) {
        let padded_area = area.inner(Margin::new(1, 0));
        let (sel_color, un_color) = COLORS[self.color_index];

        let selected_index = file_struct
            .parent_dir
            .iter()
            .position(|path| file_struct.pwd == *path);

        file_struct.parent_state.select(selected_index);
        let files = file_struct.parent_dir.iter().map(|name| {
            let value = name
                .strip_prefix(&file_struct.parent)
                .unwrap()
                .to_str()
                .unwrap();
            if name.is_dir() {
                ListItem::new(Line::from(value).fg(sel_color))
            } else {
                ListItem::new(Line::from(value).fg(un_color))
            }
        });

        let list = List::new(files)
            .highlight_style(Style::new().bg(sel_color).fg(un_color))
            .scroll_padding(18);

        StatefulWidget::render(list, padded_area, buf, &mut file_struct.parent_state);
    }

    fn render_current(&mut self, area: Rect, buf: &mut Buffer, file_struct: &mut FileStruct) {
        let (sel_color, un_color) = COLORS[self.color_index];
        Block::bordered()
            .borders(Borders::LEFT | Borders::RIGHT)
            .border_style(Style::new().fg(sel_color))
            .render(area, buf);

        let padded_area = area.inner(Margin::new(1, 0));

        let files = file_struct.current_dir.iter().map(|name| {
            let value = name
                .strip_prefix(&file_struct.pwd)
                .unwrap()
                .to_str()
                .unwrap();
            if name.is_dir() {
                ListItem::new(Line::from(value).fg(sel_color))
            } else {
                ListItem::new(Line::from(value).fg(un_color))
            }
        });
        let list = List::new(files)
            .highlight_style(Style::new().bg(sel_color).fg(un_color))
            .scroll_padding(18);
        if !list.is_empty() {
            StatefulWidget::render(list, padded_area, buf, &mut file_struct.current_state);
        } else {
            Widget::render(Text::from("No items"), padded_area, buf);
        }

        match self.mode {
            ViewMode::Rename => self.render_window(padded_area, buf, " Rename "),
            ViewMode::Create => self.render_window(padded_area, buf, " New File "),
            _ => {}
        }
    }

    fn render_window(&mut self, area: Rect, buf: &mut Buffer, title: &str) {
        let (sel_color, _) = COLORS[self.color_index];
        let [window] = Layout::horizontal([Constraint::Percentage(80)]).areas(area);
        let [window] = Layout::vertical([Constraint::Length(3)])
            .flex(Flex::Center)
            .areas(window);

        Clean.render(window, buf);
        let title = Line::from(title);
        let block = Block::bordered()
            .title(title)
            .title_alignment(Alignment::Left)
            .border_style(Style::new().fg(sel_color));

        Paragraph::new(self.input.content.as_str())
            .block(block)
            .render(window, buf);
    }

    fn render_sub(&mut self, area: Rect, buf: &mut Buffer, file_struct: &mut FileStruct) {
        let padded_area = area.inner(Margin::new(1, 0));
        let (sel_color, un_color) = COLORS[self.color_index];

        let files = file_struct.next_dir.iter().map(|name| {
            let value = name
                .strip_prefix(&file_struct.next)
                .unwrap()
                .to_str()
                .unwrap();
            if name.is_dir() {
                ListItem::new(Line::from(value).fg(sel_color))
            } else {
                ListItem::new(Line::from(value).fg(un_color))
            }
        });

        let list = List::new(files)
            .highlight_style(Style::new().bg(sel_color).fg(un_color))
            .scroll_padding(18);
        if list.is_empty() {
            Widget::render(Text::from("No items").fg(sel_color), area, buf);
        } else {
            Widget::render(list, padded_area, buf);
        }
    }

    fn render_content(&mut self, area: Rect, buf: &mut Buffer, file_struct: &mut FileStruct) {
        let (sel_col, un_col) = COLORS[self.color_index];
        if !file_struct.content.is_empty() {
            let text = Text::from(file_struct.content.to_string());
            Paragraph::new(text.fg(un_col))
                .scroll((self.text_scroll_y as u16, self.text_scroll_x as u16))
                .render(area, buf);
        } else {
            Widget::render(Text::from("No content").fg(sel_col), area, buf);
        }
    }

    fn render_message(&mut self, area: Rect, buf: &mut Buffer, file_struct: &mut FileStruct) {
        #[cfg(unix)]
        let (sel_color, _) = COLORS[self.color_index];
        #[cfg(unix)]
        Paragraph::new(
            Text::from(file_struct.permission.as_str())
                .left_aligned()
                .bold(),
        )
        .style(sel_color)
        .left_aligned()
        .render(area, buf);

        if let Some(error) = &file_struct.error {
            Paragraph::new(Text::from(error.to_string()).left_aligned().bold())
                .right_aligned()
                .render(area, buf);
        }
    }
}
