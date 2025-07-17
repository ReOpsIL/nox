use super::{Form, FormResult, FormFieldNavigation, FormValidator, FormError, handle_form_navigation, render_utils};
use crate::tui::components::inputs::{TextInput, TextArea, InputResult};
use crate::types::{Agent, AgentStatus, ResourceLimits};
use crossterm::event::KeyCode;
use ratatui::prelude::*;
use ratatui::widgets::Paragraph;
use chrono::Utc;

/// Form for creating a new agent
#[derive(Debug, Clone)]
pub struct CreateAgentForm {
    /// Agent name input
    pub name: TextInput,
    /// System prompt text area
    pub system_prompt: TextArea,
    /// Maximum memory limit input
    pub max_memory_mb: TextInput,
    /// Maximum CPU percentage input
    pub max_cpu_percent: TextInput,
    /// Currently focused field index
    pub current_field: usize,
    /// Validation errors
    pub validation_errors: Vec<String>,
}

impl CreateAgentForm {
    /// Create a new CreateAgentForm
    pub fn new() -> Self {
        Self {
            name: TextInput::new("Agent Name")
                .placeholder("Enter agent name")
                .max_length(100)
                .required(),
            system_prompt: TextArea::new("System Prompt")
                .required(),
            max_memory_mb: TextInput::new("Max Memory (MB)")
                .placeholder("512")
                .max_length(10)
                .required(),
            max_cpu_percent: TextInput::new("Max CPU (%)")
                .placeholder("50")
                .max_length(3)
                .required(),
            current_field: 0,
            validation_errors: Vec::new(),
        }
    }
    
    /// Pre-populate the form with existing agent data (for editing)
    pub fn from_agent(agent: &Agent) -> Self {
        let mut form = Self::new();
        form.name.set_value(agent.name.clone());
        form.system_prompt.set_value(agent.system_prompt.clone());
        form.max_memory_mb.set_value(agent.resource_limits.max_memory_mb.to_string());
        form.max_cpu_percent.set_value(agent.resource_limits.max_cpu_percent.to_string());
        form.update_focus();
        form
    }
    
    /// Handle input for the currently focused field
    fn handle_field_input(&mut self, key: KeyCode) -> FormResult {
        let input_result = match self.current_field {
            0 => self.name.handle_input(key),
            1 => self.system_prompt.handle_input(key),
            2 => self.max_memory_mb.handle_input(key),
            3 => self.max_cpu_percent.handle_input(key),
            _ => InputResult::Continue,
        };

        FormResult::Continue
        
        // match input_result {
        //     InputResult::Continue => FormResult::Continue,
        //     InputResult::NextField => {
        //         self.next_field();
        //         self.update_focus();
        //         FormResult::Continue
        //     }
        //     InputResult::PreviousField => {
        //         self.previous_field();
        //         self.update_focus();
        //         FormResult::Continue
        //     }
        //     InputResult::Submit => {
        //         if self.is_complete() && self.is_valid() {
        //             FormResult::Submit
        //         } else {
        //             self.next_field();
        //             self.update_focus();
        //             FormResult::Continue
        //         }
        //     }
        //     InputResult::Cancel => FormResult::Cancel,
        // }
    }
    
    /// Validate resource limits
    fn validate_resource_limits(&self) -> Result<ResourceLimits, Vec<FormError>> {
        let mut errors = Vec::new();
        
        // Validate memory limit
        let max_memory_mb = if self.max_memory_mb.value().trim().is_empty() {
            512 // Default value
        } else {
            match self.max_memory_mb.value().parse::<usize>() {
                Ok(value) => {
                    if value < 128 {
                        errors.push(FormError::new("max_memory_mb", "Memory limit must be at least 128 MB"));
                        512
                    } else if value > 8192 {
                        errors.push(FormError::new("max_memory_mb", "Memory limit cannot exceed 8192 MB"));
                        8192
                    } else {
                        value
                    }
                }
                Err(_) => {
                    errors.push(FormError::new("max_memory_mb", "Memory limit must be a valid number"));
                    512
                }
            }
        };
        
        // Validate CPU limit
        let max_cpu_percent = if self.max_cpu_percent.value().trim().is_empty() {
            50 // Default value
        } else {
            match self.max_cpu_percent.value().parse::<u8>() {
                Ok(value) => {
                    if value > 100 {
                        errors.push(FormError::new("max_cpu_percent", "CPU limit cannot exceed 100%"));
                        100
                    } else if value < 1 {
                        errors.push(FormError::new("max_cpu_percent", "CPU limit must be at least 1%"));
                        1
                    } else {
                        value
                    }
                }
                Err(_) => {
                    errors.push(FormError::new("max_cpu_percent", "CPU limit must be a valid number"));
                    50
                }
            }
        };
        
        if errors.is_empty() {
            Ok(ResourceLimits {
                max_memory_mb,
                max_cpu_percent,
            })
        } else {
            Err(errors)
        }
    }
}

impl Default for CreateAgentForm {
    fn default() -> Self {
        Self::new()
    }
}

impl FormFieldNavigation for CreateAgentForm {
    fn field_count(&self) -> usize {
        4
    }
    
    fn current_field(&self) -> usize {
        self.current_field
    }
    
    fn set_current_field(&mut self, index: usize) {
        println!("{}",index);
        self.current_field = index.min(self.field_count().saturating_sub(1));
    }
    
    fn update_focus(&mut self) {
        self.name.set_focused(self.current_field == 0);
        self.system_prompt.set_focused(self.current_field == 1);
        self.max_memory_mb.set_focused(self.current_field == 2);
        self.max_cpu_percent.set_focused(self.current_field == 3);
    }
}

impl Form for CreateAgentForm {
    type Output = Agent;
    
    fn render(&self, frame: &mut Frame, area: Rect) {
        render_utils::render_form_overlay(frame, area, "Create Agent", |frame, inner_area| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(5),  // Name field
                    Constraint::Min(8),     // System prompt field
                    Constraint::Length(5),  // Memory limit field
                    Constraint::Length(5),  // CPU limit field
                    Constraint::Length(3),  // Validation errors
                    Constraint::Length(2),  // Help text
                ])
                .split(inner_area);
            
            // Render form fields
            self.name.render(frame, chunks[0]);
            self.system_prompt.render(frame, chunks[1]);
            self.max_memory_mb.render(frame, chunks[2]);
            self.max_cpu_percent.render(frame, chunks[3]);
            
            // Render validation errors
            if !self.validation_errors.is_empty() {
                render_utils::render_validation_errors(frame, chunks[4], &self.validation_errors);
            }
            
            // Render help text
            let help_text = match self.current_field {
                0 => "Enter a descriptive name for the agent",
                1 => "Enter the system prompt that defines the agent's behavior",
                2 => "Memory limit in MB (default: 512, min: 128, max: 8192)",
                3 => "CPU usage limit as percentage (default: 50, max: 100)",
                _ => "Use Tab/Shift+Tab to navigate, Enter to submit, Esc to cancel",
            };
            render_utils::render_help_text(frame, chunks[5], help_text);
        });
    }
    
    fn handle_input(&mut self, key: KeyCode) -> FormResult {
        // Handle field-specific input first
        let field_result = self.handle_field_input(key);
        if field_result != FormResult::Continue {
            return field_result;
        }
        
        // Handle general form navigation
        handle_form_navigation(key, self, self.is_complete() && self.is_valid())
    }
    
    fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        
        // Validate name
        if let Err(error) = FormValidator::required("Agent Name", self.name.value()) {
            errors.push(error.message);
        } else if let Err(error) = FormValidator::min_length("Agent Name", self.name.value(), 2) {
            errors.push(error.message);
        } else if let Err(error) = FormValidator::max_length("Agent Name", self.name.value(), 100) {
            errors.push(error.message);
        }
        
        // Validate system prompt
        if let Err(error) = FormValidator::required("System Prompt", &self.system_prompt.value()) {
            errors.push(error.message);
        } else if let Err(error) = FormValidator::min_length("System Prompt", &self.system_prompt.value(), 10) {
            errors.push(error.message);
        }
        
        // Validate resource limits
        if let Err(resource_errors) = self.validate_resource_limits() {
            for error in resource_errors {
                errors.push(error.message);
            }
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
    
    fn submit(self) -> Result<Self::Output, String> {
        // Validate before submission
        if let Err(errors) = self.validate() {
            return Err(errors.join("; "));
        }
        
        // Parse resource limits
        let resource_limits = self.validate_resource_limits()
            .map_err(|errors| errors.into_iter().map(|e| e.message).collect::<Vec<_>>().join("; "))?;
        
        // Generate unique ID for the agent
        let id = format!("agent_{}", Utc::now().timestamp());
        
        Ok(Agent {
            id,
            name: self.name.value().to_string(),
            system_prompt: self.system_prompt.value(),
            status: AgentStatus::Inactive,
            created_at: Utc::now(),
            resource_limits,
        })
    }
    
    fn is_complete(&self) -> bool {
        !self.name.value().trim().is_empty() &&
        !self.system_prompt.value().trim().is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Agent, AgentStatus, ResourceLimits};
    use chrono::Utc;
    
    fn create_test_agent() -> Agent {
        Agent {
            id: "test_agent".to_string(),
            name: "Test Agent".to_string(),
            system_prompt: "Test system prompt".to_string(),
            status: AgentStatus::Inactive,
            created_at: Utc::now(),
            resource_limits: ResourceLimits {
                max_memory_mb: 1024,
                max_cpu_percent: 75,
            },
        }
    }
    
    #[test]
    fn test_create_agent_form_new() {
        let form = CreateAgentForm::new();
        assert_eq!(form.name.value(), "");
        assert_eq!(form.system_prompt.value(), "");
        assert_eq!(form.max_memory_mb.value(), "");
        assert_eq!(form.max_cpu_percent.value(), "");
        assert_eq!(form.current_field, 0);
        assert!(form.validation_errors.is_empty());
    }
    
    #[test]
    fn test_create_agent_form_from_agent() {
        let agent = create_test_agent();
        let form = CreateAgentForm::from_agent(&agent);
        
        assert_eq!(form.name.value(), "Test Agent");
        assert_eq!(form.system_prompt.value(), "Test system prompt");
        assert_eq!(form.max_memory_mb.value(), "1024");
        assert_eq!(form.max_cpu_percent.value(), "75");
    }
    
    #[test]
    fn test_form_field_navigation() {
        let mut form = CreateAgentForm::new();
        
        // Test initial state
        assert_eq!(form.current_field(), 0);
        assert_eq!(form.field_count(), 4);
        
        // Test next field
        form.next_field();
        assert_eq!(form.current_field(), 1);
        
        // Test previous field
        form.previous_field();
        assert_eq!(form.current_field(), 0);
        
        // Test wrapping
        form.set_current_field(3);
        form.next_field();
        assert_eq!(form.current_field(), 0);
        
        // Test previous field wrapping
        form.set_current_field(0);
        form.previous_field();
        assert_eq!(form.current_field(), 3);
    }
    
    #[test]
    fn test_form_focus_management() {
        let mut form = CreateAgentForm::new();
        
        // Test initial focus
        form.update_focus();
        assert!(form.name.focused);
        assert!(!form.system_prompt.focused);
        assert!(!form.max_memory_mb.focused);
        assert!(!form.max_cpu_percent.focused);
        
        // Test focus change
        form.set_current_field(1);
        form.update_focus();
        assert!(!form.name.focused);
        assert!(form.system_prompt.focused);
        assert!(!form.max_memory_mb.focused);
        assert!(!form.max_cpu_percent.focused);
    }
    
    #[test]
    fn test_form_validation_success() {
        let mut form = CreateAgentForm::new();
        form.name.set_value("Test Agent".to_string());
        form.system_prompt.set_value("This is a test system prompt".to_string());
        form.max_memory_mb.set_value("512".to_string());
        form.max_cpu_percent.set_value("50".to_string());
        
        assert!(form.validate().is_ok());
        assert!(form.is_complete());
        assert!(form.is_valid());
    }
    
    #[test]
    fn test_form_validation_required_fields() {
        let form = CreateAgentForm::new();
        
        let result = form.validate();
        assert!(result.is_err());
        
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.contains("Agent Name is required")));
        assert!(errors.iter().any(|e| e.contains("System Prompt is required")));
    }
    
    #[test]
    fn test_form_validation_field_lengths() {
        let mut form = CreateAgentForm::new();
        
        // Test name too short
        form.name.set_value("A".to_string());
        form.system_prompt.set_value("Short".to_string());
        
        let result = form.validate();
        assert!(result.is_err());
        
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.contains("Agent Name must be at least 2 characters")));
        assert!(errors.iter().any(|e| e.contains("System Prompt must be at least 10 characters")));
    }
    
    #[test]
    fn test_resource_limits_validation() {
        let mut form = CreateAgentForm::new();
        form.name.set_value("Test Agent".to_string());
        form.system_prompt.set_value("This is a test system prompt".to_string());
        
        // Test invalid memory limit
        form.max_memory_mb.set_value("invalid".to_string());
        form.max_cpu_percent.set_value("invalid".to_string());
        
        let result = form.validate();
        assert!(result.is_err());
        
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.contains("Memory limit must be a valid number")));
        assert!(errors.iter().any(|e| e.contains("CPU limit must be a valid number")));
    }
    
    #[test]
    fn test_resource_limits_bounds() {
        let mut form = CreateAgentForm::new();
        form.name.set_value("Test Agent".to_string());
        form.system_prompt.set_value("This is a test system prompt".to_string());
        
        // Test memory limit too low
        form.max_memory_mb.set_value("64".to_string());
        form.max_cpu_percent.set_value("101".to_string());
        
        let result = form.validate();
        assert!(result.is_err());
        
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.contains("Memory limit must be at least 128 MB")));
        assert!(errors.iter().any(|e| e.contains("CPU limit cannot exceed 100%")));
    }
    
    #[test]
    fn test_form_submission_success() {
        let mut form = CreateAgentForm::new();
        form.name.set_value("Test Agent".to_string());
        form.system_prompt.set_value("This is a test system prompt".to_string());
        form.max_memory_mb.set_value("1024".to_string());
        form.max_cpu_percent.set_value("75".to_string());
        
        let result = form.submit();
        assert!(result.is_ok());
        
        let agent = result.unwrap();
        assert_eq!(agent.name, "Test Agent");
        assert_eq!(agent.system_prompt, "This is a test system prompt");
        assert_eq!(agent.resource_limits.max_memory_mb, 1024);
        assert_eq!(agent.resource_limits.max_cpu_percent, 75);
        assert_eq!(agent.status, AgentStatus::Inactive);
    }
    
    #[test]
    fn test_form_submission_with_defaults() {
        let mut form = CreateAgentForm::new();
        form.name.set_value("Test Agent".to_string());
        form.system_prompt.set_value("This is a test system prompt".to_string());
        // Leave resource limits empty to test defaults
        
        let result = form.submit();
        assert!(result.is_ok());
        
        let agent = result.unwrap();
        assert_eq!(agent.resource_limits.max_memory_mb, 512);
        assert_eq!(agent.resource_limits.max_cpu_percent, 50);
    }
    
    #[test]
    fn test_form_submission_failure() {
        let form = CreateAgentForm::new();
        // Don't set any values
        
        let result = form.submit();
        assert!(result.is_err());
        
        let error = result.unwrap_err();
        assert!(error.contains("Agent Name is required"));
        assert!(error.contains("System Prompt is required"));
    }
    
    #[test]
    fn test_form_is_complete() {
        let mut form = CreateAgentForm::new();
        assert!(!form.is_complete());
        
        form.name.set_value("Test Agent".to_string());
        assert!(!form.is_complete());
        
        form.system_prompt.set_value("Test prompt".to_string());
        assert!(form.is_complete());
    }
    
    #[test]
    fn test_form_handle_input_navigation() {
        let mut form = CreateAgentForm::new();
        form.update_focus();
        
        // Test tab navigation
        let result = form.handle_input(KeyCode::Tab);
        assert_eq!(result, FormResult::Continue);
        assert_eq!(form.current_field, 1);
        
        // Test backtab navigation
        let result = form.handle_input(KeyCode::BackTab);
        assert_eq!(result, FormResult::Continue);
        assert_eq!(form.current_field, 0);
        
        // Test escape
        let result = form.handle_input(KeyCode::Esc);
        assert_eq!(result, FormResult::Cancel);
    }
    
    #[test]
    fn test_form_handle_input_character() {
        let mut form = CreateAgentForm::new();
        form.update_focus();
        
        // Test character input to name field
        let result = form.handle_input(KeyCode::Char('T'));
        assert_eq!(result, FormResult::Continue);
        assert_eq!(form.name.value(), "T");
        
        // Test character input continues
        let result = form.handle_input(KeyCode::Char('e'));
        assert_eq!(result, FormResult::Continue);
        assert_eq!(form.name.value(), "Te");
    }
}