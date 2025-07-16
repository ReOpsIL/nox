use super::{Dialog, DialogResult, render_utils, dialog_utils};
use crossterm::event::KeyCode;
use ratatui::prelude::*;
use ratatui::widgets::Paragraph;

/// Confirmation dialog for destructive or important actions
#[derive(Debug, Clone)]
pub struct ConfirmationDialog {
    /// Dialog title
    pub title: String,
    /// Main message to display
    pub message: String,
    /// Optional detailed information
    pub details: Option<String>,
    /// Text for the confirm button
    pub confirm_text: String,
    /// Text for the cancel button
    pub cancel_text: String,
    /// Currently selected button (0 = confirm, 1 = cancel)
    pub selected_button: usize,
    /// Whether this is a destructive action (affects styling)
    pub destructive: bool,
}

impl ConfirmationDialog {
    /// Create a new confirmation dialog
    pub fn new(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            message: message.into(),
            details: None,
            confirm_text: "Yes".to_string(),
            cancel_text: "No".to_string(),
            selected_button: 0,
            destructive: false,
        }
    }
    
    /// Set detailed information to display
    pub fn details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }
    
    /// Set custom button text
    pub fn buttons(mut self, confirm_text: impl Into<String>, cancel_text: impl Into<String>) -> Self {
        self.confirm_text = confirm_text.into();
        self.cancel_text = cancel_text.into();
        self
    }
    
    /// Mark this as a destructive action
    pub fn destructive(mut self) -> Self {
        self.destructive = true;
        self
    }
    
    /// Set the initially selected button
    pub fn default_button(mut self, button: usize) -> Self {
        self.selected_button = button.min(1);
        self
    }
    
    /// Create a delete confirmation dialog
    pub fn delete(item_type: &str, item_name: &str) -> Self {
        Self::new(
            format!("Delete {}", item_type),
            format!("Are you sure you want to delete '{}'?", item_name),
        )
        .details(format!("This action cannot be undone. The {} will be permanently removed.", item_type))
        .buttons("Delete", "Cancel")
        .destructive()
        .default_button(1) // Default to cancel for destructive actions
    }
    
    /// Create a stop agent confirmation dialog
    pub fn stop_agent(agent_name: &str) -> Self {
        Self::new(
            "Stop Agent",
            format!("Are you sure you want to stop '{}'?", agent_name),
        )
        .details("The agent will be stopped and any running tasks will be cancelled.")
        .buttons("Stop", "Cancel")
        .destructive()
        .default_button(1) // Default to cancel
    }
    
    /// Create a cancel task confirmation dialog
    pub fn cancel_task(task_title: &str) -> Self {
        Self::new(
            "Cancel Task",
            format!("Are you sure you want to cancel '{}'?", task_title),
        )
        .details("The task will be cancelled and any progress will be lost.")
        .buttons("Cancel Task", "Keep Running")
        .destructive()
        .default_button(1) // Default to keep running
    }
    
    /// Create a generic confirmation dialog
    pub fn confirm(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(title, message)
            .buttons("Confirm", "Cancel")
    }
    
    /// Create a save confirmation dialog
    pub fn save_changes(item_type: &str) -> Self {
        Self::new(
            "Save Changes",
            format!("Do you want to save changes to {}?", item_type),
        )
        .details("Unsaved changes will be lost if you don't save.")
        .buttons("Save", "Don't Save")
    }
    
    /// Create an exit confirmation dialog
    pub fn exit() -> Self {
        Self::new(
            "Exit Application",
            "Are you sure you want to exit?",
        )
        .details("Any unsaved changes will be lost.")
        .buttons("Exit", "Stay")
        .default_button(1) // Default to stay
    }
}

impl Dialog for ConfirmationDialog {
    fn render(&self, frame: &mut Frame, area: Rect) {
        let border_style = if self.destructive {
            Style::default().fg(Color::Red)
        } else {
            Style::default().fg(Color::Cyan)
        };
        
        render_utils::render_dialog_overlay(frame, area, &self.title, |frame, inner_area| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),  // Message
                    Constraint::Min(0),     // Details (if any)
                    Constraint::Length(3),  // Buttons
                ])
                .split(inner_area);
            
            // Render main message
            let message_style = if self.destructive {
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
            };
            
            render_utils::render_centered_text(frame, chunks[0], &self.message, message_style);
            
            // Render details if provided
            if let Some(details) = &self.details {
                let details_area = if chunks[1].height > 2 {
                    Rect {
                        x: chunks[1].x,
                        y: chunks[1].y + 1,
                        width: chunks[1].width,
                        height: chunks[1].height - 2,
                    }
                } else {
                    chunks[1]
                };
                
                let details_style = Style::default().fg(Color::Gray);
                render_utils::render_centered_text(frame, details_area, details, details_style);
            }
            
            // Render buttons
            let buttons = [
                (self.confirm_text.as_str(), self.selected_button == 0),
                (self.cancel_text.as_str(), self.selected_button == 1),
            ];
            
            render_utils::render_button_bar(frame, chunks[2], &buttons, self.destructive);
        }, Some(border_style));
    }
    
    fn handle_input(&mut self, key: KeyCode) -> DialogResult {
        // Handle yes/no shortcuts first
        if let Some(result) = dialog_utils::handle_yes_no_shortcuts(key) {
            return result;
        }
        
        // Handle dialog navigation
        dialog_utils::handle_dialog_navigation(key, &mut self.selected_button, 2)
    }
    
    fn title(&self) -> &str {
        &self.title
    }
    
    fn can_cancel(&self) -> bool {
        true
    }
    
    fn can_confirm(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_confirmation_dialog_new() {
        let dialog = ConfirmationDialog::new("Test Title", "Test Message");
        
        assert_eq!(dialog.title, "Test Title");
        assert_eq!(dialog.message, "Test Message");
        assert!(dialog.details.is_none());
        assert_eq!(dialog.confirm_text, "Yes");
        assert_eq!(dialog.cancel_text, "No");
        assert_eq!(dialog.selected_button, 0);
        assert!(!dialog.destructive);
    }
    
    #[test]
    fn test_confirmation_dialog_builder() {
        let dialog = ConfirmationDialog::new("Test", "Message")
            .details("Extra details")
            .buttons("OK", "Cancel")
            .destructive()
            .default_button(1);
        
        assert_eq!(dialog.details, Some("Extra details".to_string()));
        assert_eq!(dialog.confirm_text, "OK");
        assert_eq!(dialog.cancel_text, "Cancel");
        assert!(dialog.destructive);
        assert_eq!(dialog.selected_button, 1);
    }
    
    #[test]
    fn test_delete_confirmation() {
        let dialog = ConfirmationDialog::delete("Agent", "Test Agent");
        
        assert_eq!(dialog.title, "Delete Agent");
        assert_eq!(dialog.message, "Are you sure you want to delete 'Test Agent'?");
        assert!(dialog.details.is_some());
        assert_eq!(dialog.confirm_text, "Delete");
        assert_eq!(dialog.cancel_text, "Cancel");
        assert!(dialog.destructive);
        assert_eq!(dialog.selected_button, 1); // Default to cancel
    }
    
    #[test]
    fn test_stop_agent_confirmation() {
        let dialog = ConfirmationDialog::stop_agent("Test Agent");
        
        assert_eq!(dialog.title, "Stop Agent");
        assert_eq!(dialog.message, "Are you sure you want to stop 'Test Agent'?");
        assert!(dialog.details.is_some());
        assert_eq!(dialog.confirm_text, "Stop");
        assert_eq!(dialog.cancel_text, "Cancel");
        assert!(dialog.destructive);
        assert_eq!(dialog.selected_button, 1);
    }
    
    #[test]
    fn test_cancel_task_confirmation() {
        let dialog = ConfirmationDialog::cancel_task("Test Task");
        
        assert_eq!(dialog.title, "Cancel Task");
        assert_eq!(dialog.message, "Are you sure you want to cancel 'Test Task'?");
        assert!(dialog.details.is_some());
        assert_eq!(dialog.confirm_text, "Cancel Task");
        assert_eq!(dialog.cancel_text, "Keep Running");
        assert!(dialog.destructive);
        assert_eq!(dialog.selected_button, 1);
    }
    
    #[test]
    fn test_generic_confirmation() {
        let dialog = ConfirmationDialog::confirm("Test", "Message");
        
        assert_eq!(dialog.title, "Test");
        assert_eq!(dialog.message, "Message");
        assert_eq!(dialog.confirm_text, "Confirm");
        assert_eq!(dialog.cancel_text, "Cancel");
        assert!(!dialog.destructive);
    }
    
    #[test]
    fn test_save_changes_confirmation() {
        let dialog = ConfirmationDialog::save_changes("document");
        
        assert_eq!(dialog.title, "Save Changes");
        assert_eq!(dialog.message, "Do you want to save changes to document?");
        assert!(dialog.details.is_some());
        assert_eq!(dialog.confirm_text, "Save");
        assert_eq!(dialog.cancel_text, "Don't Save");
    }
    
    #[test]
    fn test_exit_confirmation() {
        let dialog = ConfirmationDialog::exit();
        
        assert_eq!(dialog.title, "Exit Application");
        assert_eq!(dialog.message, "Are you sure you want to exit?");
        assert!(dialog.details.is_some());
        assert_eq!(dialog.confirm_text, "Exit");
        assert_eq!(dialog.cancel_text, "Stay");
        assert_eq!(dialog.selected_button, 1);
    }
    
    #[test]
    fn test_dialog_trait_implementation() {
        let dialog = ConfirmationDialog::new("Test", "Message");
        
        assert_eq!(dialog.title(), "Test");
        assert!(dialog.can_cancel());
        assert!(dialog.can_confirm());
    }
    
    #[test]
    fn test_dialog_input_handling() {
        let mut dialog = ConfirmationDialog::new("Test", "Message");
        
        // Test yes shortcut
        let result = dialog.handle_input(KeyCode::Char('y'));
        assert_eq!(result, DialogResult::Confirm);
        
        // Test no shortcut
        let result = dialog.handle_input(KeyCode::Char('n'));
        assert_eq!(result, DialogResult::Cancel);
        
        // Test escape
        let result = dialog.handle_input(KeyCode::Esc);
        assert_eq!(result, DialogResult::Cancel);
        
        // Test navigation
        dialog.selected_button = 0;
        let result = dialog.handle_input(KeyCode::Right);
        assert_eq!(result, DialogResult::Continue);
        assert_eq!(dialog.selected_button, 1);
        
        let result = dialog.handle_input(KeyCode::Left);
        assert_eq!(result, DialogResult::Continue);
        assert_eq!(dialog.selected_button, 0);
        
        // Test enter on confirm button
        dialog.selected_button = 0;
        let result = dialog.handle_input(KeyCode::Enter);
        assert_eq!(result, DialogResult::Confirm);
        
        // Test enter on cancel button
        dialog.selected_button = 1;
        let result = dialog.handle_input(KeyCode::Enter);
        assert_eq!(result, DialogResult::Cancel);
    }
    
    #[test]
    fn test_default_button_bounds() {
        let dialog = ConfirmationDialog::new("Test", "Message")
            .default_button(5); // Should be clamped to 1
        
        assert_eq!(dialog.selected_button, 1);
    }
    
    #[test]
    fn test_dialog_navigation_wrapping() {
        let mut dialog = ConfirmationDialog::new("Test", "Message");
        
        // Test right wrapping
        dialog.selected_button = 1;
        let result = dialog.handle_input(KeyCode::Right);
        assert_eq!(result, DialogResult::Continue);
        assert_eq!(dialog.selected_button, 0);
        
        // Test left wrapping
        dialog.selected_button = 0;
        let result = dialog.handle_input(KeyCode::Left);
        assert_eq!(result, DialogResult::Continue);
        assert_eq!(dialog.selected_button, 1);
    }
}