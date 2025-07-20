pub mod create_agent;
pub mod create_task;

use crossterm::event::KeyCode;
use ratatui::prelude::*;

/// Result of form interactions
#[derive(Debug, Clone, PartialEq)]
pub enum FormResult {
    /// Continue processing input
    Continue,
    /// Submit the form
    Submit,
    /// Cancel the form operation
    Cancel,
}

/// Base trait for all form components
pub trait Form {
    type Output;
    
    /// Render the form
    fn render(&self, frame: &mut Frame, area: Rect);
    
    /// Handle keyboard input
    fn handle_input(&mut self, key: KeyCode) -> FormResult;
    
    /// Validate the form
    fn validate(&self) -> Result<(), Vec<String>>;
    
    /// Submit the form and return the result
    fn submit(self) -> Result<Self::Output, String>;
    
    /// Check if the form is ready for submission
    fn is_complete(&self) -> bool;
    
    /// Check if the form is valid
    fn is_valid(&self) -> bool {
        self.validate().is_ok()
    }
}

/// Base form field navigation
pub trait FormFieldNavigation {
    /// Get the number of fields in the form
    fn field_count(&self) -> usize;
    
    /// Get the current field index
    fn current_field(&self) -> usize;
    
    /// Set the current field index
    fn set_current_field(&mut self, index: usize);
    
    /// Move to the next field
    fn next_field(&mut self) {
        let current = self.current_field();
        let count = self.field_count();
        if count > 0 {
            self.set_current_field((current + 1) % count);
        }
    }
    
    /// Move to the previous field
    fn previous_field(&mut self) {
        let current = self.current_field();
        let count = self.field_count();
        if count > 0 {
            let new_index = if current == 0 {
                count - 1
            } else {
                current - 1
            };
            self.set_current_field(new_index);
        }
    }
    
    /// Update focus states based on current field
    fn update_focus(&mut self);
}

/// Form validation error
#[derive(Debug, Clone)]
pub struct FormError {
    pub field: String,
    pub message: String,
}

impl FormError {
    pub fn new(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            message: message.into(),
        }
    }
}

/// Form validation result
pub type FormValidationResult = Result<(), Vec<FormError>>;

/// Common form validation utilities
pub struct FormValidator;

impl FormValidator {
    /// Validate required field
    pub fn required(field_name: &str, value: &str) -> Result<(), FormError> {
        if value.trim().is_empty() {
            Err(FormError::new(field_name, format!("{} is required", field_name)))
        } else {
            Ok(())
        }
    }
    
    /// Validate minimum length
    pub fn min_length(field_name: &str, value: &str, min_len: usize) -> Result<(), FormError> {
        if value.len() < min_len {
            Err(FormError::new(field_name, format!("{} must be at least {} characters", field_name, min_len)))
        } else {
            Ok(())
        }
    }
    
    /// Validate maximum length
    pub fn max_length(field_name: &str, value: &str, max_len: usize) -> Result<(), FormError> {
        if value.len() > max_len {
            Err(FormError::new(field_name, format!("{} must be {} characters or less", field_name, max_len)))
        } else {
            Ok(())
        }
    }
    
    /// Validate that value is not empty after trimming
    pub fn not_empty(field_name: &str, value: &str) -> Result<(), FormError> {
        if value.trim().is_empty() {
            Err(FormError::new(field_name, format!("{} cannot be empty", field_name)))
        } else {
            Ok(())
        }
    }
}

/// Helper function to handle common form navigation
pub fn handle_form_navigation(
    key: KeyCode,
    form: &mut impl FormFieldNavigation,
    allow_submit: bool,
) -> FormResult {
    match key {
        KeyCode::Tab => {
            form.next_field();
            form.update_focus();
            FormResult::Continue
        }
        KeyCode::BackTab => {
            form.previous_field();
            form.update_focus();
            FormResult::Continue
        }
        KeyCode::Enter => {
            if allow_submit {
                FormResult::Submit
            } else {
                form.next_field();
                form.update_focus();
                FormResult::Continue
            }
        }
        KeyCode::Esc => FormResult::Cancel,
        _ => FormResult::Continue,
    }
}

/// Common form rendering utilities
pub mod render_utils {
    use ratatui::prelude::*;
    use ratatui::widgets::{Block, Borders, Paragraph, Clear};
    
    /// Create a centered overlay area
    pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
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
    
    /// Render a form overlay with title and content
    pub fn render_form_overlay(
        frame: &mut Frame,
        area: Rect,
        title: &str,
        content_render: impl FnOnce(&mut Frame, Rect),
    ) {
        let overlay_area = centered_rect(80, 70, area);
        
        // Clear the background
        frame.render_widget(Clear, overlay_area);
        
        // Create the main form block
        let block = Block::default()
            .borders(Borders::ALL)
            .title(title)
            .border_style(Style::default().fg(Color::Yellow));
        
        // Render the block
        frame.render_widget(block, overlay_area);
        
        // Create inner area for content
        let inner_area = Rect {
            x: overlay_area.x + 1,
            y: overlay_area.y + 1,
            width: overlay_area.width.saturating_sub(2),
            height: overlay_area.height.saturating_sub(2),
        };
        
        // Render the content
        content_render(frame, inner_area);
    }
    
    /// Render validation errors at the bottom of a form
    pub fn render_validation_errors(frame: &mut Frame, area: Rect, errors: &[String]) {
        if errors.is_empty() {
            return;
        }
        
        let error_text = errors.join("\n");
        let paragraph = Paragraph::new(error_text)
            .style(Style::default().fg(Color::Red))
            .wrap(ratatui::widgets::Wrap { trim: true });
        
        frame.render_widget(paragraph, area);
    }
    
    /// Render form help text
    pub fn render_help_text(frame: &mut Frame, area: Rect, text: &str) {
        let paragraph = Paragraph::new(text)
            .style(Style::default().fg(Color::Gray))
            .wrap(ratatui::widgets::Wrap { trim: true });
        
        frame.render_widget(paragraph, area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_form_error_creation() {
        let error = FormError::new("field_name", "error message");
        assert_eq!(error.field, "field_name");
        assert_eq!(error.message, "error message");
    }
    
    #[test]
    fn test_form_validator_required() {
        // Should pass for non-empty string
        assert!(FormValidator::required("name", "John").is_ok());
        
        // Should fail for empty string
        assert!(FormValidator::required("name", "").is_err());
        
        // Should fail for whitespace-only string
        assert!(FormValidator::required("name", "   ").is_err());
    }
    
    #[test]
    fn test_form_validator_min_length() {
        // Should pass for string longer than minimum
        assert!(FormValidator::min_length("name", "John", 3).is_ok());
        
        // Should pass for string equal to minimum
        assert!(FormValidator::min_length("name", "Jon", 3).is_ok());
        
        // Should fail for string shorter than minimum
        assert!(FormValidator::min_length("name", "Jo", 3).is_err());
    }
    
    #[test]
    fn test_form_validator_max_length() {
        // Should pass for string shorter than maximum
        assert!(FormValidator::max_length("name", "John", 10).is_ok());
        
        // Should pass for string equal to maximum
        assert!(FormValidator::max_length("name", "John", 4).is_ok());
        
        // Should fail for string longer than maximum
        assert!(FormValidator::max_length("name", "John", 3).is_err());
    }
    
    #[test]
    fn test_form_validator_not_empty() {
        // Should pass for non-empty string
        assert!(FormValidator::not_empty("name", "John").is_ok());
        
        // Should fail for empty string
        assert!(FormValidator::not_empty("name", "").is_err());
        
        // Should fail for whitespace-only string
        assert!(FormValidator::not_empty("name", "   ").is_err());
    }
    
    #[test]
    fn test_form_result_equality() {
        assert_eq!(FormResult::Continue, FormResult::Continue);
        assert_eq!(FormResult::Submit, FormResult::Submit);
        assert_eq!(FormResult::Cancel, FormResult::Cancel);
        
        assert_ne!(FormResult::Continue, FormResult::Submit);
        assert_ne!(FormResult::Submit, FormResult::Cancel);
    }
}