use crate::tui::{
    app::{App, AppState, Screen, FormState, DialogState},
    screens::{Dashboard, AgentsScreen, TasksScreen, ExecutionScreen, LogsScreen},
    utils::colors::*,
    forms::Form,
    dialogs::Dialog,
};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style, Color},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Clear},
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
    
    // Render overlays
    render_overlays(frame, frame.size(), app);
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
        Screen::Dashboard => "📊 [1] Agents  [2] Tasks  [3] Execution  [4] Logs  [5] Logs2  [Tab] Next  [←/→] Navigate",
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

/// Render overlays (forms, dialogs, messages)
fn render_overlays(frame: &mut Frame, area: Rect, app: &App) {
    // Render loading indicator
    if app.state.operation_status.is_loading {
        render_loading_overlay(frame, area, &app.state.operation_status);
    }
    
    // Render form overlay
    if let Some(form_state) = &app.state.current_form {
        render_form_overlay(frame, area, form_state);
    }
    
    // Render dialog overlay
    if let Some(dialog_state) = &app.state.current_dialog {
        render_dialog_overlay(frame, area, dialog_state);
    }
    
    // Render messages
    render_messages(frame, area, app);
}

/// Render form overlay
fn render_form_overlay(frame: &mut Frame, area: Rect, form_state: &FormState) {
    match form_state {
        FormState::CreateAgent(form) => {
            form.render(frame, area);
        }
        FormState::EditAgent(form) => {
            form.render(frame, area);
        }
        FormState::CreateTask(form) => {
            form.render(frame, area);
        }
        FormState::EditTask(form) => {
            form.render(frame, area);
        }
    }
}

/// Render dialog overlay
fn render_dialog_overlay(frame: &mut Frame, area: Rect, dialog_state: &DialogState) {
    match dialog_state {
        DialogState::Confirmation(dialog) => {
            dialog.render(frame, area);
        }
        DialogState::Progress(dialog) => {
            dialog.render(frame, area);
        }
        DialogState::Help(dialog) => {
            dialog.render(frame, area);
        }
    }
}

/// Render loading overlay
fn render_loading_overlay(frame: &mut Frame, area: Rect, operation_status: &crate::tui::app::OperationStatus) {
    if let Some(operation_type) = &operation_status.operation_type {
        let loading_area = centered_rect(40, 10, area);
        
        // Clear the background
        frame.render_widget(Clear, loading_area);
        
        // Create loading message
        let message = if let Some(progress) = operation_status.progress {
            format!("{} ({}%)", operation_type, progress)
        } else {
            format!("{}...", operation_type)
        };
        
        let block = Block::default()
            .borders(Borders::ALL)
            .title("Loading")
            .border_style(Style::default().fg(Color::Cyan));
        
        let paragraph = Paragraph::new(message)
            .style(Style::default().fg(Color::White))
            .block(block)
            .alignment(ratatui::layout::Alignment::Center);
        
        frame.render_widget(paragraph, loading_area);
    }
}

/// Render success and error messages
fn render_messages(frame: &mut Frame, area: Rect, app: &App) {
    let message_height = 3;
    let message_width = 50;
    
    // Render error message
    if let Some(error) = &app.state.error_message {
        let error_area = Rect {
            x: area.width.saturating_sub(message_width + 2),
            y: area.height.saturating_sub(message_height * 2 + 2),
            width: message_width,
            height: message_height,
        };
        
        frame.render_widget(Clear, error_area);
        
        let block = Block::default()
            .borders(Borders::ALL)
            .title("Error")
            .border_style(Style::default().fg(Color::Red));
        
        let paragraph = Paragraph::new(error.as_str())
            .style(Style::default().fg(Color::Red))
            .block(block)
            .wrap(ratatui::widgets::Wrap { trim: true });
        
        frame.render_widget(paragraph, error_area);
    }
    
    // Render success message
    if let Some(success) = &app.state.success_message {
        let success_area = Rect {
            x: area.width.saturating_sub(message_width + 2),
            y: area.height.saturating_sub(message_height + 1),
            width: message_width,
            height: message_height,
        };
        
        frame.render_widget(Clear, success_area);
        
        let block = Block::default()
            .borders(Borders::ALL)
            .title("Success")
            .border_style(Style::default().fg(Color::Green));
        
        let paragraph = Paragraph::new(success.as_str())
            .style(Style::default().fg(Color::Green))
            .block(block)
            .wrap(ratatui::widgets::Wrap { trim: true });
        
        frame.render_widget(paragraph, success_area);
    }
}

/// Create a centered rect
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
