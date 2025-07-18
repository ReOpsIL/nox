use super::{Form, FormResult, FormFieldNavigation, FormValidator, handle_form_navigation, render_utils};
use crate::tui::components::inputs::{TextInput, TextArea, AgentSelector, PrioritySelector, InputResult};
use crate::types::{Agent, Task, TaskStatus};
use std::collections::HashMap;
use crossterm::event::KeyCode;
use ratatui::prelude::*;
use chrono::Utc;

/// Form for creating a new task
#[derive(Debug, Clone)]
pub struct CreateTaskForm {
    /// Task title input
    pub title: TextInput,
    /// Task description text area
    pub description: TextArea,
    /// Agent selector
    pub agent_selector: AgentSelector,
    /// Priority selector
    pub priority_selector: PrioritySelector,
    /// Currently focused field index
    pub current_field: usize,
    /// Validation errors
    pub validation_errors: Vec<String>,
    /// Existing task ID (for updates, None for new tasks)
    pub existing_task_id: Option<String>,
}

impl CreateTaskForm {
    /// Create a new CreateTaskForm
    pub fn new() -> Self {
        Self {
            title: TextInput::new("Task Title")
                .placeholder("Enter task title")
                .max_length(200)
                .required(),
            description: TextArea::new("Task Description")
                .max_lines(10)
                .required(),
            agent_selector: AgentSelector::new("Select Agent")
                .required(),
            priority_selector: PrioritySelector::new("Priority"),
            current_field: 0,
            validation_errors: Vec::new(),
            existing_task_id: None,
        }
    }
    
    /// Set available agents for selection
    pub fn set_agents(&mut self, agents: Vec<Agent>) {
        self.agent_selector.set_agents(agents);
    }
    
    /// Pre-populate the form with existing task data (for editing)
    pub fn from_task(task: &Task, agents: Vec<Agent>) -> Self {
        let mut form = Self::new();
        form.title.set_value(task.title.clone());
        form.description.set_value(task.description.clone());
        form.agent_selector.set_agents(agents);
        form.agent_selector.set_selected_agent(&task.agent_id);
        form.priority_selector.set_selected_priority(task.priority.clone());
        form.existing_task_id = Some(task.id.clone());
        form.update_focus();
        form
    }
    
    /// Handle input for the currently focused field
    fn handle_field_input(&mut self, key: KeyCode) -> FormResult {
        let _input_result = match self.current_field {
            0 => self.title.handle_input(key),
            1 => self.description.handle_input(key),
            2 => self.agent_selector.handle_input(key),
            3 => self.priority_selector.handle_input(key),
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
}

impl Default for CreateTaskForm {
    fn default() -> Self {
        Self::new()
    }
}

impl FormFieldNavigation for CreateTaskForm {
    fn field_count(&self) -> usize {
        4
    }
    
    fn current_field(&self) -> usize {
        self.current_field
    }
    
    fn set_current_field(&mut self, index: usize) {
        self.current_field = index.min(self.field_count().saturating_sub(1));
    }
    
    fn update_focus(&mut self) {
        self.title.set_focused(self.current_field == 0);
        self.description.set_focused(self.current_field == 1);
        self.agent_selector.set_focused(self.current_field == 2);
        self.priority_selector.set_focused(self.current_field == 3);
    }
}

impl Form for CreateTaskForm {
    type Output = Task;
    
    fn render(&self, frame: &mut Frame, area: Rect) {
        render_utils::render_form_overlay(frame, area, "Create Task", |frame, inner_area| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(5),  // Title field
                    Constraint::Min(8),     // Description field
                    Constraint::Length(7),  // Agent selector field
                    Constraint::Length(7),  // Priority selector field
                    Constraint::Length(3),  // Validation errors
                    Constraint::Length(2),  // Help text
                ])
                .split(inner_area);
            
            // Render form fields
            self.title.render(frame, chunks[0]);
            self.description.render(frame, chunks[1]);
            self.agent_selector.render(frame, chunks[2]);
            self.priority_selector.render(frame, chunks[3]);
            
            // Render validation errors
            if !self.validation_errors.is_empty() {
                render_utils::render_validation_errors(frame, chunks[4], &self.validation_errors);
            }
            
            // Render help text
            let help_text = match self.current_field {
                0 => "Enter a descriptive title for the task",
                1 => "Enter detailed description of what the task should accomplish",
                2 => "Select the agent that will execute this task",
                3 => "Set the priority level for this task",
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
        
        // Validate title
        if let Err(error) = FormValidator::required("Task Title", self.title.value()) {
            errors.push(error.message);
        } else if let Err(error) = FormValidator::min_length("Task Title", self.title.value(), 3) {
            errors.push(error.message);
        } else if let Err(error) = FormValidator::max_length("Task Title", self.title.value(), 200) {
            errors.push(error.message);
        }
        
        // Validate description
        if let Err(error) = FormValidator::required("Task Description", &self.description.value()) {
            errors.push(error.message);
        } else if let Err(error) = FormValidator::min_length("Task Description", &self.description.value(), 10) {
            errors.push(error.message);
        }
        
        // Validate agent selection
        if self.agent_selector.selected_agent().is_none() {
            errors.push("Agent selection is required".to_string());
        }
        
        // Update validation errors
        if !errors.is_empty() {
            let mut form_mut = self.clone();
            form_mut.validation_errors = errors.clone();
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
        
        // Get selected agent
        let selected_agent = self.agent_selector.selected_agent()
            .ok_or("Agent selection is required")?;
        
        // Use existing ID if updating, otherwise generate a new one
        let id = self.existing_task_id.clone()
            .unwrap_or_else(|| format!("task_{}", Utc::now().timestamp()));
        
        Ok(Task {
            id,
            title: self.title.value().to_string(),
            description: self.description.value(),
            agent_id: selected_agent.id.clone(),
            priority: self.priority_selector.selected_priority().clone(),
            status: TaskStatus::Todo,
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
            progress: 0,
            metadata: HashMap::new(),
        })
    }
    
    fn is_complete(&self) -> bool {
        !self.title.value().trim().is_empty() &&
        !self.description.value().trim().is_empty() &&
        self.agent_selector.selected_agent().is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Agent, AgentStatus, Task, TaskStatus, TaskPriority, ResourceLimits};
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
    
    fn create_test_task() -> Task {
        Task {
            id: "test_task".to_string(),
            title: "Test Task".to_string(),
            description: "Test task description".to_string(),
            agent_id: "test_agent".to_string(),
            priority: TaskPriority::Medium,
            status: TaskStatus::Todo,
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
            progress: 0,
            metadata: HashMap::new(),
        }
    }
    
    #[test]
    fn test_create_task_form_new() {
        let form = CreateTaskForm::new();
        assert_eq!(form.title.value(), "");
        assert_eq!(form.description.value(), "");
        assert!(form.agent_selector.selected_agent().is_none());
        assert_eq!(*form.priority_selector.selected_priority(), TaskPriority::Medium);
        assert_eq!(form.current_field, 0);
        assert!(form.validation_errors.is_empty());
        assert!(form.existing_task_id.is_none());
    }
    
    #[test]
    fn test_create_task_form_from_task() {
        let task = create_test_task();
        let agents = vec![create_test_agent()];
        let form = CreateTaskForm::from_task(&task, agents);
        
        assert_eq!(form.title.value(), "Test Task");
        assert_eq!(form.description.value(), "Test task description");
        assert_eq!(form.agent_selector.selected_agent().unwrap().id, "test_agent");
        assert_eq!(*form.priority_selector.selected_priority(), TaskPriority::Medium);
        assert_eq!(form.existing_task_id, Some("test_task".to_string()));
    }
    
    #[test]
    fn test_form_field_navigation() {
        let mut form = CreateTaskForm::new();
        
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
        let mut form = CreateTaskForm::new();
        
        // Test initial focus
        form.update_focus();
        assert!(form.title.focused);
        assert!(!form.description.focused);
        assert!(!form.agent_selector.focused);
        assert!(!form.priority_selector.focused);
        
        // Test focus change
        form.set_current_field(1);
        form.update_focus();
        assert!(!form.title.focused);
        assert!(form.description.focused);
        assert!(!form.agent_selector.focused);
        assert!(!form.priority_selector.focused);
        
        // Test agent selector focus
        form.set_current_field(2);
        form.update_focus();
        assert!(!form.title.focused);
        assert!(!form.description.focused);
        assert!(form.agent_selector.focused);
        assert!(!form.priority_selector.focused);
    }
    
    #[test]
    fn test_form_validation_success() {
        let mut form = CreateTaskForm::new();
        form.title.set_value("Test Task".to_string());
        form.description.set_value("This is a test task description".to_string());
        
        let agents = vec![create_test_agent()];
        form.set_agents(agents);
        form.agent_selector.set_selected_agent("test_agent");
        
        assert!(form.validate().is_ok());
        assert!(form.is_complete());
        assert!(form.is_valid());
    }
    
    #[test]
    fn test_form_validation_required_fields() {
        let form = CreateTaskForm::new();
        
        let result = form.validate();
        assert!(result.is_err());
        
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.contains("Task Title is required")));
        assert!(errors.iter().any(|e| e.contains("Task Description is required")));
        assert!(errors.iter().any(|e| e.contains("Agent selection is required")));
    }
    
    #[test]
    fn test_form_validation_field_lengths() {
        let mut form = CreateTaskForm::new();
        
        // Test title too short
        form.title.set_value("Hi".to_string());
        form.description.set_value("Short".to_string());
        
        let result = form.validate();
        assert!(result.is_err());
        
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.contains("Task Title must be at least 3 characters")));
        assert!(errors.iter().any(|e| e.contains("Task Description must be at least 10 characters")));
    }
    
    #[test]
    fn test_form_submission_success() {
        let mut form = CreateTaskForm::new();
        form.title.set_value("Test Task".to_string());
        form.description.set_value("This is a test task description".to_string());
        
        let agents = vec![create_test_agent()];
        form.set_agents(agents);
        form.agent_selector.set_selected_agent("test_agent");
        form.priority_selector.set_selected_priority(TaskPriority::High);
        
        let result = form.submit();
        assert!(result.is_ok());
        
        let task = result.unwrap();
        assert_eq!(task.title, "Test Task");
        assert_eq!(task.description, "This is a test task description");
        assert_eq!(task.agent_id, "test_agent");
        assert_eq!(task.priority, TaskPriority::High);
        assert_eq!(task.status, TaskStatus::Todo);
    }
    
    #[test]
    fn test_form_submission_failure() {
        let form = CreateTaskForm::new();
        // Don't set any values
        
        let result = form.submit();
        assert!(result.is_err());
        
        let error = result.unwrap_err();
        assert!(error.contains("Task Title is required"));
        assert!(error.contains("Task Description is required"));
        assert!(error.contains("Agent selection is required"));
    }
    
    #[test]
    fn test_form_is_complete() {
        let mut form = CreateTaskForm::new();
        assert!(!form.is_complete());
        
        form.title.set_value("Test Task".to_string());
        assert!(!form.is_complete());
        
        form.description.set_value("Test description".to_string());
        assert!(!form.is_complete());
        
        let agents = vec![create_test_agent()];
        form.set_agents(agents);
        form.agent_selector.set_selected_agent("test_agent");
        assert!(form.is_complete());
    }
    
    #[test]
    fn test_form_handle_input_navigation() {
        let mut form = CreateTaskForm::new();
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
        let mut form = CreateTaskForm::new();
        form.update_focus();
        
        // Test character input to title field
        let result = form.handle_input(KeyCode::Char('T'));
        assert_eq!(result, FormResult::Continue);
        assert_eq!(form.title.value(), "T");
        
        // Test character input continues
        let result = form.handle_input(KeyCode::Char('e'));
        assert_eq!(result, FormResult::Continue);
        assert_eq!(form.title.value(), "Te");
    }
    
    #[test]
    fn test_set_agents() {
        let mut form = CreateTaskForm::new();
        let agents = vec![create_test_agent()];
        
        form.set_agents(agents.clone());
        assert_eq!(form.agent_selector.agents.len(), 1);
        assert_eq!(form.agent_selector.agents[0].id, "test_agent");
    }
    
    #[test]
    fn test_priority_selection() {
        let mut form = CreateTaskForm::new();
        
        // Test default priority
        assert_eq!(*form.priority_selector.selected_priority(), TaskPriority::Medium);
        
        // Test setting priority
        form.priority_selector.set_selected_priority(TaskPriority::High);
        assert_eq!(*form.priority_selector.selected_priority(), TaskPriority::High);
        
        form.priority_selector.set_selected_priority(TaskPriority::Low);
        assert_eq!(*form.priority_selector.selected_priority(), TaskPriority::Low);
    }
}