use std::io;

use anyhow::Result;
use clap::Parser;
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use lazyinstall::ui::app::App;

#[derive(Parser, Debug)]
#[command(
    name = "lazyinstall",
    version = concat!(
        env!("CARGO_PKG_VERSION"),
        " (commit ", env!("LI_GIT_COMMIT"),
        ", built ", env!("LI_BUILD_DATE"),
        ", ", env!("LI_OS"), "/", env!("LI_ARCH"), ")",
    ),
    about = "A TUI to track folders holding update scripts and run them"
)]
struct Cli {}

fn main() -> Result<()> {
    let _cli = Cli::parse();
    let mut terminal = setup_terminal()?;
    let result = App::new().run(&mut terminal);
    restore_terminal(&mut terminal)?;
    result
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    Ok(Terminal::new(backend)?)
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}
