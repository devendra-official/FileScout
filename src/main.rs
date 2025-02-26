mod explorer;
mod key_events;
mod ui;

use explorer::FileStruct;
use ratatui::DefaultTerminal;
use std::path::Path;
use ui::FileScout;

fn main() {
    let mut file = FileStruct::default();
    file.present_dir_fn(Path::new("."),None);
    let mut terminal: DefaultTerminal = ratatui::init();
    let app = FileScout::new(file);
    app.run(&mut terminal).unwrap();
    ratatui::restore();
}
