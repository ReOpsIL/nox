use anyhow::Result;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use nox::tui::{app::App, events::EventHandler, ui};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use std::{
    io,
    time::Duration,
};
use tokio::time::interval;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    // Initialize logging

    let mut event_handler = EventHandler::new(Duration::from_millis(250));
    let event_sender = event_handler.sender();

    let mut app = App::new(event_sender.clone());

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    event_handler.start_event_loop().await?;

    let mut refresh_interval = interval(Duration::from_secs(1));

    app.refresh_data().await?;

    loop {
        terminal.draw(|f| ui::render(f, &app))?;

        tokio::select! {
            Some(event) = event_handler.next() => {
                match event {
                    nox::tui::events::AppEvent::Key(key) => {
                        if nox::tui::events::AppEvent::is_quit_key(&key) {
                            break;
                        }
                        if let Err(e) = app.handle_key_input(key.code) {
                            log::error!("Failed to handle key input: {}", e);
                        }
                    }
                    nox::tui::events::AppEvent::Tick => {
                        // Handle periodic updates
                    }
                    nox::tui::events::AppEvent::Quit => {
                        break;
                    }
                    _ => {}
                }

                if app.state.should_quit {
                    break;
                }
            }
            _ = refresh_interval.tick() => {
                if let Err(e) = app.refresh_data().await {
                    log::error!("Failed to refresh data: {}", e);
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}