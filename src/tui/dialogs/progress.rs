use super::{Dialog, DialogResult, render_utils};
use crossterm::event::KeyCode;
use ratatui::prelude::*;
use ratatui::widgets::{Paragraph, Gauge, Block, Borders};
use std::time::Instant;

/// Progress dialog for long-running operations
#[derive(Debug, Clone)]
pub struct ProgressDialog {
    /// Dialog title
    pub title: String,
    /// Current operation message
    pub message: String,
    /// Current progress percentage (0-100, None for indeterminate)
    pub progress: Option<u8>,
    /// Whether the operation can be cancelled
    pub can_cancel: bool,
    /// Whether the operation is complete
    pub is_complete: bool,
    /// Whether the operation failed
    pub is_failed: bool,
    /// Error message if operation failed
    pub error_message: Option<String>,
    /// Success message if operation completed
    pub success_message: Option<String>,
    /// Start time of the operation
    pub start_time: Instant,
    /// Current spinner frame for indeterminate progress
    pub spinner_frame: usize,
    /// Whether to show elapsed time
    pub show_elapsed_time: bool,
    /// Additional details or logs
    pub details: Vec<String>,
    /// Maximum number of detail lines to show
    pub max_details: usize,
}

impl ProgressDialog {
    /// Create a new progress dialog
    pub fn new(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            message: message.into(),
            progress: None,
            can_cancel: true,
            is_complete: false,
            is_failed: false,
            error_message: None,
            success_message: None,
            start_time: Instant::now(),
            spinner_frame: 0,
            show_elapsed_time: false,
            details: Vec::new(),
            max_details: 10,
        }
    }
    
    /// Set the progress percentage
    pub fn set_progress(&mut self, progress: u8) {
        self.progress = Some(progress.min(100));
    }
    
    /// Set indeterminate progress (spinner)
    pub fn set_indeterminate(&mut self) {
        self.progress = None;
    }
    
    /// Update the operation message
    pub fn set_message(&mut self, message: impl Into<String>) {
        self.message = message.into();
    }
    
    /// Set whether the operation can be cancelled
    pub fn set_cancellable(&mut self, can_cancel: bool) {
        self.can_cancel = can_cancel;
    }
    
    /// Mark the operation as complete with success
    pub fn set_complete(&mut self, success_message: impl Into<String>) {
        self.is_complete = true;
        self.is_failed = false;
        self.success_message = Some(success_message.into());
        self.progress = Some(100);
    }
    
    /// Mark the operation as failed
    pub fn set_failed(&mut self, error_message: impl Into<String>) {
        self.is_complete = true;
        self.is_failed = true;
        self.error_message = Some(error_message.into());
    }
    
    /// Add a detail line (for logging)
    pub fn add_detail(&mut self, detail: impl Into<String>) {
        self.details.push(detail.into());
        if self.details.len() > self.max_details {
            self.details.remove(0);
        }
    }
    
    /// Set whether to show elapsed time
    pub fn show_elapsed_time(mut self, show: bool) -> Self {
        self.show_elapsed_time = show;
        self
    }
    
    /// Set maximum number of detail lines
    pub fn max_details(mut self, max: usize) -> Self {
        self.max_details = max;
        self
    }
    
    /// Create a progress dialog for agent operations
    pub fn agent_operation(operation: &str, agent_name: &str) -> Self {
        Self::new(
            format!("Agent: {}", agent_name),
            format!("{}...", operation),
        )
        .show_elapsed_time(true)
    }
    
    /// Create a progress dialog for task operations
    pub fn task_operation(operation: &str, task_title: &str) -> Self {
        Self::new(
            format!("Task: {}", task_title),
            format!("{}...", operation),
        )
        .show_elapsed_time(true)
    }
    
    /// Create a progress dialog for file operations
    pub fn file_operation(operation: &str, filename: &str) -> Self {
        Self::new(
            format!("File: {}", filename),
            format!("{}...", operation),
        )
    }
    
    /// Update the spinner animation
    pub fn update_spinner(&mut self) {
        if self.progress.is_none() {
            self.spinner_frame = (self.spinner_frame + 1) % 4;
        }
    }
    
    /// Get the current spinner character
    fn get_spinner_char(&self) -> char {
        match self.spinner_frame {
            0 => '◐',
            1 => '◓',
            2 => '◑',
            3 => '◒',
            _ => '◐',
        }
    }
    
    /// Get elapsed time as a formatted string
    fn get_elapsed_time(&self) -> String {
        let elapsed = self.start_time.elapsed();
        format!("{:.1}s", elapsed.as_secs_f64())
    }
    
    /// Get the appropriate title based on state
    fn get_display_title(&self) -> String {
        if self.is_complete {
            if self.is_failed {
                format!("❌ {}", self.title)
            } else {
                format!("✅ {}", self.title)
            }
        } else {
            self.title.clone()
        }
    }
}

impl Dialog for ProgressDialog {
    fn render(&self, frame: &mut Frame, area: Rect) {
        let border_style = if self.is_complete {
            if self.is_failed {
                Style::default().fg(Color::Red)
            } else {
                Style::default().fg(Color::Green)
            }
        } else {
            Style::default().fg(Color::Cyan)
        };
        
        render_utils::render_dialog_overlay(frame, area, &self.get_display_title(), |frame, inner_area| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),  // Message and spinner/progress
                    Constraint::Length(3),  // Progress bar (if determinate)
                    Constraint::Min(0),     // Details
                    Constraint::Length(2),  // Status and time
                    Constraint::Length(2),  // Instructions
                ])
                .split(inner_area);
            
            // Render message with spinner or progress
            let message_text = if self.progress.is_some() {
                format!("{}", self.message)
            } else {
                format!("{} {}", self.get_spinner_char(), self.message)
            };
            
            let message_style = if self.is_complete {
                if self.is_failed {
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
                }
            } else {
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
            };
            
            render_utils::render_centered_text(frame, chunks[0], &message_text, message_style);
            
            // Render progress bar if determinate
            if let Some(progress) = self.progress {
                let progress_bar = Gauge::default()
                    .block(Block::default().borders(Borders::ALL))
                    .gauge_style(if self.is_complete {
                        if self.is_failed {
                            Style::default().fg(Color::Red)
                        } else {
                            Style::default().fg(Color::Green)
                        }
                    } else {
                        Style::default().fg(Color::Blue)
                    })
                    .percent(progress as u16)
                    .label(format!("{}%", progress));
                
                frame.render_widget(progress_bar, chunks[1]);
            } else {
                // Render indeterminate progress indicator
                let progress_text = "Working...";
                render_utils::render_centered_text(
                    frame,
                    chunks[1],
                    progress_text,
                    Style::default().fg(Color::Gray),
                );
            }
            
            // Render details if any
            if !self.details.is_empty() && chunks[2].height > 0 {
                let details_text = self.details.join("\n");
                let details_paragraph = Paragraph::new(details_text)
                    .style(Style::default().fg(Color::Gray))
                    .block(Block::default().borders(Borders::ALL).title("Details"))
                    .wrap(ratatui::widgets::Wrap { trim: true });
                
                frame.render_widget(details_paragraph, chunks[2]);
            }
            
            // Render status and elapsed time
            let status_text = if self.is_complete {
                if self.is_failed {
                    if let Some(error) = &self.error_message {
                        format!("Failed: {}", error)
                    } else {
                        "Failed".to_string()
                    }
                } else {
                    if let Some(success) = &self.success_message {
                        format!("Success: {}", success)
                    } else {
                        "Completed successfully".to_string()
                    }
                }
            } else {
                if self.show_elapsed_time {
                    format!("Elapsed: {}", self.get_elapsed_time())
                } else {
                    "In progress...".to_string()
                }
            };
            
            let status_style = if self.is_complete {
                if self.is_failed {
                    Style::default().fg(Color::Red)
                } else {
                    Style::default().fg(Color::Green)
                }
            } else {
                Style::default().fg(Color::Gray)
            };
            
            render_utils::render_centered_text(frame, chunks[3], &status_text, status_style);
            
            // Render instructions
            let instructions_text = if self.is_complete {
                "Press any key to close"
            } else if self.can_cancel {
                "Press Esc to cancel"
            } else {
                "Please wait..."
            };
            
            render_utils::render_centered_text(
                frame,
                chunks[4],
                instructions_text,
                Style::default().fg(Color::Gray),
            );
        }, Some(border_style));
    }
    
    fn handle_input(&mut self, key: KeyCode) -> DialogResult {
        match key {
            KeyCode::Esc => {
                if self.is_complete {
                    DialogResult::Close
                } else if self.can_cancel {
                    DialogResult::Cancel
                } else {
                    DialogResult::Continue
                }
            }
            _ => {
                if self.is_complete {
                    DialogResult::Close
                } else {
                    DialogResult::Continue
                }
            }
        }
    }
    
    fn title(&self) -> &str {
        &self.title
    }
    
    fn can_cancel(&self) -> bool {
        self.can_cancel && !self.is_complete
    }
    
    fn can_confirm(&self) -> bool {
        false // Progress dialogs don't have confirm actions
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;
    
    #[test]
    fn test_progress_dialog_new() {
        let dialog = ProgressDialog::new("Test Operation", "Processing...");
        
        assert_eq!(dialog.title, "Test Operation");
        assert_eq!(dialog.message, "Processing...");
        assert!(dialog.progress.is_none());
        assert!(dialog.can_cancel);
        assert!(!dialog.is_complete);
        assert!(!dialog.is_failed);
        assert!(dialog.error_message.is_none());
        assert!(dialog.success_message.is_none());
        assert!(!dialog.show_elapsed_time);
        assert!(dialog.details.is_empty());
    }
    
    #[test]
    fn test_progress_dialog_set_progress() {
        let mut dialog = ProgressDialog::new("Test", "Message");
        
        dialog.set_progress(50);
        assert_eq!(dialog.progress, Some(50));
        
        dialog.set_progress(150); // Should be clamped to 100
        assert_eq!(dialog.progress, Some(100));
        
        dialog.set_indeterminate();
        assert!(dialog.progress.is_none());
    }
    
    #[test]
    fn test_progress_dialog_completion() {
        let mut dialog = ProgressDialog::new("Test", "Message");
        
        // Test successful completion
        dialog.set_complete("Operation completed successfully");
        assert!(dialog.is_complete);
        assert!(!dialog.is_failed);
        assert_eq!(dialog.success_message, Some("Operation completed successfully".to_string()));
        assert_eq!(dialog.progress, Some(100));
        
        // Test failure
        let mut dialog = ProgressDialog::new("Test", "Message");
        dialog.set_failed("Operation failed with error");
        assert!(dialog.is_complete);
        assert!(dialog.is_failed);
        assert_eq!(dialog.error_message, Some("Operation failed with error".to_string()));
    }
    
    #[test]
    fn test_progress_dialog_details() {
        let mut dialog = ProgressDialog::new("Test", "Message");
        
        dialog.add_detail("Step 1 completed");
        dialog.add_detail("Step 2 completed");
        assert_eq!(dialog.details.len(), 2);
        assert_eq!(dialog.details[0], "Step 1 completed");
        assert_eq!(dialog.details[1], "Step 2 completed");
        
        // Test max details limit
        dialog.max_details = 2;
        dialog.add_detail("Step 3 completed");
        assert_eq!(dialog.details.len(), 2);
        assert_eq!(dialog.details[0], "Step 2 completed");
        assert_eq!(dialog.details[1], "Step 3 completed");
    }
    
    #[test]
    fn test_progress_dialog_spinner() {
        let mut dialog = ProgressDialog::new("Test", "Message");
        
        let initial_char = dialog.get_spinner_char();
        dialog.update_spinner();
        let updated_char = dialog.get_spinner_char();
        
        // Should be different after update
        assert_ne!(initial_char, updated_char);
        
        // Test spinner cycling
        for _ in 0..4 {
            dialog.update_spinner();
        }
        assert_eq!(dialog.spinner_frame, 0); // Should cycle back to 0
    }
    
    #[test]
    fn test_progress_dialog_elapsed_time() {
        let dialog = ProgressDialog::new("Test", "Message");
        
        // Should return a formatted time string
        let elapsed = dialog.get_elapsed_time();
        assert!(elapsed.ends_with("s"));
        assert!(elapsed.contains("."));
    }
    
    #[test]
    fn test_progress_dialog_factory_methods() {
        let dialog = ProgressDialog::agent_operation("Starting", "Test Agent");
        assert_eq!(dialog.title, "Agent: Test Agent");
        assert_eq!(dialog.message, "Starting...");
        assert!(dialog.show_elapsed_time);
        
        let dialog = ProgressDialog::task_operation("Executing", "Test Task");
        assert_eq!(dialog.title, "Task: Test Task");
        assert_eq!(dialog.message, "Executing...");
        assert!(dialog.show_elapsed_time);
        
        let dialog = ProgressDialog::file_operation("Loading", "test.txt");
        assert_eq!(dialog.title, "File: test.txt");
        assert_eq!(dialog.message, "Loading...");
        assert!(!dialog.show_elapsed_time);
    }
    
    #[test]
    fn test_progress_dialog_display_title() {
        let mut dialog = ProgressDialog::new("Test Operation", "Message");
        
        // Normal state
        assert_eq!(dialog.get_display_title(), "Test Operation");
        
        // Completed successfully
        dialog.set_complete("Done");
        assert_eq!(dialog.get_display_title(), "✅ Test Operation");
        
        // Failed
        dialog.set_failed("Error");
        assert_eq!(dialog.get_display_title(), "❌ Test Operation");
    }
    
    #[test]
    fn test_progress_dialog_input_handling() {
        let mut dialog = ProgressDialog::new("Test", "Message");
        
        // Test cancellation when allowed
        let result = dialog.handle_input(KeyCode::Esc);
        assert_eq!(result, DialogResult::Cancel);
        
        // Test when cancellation is not allowed
        dialog.set_cancellable(false);
        let result = dialog.handle_input(KeyCode::Esc);
        assert_eq!(result, DialogResult::Continue);
        
        // Test other keys during progress
        let result = dialog.handle_input(KeyCode::Char('a'));
        assert_eq!(result, DialogResult::Continue);
        
        // Test completion
        dialog.set_complete("Done");
        let result = dialog.handle_input(KeyCode::Esc);
        assert_eq!(result, DialogResult::Close);
        
        let result = dialog.handle_input(KeyCode::Char('a'));
        assert_eq!(result, DialogResult::Close);
    }
    
    #[test]
    fn test_dialog_trait_implementation() {
        let dialog = ProgressDialog::new("Test", "Message");
        
        assert_eq!(dialog.title(), "Test");
        assert!(dialog.can_cancel());
        assert!(!dialog.can_confirm());
        
        // Test when completed
        let mut dialog = ProgressDialog::new("Test", "Message");
        dialog.set_complete("Done");
        assert!(!dialog.can_cancel());
        assert!(!dialog.can_confirm());
    }
    
    #[test]
    fn test_progress_dialog_builder_pattern() {
        let dialog = ProgressDialog::new("Test", "Message")
            .show_elapsed_time(true)
            .max_details(5);
        
        assert!(dialog.show_elapsed_time);
        assert_eq!(dialog.max_details, 5);
    }
    
    #[test]
    fn test_progress_dialog_message_update() {
        let mut dialog = ProgressDialog::new("Test", "Initial message");
        
        dialog.set_message("Updated message");
        assert_eq!(dialog.message, "Updated message");
        
        dialog.set_cancellable(false);
        assert!(!dialog.can_cancel);
        
        dialog.set_cancellable(true);
        assert!(dialog.can_cancel);
    }
}