pub mod confirmation;
pub mod progress;
pub mod help;

use crossterm::event::KeyCode;
use ratatui::prelude::*;

/// Result of dialog interactions
#[derive(Debug, Clone, PartialEq)]
pub enum DialogResult {
    /// Continue processing input
    Continue,
    /// User confirmed the action
    Confirm,
    /// User cancelled the action
    Cancel,
    /// Close the dialog
    Close,
}

/// Base trait for all dialog components
pub trait Dialog {
    /// Render the dialog
    fn render(&self, frame: &mut Frame, area: Rect);
    
    /// Handle keyboard input
    fn handle_input(&mut self, key: KeyCode) -> DialogResult;
    
    /// Get the dialog title
    fn title(&self) -> &str;
    
    /// Check if the dialog can be cancelled
    fn can_cancel(&self) -> bool {
        true
    }
    
    /// Check if the dialog can be confirmed
    fn can_confirm(&self) -> bool {
        true
    }
}

/// Common dialog rendering utilities
pub mod render_utils {
    use ratatui::prelude::*;
    use ratatui::widgets::{Block, Borders, Clear, Paragraph};
    
    /// Create a centered dialog area
    pub fn centered_dialog_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
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
    
    /// Render a dialog overlay with title and content
    pub fn render_dialog_overlay(
        frame: &mut Frame,
        area: Rect,
        title: &str,
        content_render: impl FnOnce(&mut Frame, Rect),
        border_style: Option<Style>,
    ) {
        let dialog_area = centered_dialog_rect(60, 40, area);
        
        // Clear the background
        frame.render_widget(Clear, dialog_area);
        
        // Create the main dialog block
        let block = Block::default()
            .borders(Borders::ALL)
            .title(title)
            .border_style(border_style.unwrap_or_else(|| Style::default().fg(Color::Cyan)));
        
        // Render the block
        frame.render_widget(block, dialog_area);
        
        // Create inner area for content
        let inner_area = Rect {
            x: dialog_area.x + 1,
            y: dialog_area.y + 1,
            width: dialog_area.width.saturating_sub(2),
            height: dialog_area.height.saturating_sub(2),
        };
        
        // Render the content
        content_render(frame, inner_area);
    }
    
    /// Render button bar at the bottom of a dialog
    pub fn render_button_bar(
        frame: &mut Frame,
        area: Rect,
        buttons: &[(&str, bool)], // (label, selected)
        destructive: bool,
    ) {
        let button_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                buttons.iter()
                    .map(|_| Constraint::Percentage(100 / buttons.len() as u16))
                    .collect::<Vec<_>>()
            )
            .split(area);
        
        for (i, (label, selected)) in buttons.iter().enumerate() {
            if i < button_chunks.len() {
                let style = if *selected {
                    if destructive && i == 0 {
                        Style::default().bg(Color::Red).fg(Color::White).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().bg(Color::Blue).fg(Color::White).add_modifier(Modifier::BOLD)
                    }
                } else {
                    Style::default().fg(Color::Gray)
                };
                
                let button = Paragraph::new(*label)
                    .style(style)
                    .alignment(Alignment::Center);
                
                frame.render_widget(button, button_chunks[i]);
            }
        }
    }
    
    /// Render centered text in a dialog
    pub fn render_centered_text(
        frame: &mut Frame,
        area: Rect,
        text: &str,
        style: Style,
    ) {
        let paragraph = Paragraph::new(text)
            .style(style)
            .alignment(Alignment::Center)
            .wrap(ratatui::widgets::Wrap { trim: true });
        
        frame.render_widget(paragraph, area);
    }
}

/// Common dialog utilities
pub mod dialog_utils {
    use super::DialogResult;
    use crossterm::event::KeyCode;
    
    /// Handle common dialog navigation keys
    pub fn handle_dialog_navigation(
        key: KeyCode,
        current_button: &mut usize,
        button_count: usize,
    ) -> DialogResult {
        match key {
            KeyCode::Left | KeyCode::BackTab => {
                if button_count > 0 {
                    *current_button = if *current_button == 0 {
                        button_count - 1
                    } else {
                        *current_button - 1
                    };
                }
                DialogResult::Continue
            }
            KeyCode::Right | KeyCode::Tab => {
                if button_count > 0 {
                    *current_button = (*current_button + 1) % button_count;
                }
                DialogResult::Continue
            }
            KeyCode::Enter => {
                match *current_button {
                    0 => DialogResult::Confirm,
                    _ => DialogResult::Cancel,
                }
            }
            KeyCode::Esc => DialogResult::Cancel,
            KeyCode::Char('y') | KeyCode::Char('Y') => DialogResult::Confirm,
            KeyCode::Char('n') | KeyCode::Char('N') => DialogResult::Cancel,
            _ => DialogResult::Continue,
        }
    }
    
    /// Handle yes/no dialog shortcuts
    pub fn handle_yes_no_shortcuts(key: KeyCode) -> Option<DialogResult> {
        match key {
            KeyCode::Char('y') | KeyCode::Char('Y') => Some(DialogResult::Confirm),
            KeyCode::Char('n') | KeyCode::Char('N') => Some(DialogResult::Cancel),
            KeyCode::Esc => Some(DialogResult::Cancel),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::dialog_utils::*;
    
    #[test]
    fn test_dialog_result_equality() {
        assert_eq!(DialogResult::Continue, DialogResult::Continue);
        assert_eq!(DialogResult::Confirm, DialogResult::Confirm);
        assert_eq!(DialogResult::Cancel, DialogResult::Cancel);
        assert_eq!(DialogResult::Close, DialogResult::Close);
        
        assert_ne!(DialogResult::Continue, DialogResult::Confirm);
        assert_ne!(DialogResult::Confirm, DialogResult::Cancel);
        assert_ne!(DialogResult::Cancel, DialogResult::Close);
    }
    
    #[test]
    fn test_dialog_navigation() {
        let mut current_button = 0;
        let button_count = 2;
        
        // Test right navigation
        let result = handle_dialog_navigation(KeyCode::Right, &mut current_button, button_count);
        assert_eq!(result, DialogResult::Continue);
        assert_eq!(current_button, 1);
        
        // Test wrapping
        let result = handle_dialog_navigation(KeyCode::Right, &mut current_button, button_count);
        assert_eq!(result, DialogResult::Continue);
        assert_eq!(current_button, 0);
        
        // Test left navigation
        let result = handle_dialog_navigation(KeyCode::Left, &mut current_button, button_count);
        assert_eq!(result, DialogResult::Continue);
        assert_eq!(current_button, 1);
        
        // Test enter on first button
        current_button = 0;
        let result = handle_dialog_navigation(KeyCode::Enter, &mut current_button, button_count);
        assert_eq!(result, DialogResult::Confirm);
        
        // Test enter on second button
        current_button = 1;
        let result = handle_dialog_navigation(KeyCode::Enter, &mut current_button, button_count);
        assert_eq!(result, DialogResult::Cancel);
        
        // Test escape
        let result = handle_dialog_navigation(KeyCode::Esc, &mut current_button, button_count);
        assert_eq!(result, DialogResult::Cancel);
    }
    
    #[test]
    fn test_yes_no_shortcuts() {
        assert_eq!(handle_yes_no_shortcuts(KeyCode::Char('y')), Some(DialogResult::Confirm));
        assert_eq!(handle_yes_no_shortcuts(KeyCode::Char('Y')), Some(DialogResult::Confirm));
        assert_eq!(handle_yes_no_shortcuts(KeyCode::Char('n')), Some(DialogResult::Cancel));
        assert_eq!(handle_yes_no_shortcuts(KeyCode::Char('N')), Some(DialogResult::Cancel));
        assert_eq!(handle_yes_no_shortcuts(KeyCode::Esc), Some(DialogResult::Cancel));
        assert_eq!(handle_yes_no_shortcuts(KeyCode::Char('x')), None);
    }
    
    #[test]
    fn test_dialog_navigation_edge_cases() {
        let mut current_button = 0;
        let button_count = 0;
        
        // Test with no buttons
        let result = handle_dialog_navigation(KeyCode::Right, &mut current_button, button_count);
        assert_eq!(result, DialogResult::Continue);
        assert_eq!(current_button, 0);
        
        // Test with single button
        let button_count = 1;
        let result = handle_dialog_navigation(KeyCode::Right, &mut current_button, button_count);
        assert_eq!(result, DialogResult::Continue);
        assert_eq!(current_button, 0);
        
        let result = handle_dialog_navigation(KeyCode::Left, &mut current_button, button_count);
        assert_eq!(result, DialogResult::Continue);
        assert_eq!(current_button, 0);
    }
}