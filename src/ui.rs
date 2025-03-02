use std::io::Result;

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Margin, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph, StatefulWidget, Widget},
    DefaultTerminal,
};

use crate::explorer::FileStruct;

pub enum ViewMode {
    ListView,
    ContentView,
}
pub struct FileScout {
    pub files: FileStruct,
    pub text_scroll_y: u16,
    pub text_scroll_x: u16,
    pub mode: ViewMode,
    pub exit: bool,
}

impl FileScout {
    pub fn new(files: FileStruct) -> Self {
        Self {
            files,
            mode: ViewMode::ListView,
            text_scroll_y: 0,
            text_scroll_x: 0,
            exit: false,
        }
    }

    pub fn run(mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        use crate::key_events::handle_events;
        while !self.exit {
            terminal.draw(|frame| frame.render_widget(&mut self, frame.area()))?;
            handle_events(&mut self);
        }
        Ok(())
    }
}

impl Widget for &mut FileScout {
    fn render(self, area: Rect, buf: &mut Buffer) {
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

        self.render_pwd(pwd_area, buf);
        self.render_current(current_dir, buf);
        self.render_parent(parent_dir, buf);
        if !self.files.current_dir.is_empty() {
            match self.files.current_state.selected() {
                Some(index) => {
                    if self.files.current_dir[index].is_file() {
                        self.render_content(files, buf);
                    } else {
                        self.render_sub(files, buf);
                    }
                }
                None => {}
            }
        }
        self.render_message(message, buf);
    }
}

impl FileScout {
    fn render_pwd(&self, area: Rect, buf: &mut Buffer) {
        Paragraph::new(Text::from(self.files.pwd.to_str().unwrap())).render(area, buf);
    }

    fn render_parent(&mut self, area: Rect, buf: &mut Buffer) {
        let padded_area = area.inner(Margin::new(1, 0));

        let files = self
            .files
            .parent_dir
            .iter()
            .enumerate()
            .map(|(index, name)| {
                let value = name
                    .strip_prefix(&self.files.parent)
                    .unwrap()
                    .to_str()
                    .unwrap();
                if name.is_dir() {
                    if self.files.pwd == self.files.parent_dir[index] {
                        self.files.parent_state.select(Some(index));
                    }
                    ListItem::new(Line::from(value).fg(Color::Blue))
                } else {
                    ListItem::new(Line::from(value))
                }
            });

        let list = List::new(files)
            .highlight_style(Style::new().bg(Color::Blue).fg(Color::White))
            .scroll_padding(18);
        StatefulWidget::render(list, padded_area, buf, &mut self.files.parent_state);
    }

    fn render_current(&mut self, area: Rect, buf: &mut Buffer) {
        Block::bordered()
            .borders(Borders::LEFT | Borders::RIGHT)
            .border_style(Style::default())
            .render(area, buf);

        let padded_area = area.inner(Margin::new(1, 0));

        let files = self.files.current_dir.iter().enumerate().map(|(_, name)| {
            let value = name
                .strip_prefix(&self.files.pwd)
                .unwrap()
                .to_str()
                .unwrap();
            if name.is_dir() {
                ListItem::new(Line::from(value).fg(Color::Blue))
            } else {
                ListItem::new(Line::from(value))
            }
        });

        let list = List::new(files)
            .highlight_style(Style::new().bg(Color::Blue).fg(Color::White))
            .scroll_padding(18);
        if !list.is_empty() {
            StatefulWidget::render(list, padded_area, buf, &mut self.files.current_state);
        } else {
            Widget::render(Text::from("No items"), padded_area, buf);
        }
    }

    fn render_sub(&mut self, area: Rect, buf: &mut Buffer) {
        let padded_area = area.inner(Margin::new(1, 0));

        let files = self.files.next_dir.iter().enumerate().map(|(_, name)| {
            let value = name
                .strip_prefix(&self.files.next)
                .unwrap()
                .to_str()
                .unwrap();
            if name.is_dir() {
                ListItem::new(Line::from(value).fg(Color::Blue))
            } else {
                ListItem::new(Line::from(value))
            }
        });

        let list = List::new(files)
            .highlight_style(Style::new().bg(Color::Blue).fg(Color::White))
            .scroll_padding(18);
        if list.is_empty() {
            Widget::render(Text::from("No items"), area, buf);
        } else {
            Widget::render(list, padded_area, buf);
        }
    }

    fn render_content(&mut self, area: Rect, buf: &mut Buffer) {
        match &self.files.content {
            Some(content) => {
                let text = Text::from(format!("{}", content));
                Paragraph::new(text)
                    .scroll((self.text_scroll_y, self.text_scroll_x))
                    .render(area, buf);
            }
            None => {
                Widget::render(Text::from("No content"), area, buf);
            }
        }
    }

    fn render_message(&mut self, area: Rect, buf: &mut Buffer) {
        #[cfg(unix)]
        Paragraph::new(Text::from(self.files.permission.as_str()).left_aligned().bold())
                .style(Color::Blue)
                .left_aligned()
                .render(area, buf);
        if let Some(error) = &self.files.error {
            Paragraph::new(Text::from(error.to_string()).left_aligned().bold())
                .style(Color::Red)
                .right_aligned()
                .render(area, buf);
        }
    }
}
