mod app;
mod editor;
mod filetree;
mod shell;
mod theme;
mod ui;

use std::io;
use std::time::Duration;

use crossterm::event::{self, Event};
use crossterm::execute;
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;

use app::App;

fn main() -> io::Result<()> {
    // Panic hook to restore terminal on crash
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
        original_hook(panic_info);
    }));

    // Terminal setup
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();

    // Initialize shell pane with terminal size
    let size = terminal.size()?;
    // Shell pane is ~40% width, full height minus borders and status bar
    let shell_cols = (size.width as f32 * 0.40) as u16;
    let shell_rows = size.height.saturating_sub(3); // borders + status bar
    app.init_shell(shell_rows, shell_cols);

    // Main loop
    loop {
        // Sync file tree if shell changed directories
        app.sync_shell_cwd();

        terminal.draw(|frame| ui::draw(frame, &mut app))?;

        // Use a short poll timeout so PTY output renders promptly
        if event::poll(Duration::from_millis(16))? {
            match event::read()? {
                Event::Key(key) => {
                    app.handle_key(key);
                }
                Event::Resize(_, _) => {
                    // ratatui handles resize automatically on next draw
                }
                _ => {}
            }
        }

        if app.should_quit {
            break;
        }
    }

    // Terminal teardown
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

    Ok(())
}
