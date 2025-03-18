mod constant;
mod explorer;
mod key_events;
mod ui;
mod crypto_handler;

use color_eyre::eyre::Result;
use explorer::FileStruct;
use ratatui::DefaultTerminal;
use std::path::Path;
use ui::FileScout;

fn main() -> Result<()> {
    color_eyre::install()?;
    let mut file = FileStruct::default();
    file.present_dir_fn(Path::new("."), None);
    let mut terminal: DefaultTerminal = ratatui::init();
    let app = FileScout::new(file);
    app.run(&mut terminal)?;
    ratatui::restore();
    Ok(())
}
