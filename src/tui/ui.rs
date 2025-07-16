use crate::tui::{
    app::{App, AppState, Screen},
    screens::{Dashboard, AgentsScreen, TasksScreen, ExecutionScreen, LogsScreen},
    utils::colors::*,
};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn render(frame: &mut Frame, app: &App) {
    let state = &app.state;
    
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Min(0),     // Main content
            Constraint::Length(3),  // Footer
        ])
        .split(frame.size());

    render_header(frame, chunks[0], state);
    render_main_content(frame, chunks[1], state);
    render_footer(frame, chunks[2], state);
}

fn render_header(frame: &mut Frame, area: Rect, _state: &AppState) {
    let title = "🚀 Nox Agent Ecosystem v0.1.0";
    let help_text = "[Q] Quit [H] Help";
    
    let header_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(help_text.len() as u16 + 2),
        ])
        .split(area);

    let title_block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_active_style());
    
    let title_paragraph = Paragraph::new(title)
        .style(title_style())
        .block(title_block);

    let help_block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style());
    
    let help_paragraph = Paragraph::new(help_text)
        .style(highlight_style())
        .block(help_block);

    frame.render_widget(title_paragraph, header_chunks[0]);
    frame.render_widget(help_paragraph, header_chunks[1]);
}

fn render_main_content(frame: &mut Frame, area: Rect, state: &AppState) {
    match state.current_screen {
        Screen::Dashboard => Dashboard::render(frame, area, state),
        Screen::Agents => AgentsScreen::render(frame, area, state),
        Screen::Tasks => TasksScreen::render(frame, area, state),
        Screen::Execution => ExecutionScreen::render(frame, area, state),
        Screen::Logs => LogsScreen::render(frame, area, state),
    }
}

fn render_footer(frame: &mut Frame, area: Rect, state: &AppState) {
    let navigation_text = match state.current_screen {
        Screen::Dashboard => "📊 [1] Agents  [2] Tasks  [3] Execution  [4] Logs  [Tab] Next  [←/→] Navigate",
        Screen::Agents => "🤖 [↑/↓] Select  [Enter] Action  [Tab] Switch Panel  [F1] Help",
        Screen::Tasks => "📋 [↑/↓] Select  [Enter] Action  [Tab] Switch Panel  [F] Filter",
        Screen::Execution => "⚡ [↑/↓] Select  [Enter] View Details  [Space] Pause/Resume  [Del] Cancel",
        Screen::Logs => "📜 [↑/↓] Scroll  [Home/End] Jump  [Space] Toggle Filter  [Enter] View Details",
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_active_style());
    
    let paragraph = Paragraph::new(navigation_text)
        .style(info_style())
        .block(block);

    frame.render_widget(paragraph, area);
}