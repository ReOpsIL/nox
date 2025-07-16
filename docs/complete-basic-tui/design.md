# Design Document

## Overview

The complete basic TUI functionality will transform the current read-only TUI into a fully interactive terminal interface for managing Nox agents and tasks. This design builds upon the existing TUI framework while adding interactive forms, confirmation dialogs, real-time feedback, and comprehensive keybinding support.

The implementation will extend the current screen-based architecture with new UI components for forms and dialogs, enhance the event handling system to support complex interactions, and integrate with the existing core managers for seamless operation execution.

## Architecture

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Enhanced TUI Layer                       │
├─────────────────┬─────────────────┬─────────────────────────┤
│ Interactive     │ Form System     │    Dialog System        │
│ Keybindings     │                 │                         │
├─────────────────┼─────────────────┼─────────────────────────┤
│ State Manager   │ Error Handler   │   Progress Tracker      │
└─────────────────┴─────────────────┴─────────────────────────┘
                            │
┌─────────────────────────────────────────────────────────────┐
│                 Existing TUI Framework                      │
├─────────────────┬─────────────────┬─────────────────────────┤
│ Screen System   │ Event Handler   │   Component System      │
├─────────────────┼─────────────────┼─────────────────────────┤
│ App State       │ UI Renderer     │   Utility Functions     │
└─────────────────┴─────────────────┴─────────────────────────┘
                            │
┌─────────────────────────────────────────────────────────────┐
│                   Existing Nox Core                         │
├─────────────────┬─────────────────┬─────────────────────────┤
│ Agent Manager   │ Task Manager    │   Registry Manager      │
└─────────────────┴─────────────────┴─────────────────────────┘
```

### Component Integration

The enhanced TUI integrates with existing components:
- **App State**: Extended with form state, dialog state, and operation status
- **Event Handler**: Enhanced to handle form inputs and complex key combinations
- **Screen System**: Extended with overlay support for forms and dialogs
- **Core Managers**: Direct integration for performing operations

## Components and Interfaces

### 1. Enhanced App State (`src/tui/app.rs`)

Extended application state to support interactive operations:

```rust
#[derive(Debug, Clone)]
pub struct AppState {
    // Existing fields
    pub current_screen: Screen,
    pub agents: Vec<Agent>,
    pub tasks: Vec<Task>,
    pub selected_agent: Option<usize>,
    pub selected_task: Option<usize>,
    pub system_status: SystemStatus,
    pub filters: FilterState,
    pub should_quit: bool,
    pub active_panel: usize,
    
    // New interactive fields
    pub current_form: Option<FormState>,
    pub current_dialog: Option<DialogState>,
    pub operation_status: OperationStatus,
    pub search_state: SearchState,
    pub help_visible: bool,
    pub error_message: Option<String>,
    pub success_message: Option<String>,
}

#[derive(Debug, Clone)]
pub enum FormState {
    CreateAgent(CreateAgentForm),
    EditAgent(EditAgentForm),
    CreateTask(CreateTaskForm),
    EditTask(EditTaskForm),
}

#[derive(Debug, Clone)]
pub enum DialogState {
    Confirmation(ConfirmationDialog),
    Progress(ProgressDialog),
    Help(HelpDialog),
}

#[derive(Debug, Clone)]
pub struct OperationStatus {
    pub is_loading: bool,
    pub operation_type: Option<String>,
    pub progress: Option<u8>,
}

#[derive(Debug, Clone)]
pub struct SearchState {
    pub active: bool,
    pub query: String,
    pub filtered_indices: Vec<usize>,
}
```

### 2. Form System (`src/tui/forms/mod.rs`)

Interactive form components for data entry:

```rust
pub trait Form {
    type Output;
    
    fn render(&self, frame: &mut Frame, area: Rect);
    fn handle_input(&mut self, key: KeyCode) -> FormResult;
    fn validate(&self) -> Result<(), Vec<String>>;
    fn submit(self) -> Result<Self::Output, String>;
    fn is_complete(&self) -> bool;
}

#[derive(Debug, Clone)]
pub enum FormResult {
    Continue,
    Submit,
    Cancel,
    NextField,
    PreviousField,
}

#[derive(Debug, Clone)]
pub struct CreateAgentForm {
    pub name: TextInput,
    pub system_prompt: TextArea,
    pub current_field: usize,
    pub validation_errors: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct CreateTaskForm {
    pub title: TextInput,
    pub description: TextArea,
    pub agent_selector: AgentSelector,
    pub priority: PrioritySelector,
    pub current_field: usize,
    pub validation_errors: Vec<String>,
}

impl Form for CreateAgentForm {
    type Output = Agent;
    
    fn render(&self, frame: &mut Frame, area: Rect) {
        // Render form fields with highlighting for current field
        // Show validation errors at the bottom
        // Display help text for navigation
    }
    
    fn handle_input(&mut self, key: KeyCode) -> FormResult {
        match key {
            KeyCode::Tab => FormResult::NextField,
            KeyCode::BackTab => FormResult::PreviousField,
            KeyCode::Enter => {
                if self.is_complete() && self.validate().is_ok() {
                    FormResult::Submit
                } else {
                    FormResult::NextField
                }
            }
            KeyCode::Escape => FormResult::Cancel,
            _ => {
                // Delegate to current field
                self.handle_field_input(key);
                FormResult::Continue
            }
        }
    }
}
```

### 3. Dialog System (`src/tui/dialogs/mod.rs`)

Modal dialog components for confirmations and progress:

```rust
pub trait Dialog {
    fn render(&self, frame: &mut Frame, area: Rect);
    fn handle_input(&mut self, key: KeyCode) -> DialogResult;
    fn title(&self) -> &str;
}

#[derive(Debug, Clone)]
pub enum DialogResult {
    Continue,
    Confirm,
    Cancel,
    Close,
}

#[derive(Debug, Clone)]
pub struct ConfirmationDialog {
    pub title: String,
    pub message: String,
    pub confirm_text: String,
    pub cancel_text: String,
    pub selected_button: usize,
    pub destructive: bool,
}

#[derive(Debug, Clone)]
pub struct ProgressDialog {
    pub title: String,
    pub message: String,
    pub progress: Option<u8>,
    pub can_cancel: bool,
}

impl Dialog for ConfirmationDialog {
    fn render(&self, frame: &mut Frame, area: Rect) {
        // Center the dialog in the area
        // Show title, message, and buttons
        // Highlight selected button
        // Use warning colors for destructive actions
    }
    
    fn handle_input(&mut self, key: KeyCode) -> DialogResult {
        match key {
            KeyCode::Tab | KeyCode::Right => {
                self.selected_button = (self.selected_button + 1) % 2;
                DialogResult::Continue
            }
            KeyCode::Left => {
                self.selected_button = if self.selected_button == 0 { 1 } else { 0 };
                DialogResult::Continue
            }
            KeyCode::Enter => {
                if self.selected_button == 0 {
                    DialogResult::Confirm
                } else {
                    DialogResult::Cancel
                }
            }
            KeyCode::Escape => DialogResult::Cancel,
            KeyCode::Char('y') | KeyCode::Char('Y') => DialogResult::Confirm,
            KeyCode::Char('n') | KeyCode::Char('N') => DialogResult::Cancel,
            _ => DialogResult::Continue,
        }
    }
}
```

### 4. Enhanced Event Handling (`src/tui/app.rs`)

Extended key handling for interactive operations:

```rust
impl App {
    pub fn handle_key_input(&mut self, key: KeyCode) -> Result<()> {
        // Handle global keys first
        if self.handle_global_keys(key)? {
            return Ok(());
        }
        
        // Handle form input if form is active
        if let Some(form_state) = &mut self.state.current_form {
            return self.handle_form_input(form_state, key);
        }
        
        // Handle dialog input if dialog is active
        if let Some(dialog_state) = &mut self.state.current_dialog {
            return self.handle_dialog_input(dialog_state, key);
        }
        
        // Handle search input if search is active
        if self.state.search_state.active {
            return self.handle_search_input(key);
        }
        
        // Handle screen-specific keys
        match self.state.current_screen {
            Screen::Agents => self.handle_agents_keys(key),
            Screen::Tasks => self.handle_tasks_keys(key),
            _ => self.handle_navigation_keys(key),
        }
    }
    
    fn handle_agents_keys(&mut self, key: KeyCode) -> Result<()> {
        match key {
            KeyCode::Char('n') | KeyCode::Char('N') => {
                self.show_create_agent_form();
            }
            KeyCode::Char('e') | KeyCode::Char('E') => {
                if let Some(agent) = self.get_selected_agent() {
                    self.show_edit_agent_form(agent.clone());
                }
            }
            KeyCode::Char('s') | KeyCode::Char('S') => {
                if let Some(agent) = self.get_selected_agent() {
                    self.start_agent_operation(agent.id.clone());
                }
            }
            KeyCode::Char('t') | KeyCode::Char('T') => {
                if let Some(agent) = self.get_selected_agent() {
                    self.show_stop_agent_confirmation(agent.clone());
                }
            }
            KeyCode::Char('d') | KeyCode::Char('D') => {
                if let Some(agent) = self.get_selected_agent() {
                    self.show_delete_agent_confirmation(agent.clone());
                }
            }
            KeyCode::Char('/') => {
                self.activate_search();
            }
            KeyCode::Char('f') | KeyCode::Char('F') => {
                self.show_filter_options();
            }
            _ => self.handle_navigation_keys(key)?,
        }
        Ok(())
    }
    
    async fn start_agent_operation(&mut self, agent_id: String) -> Result<()> {
        self.state.operation_status.is_loading = true;
        self.state.operation_status.operation_type = Some("Starting agent".to_string());
        
        match agent_manager::start_agent(&agent_id).await {
            Ok(_) => {
                self.state.success_message = Some("Agent started successfully".to_string());
                self.refresh_data().await?;
            }
            Err(e) => {
                self.state.error_message = Some(format!("Failed to start agent: {}", e));
            }
        }
        
        self.state.operation_status.is_loading = false;
        self.state.operation_status.operation_type = None;
        Ok(())
    }
}
```

### 5. Input Components (`src/tui/components/inputs.rs`)

Reusable input field components:

```rust
#[derive(Debug, Clone)]
pub struct TextInput {
    pub value: String,
    pub placeholder: String,
    pub cursor_position: usize,
    pub focused: bool,
    pub max_length: Option<usize>,
    pub validation_regex: Option<Regex>,
}

#[derive(Debug, Clone)]
pub struct TextArea {
    pub lines: Vec<String>,
    pub cursor_line: usize,
    pub cursor_column: usize,
    pub focused: bool,
    pub max_lines: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct AgentSelector {
    pub agents: Vec<Agent>,
    pub selected_index: Option<usize>,
    pub focused: bool,
    pub filter_text: String,
}

impl TextInput {
    pub fn handle_input(&mut self, key: KeyCode) -> bool {
        match key {
            KeyCode::Char(c) => {
                if let Some(max_len) = self.max_length {
                    if self.value.len() >= max_len {
                        return false;
                    }
                }
                self.value.insert(self.cursor_position, c);
                self.cursor_position += 1;
                true
            }
            KeyCode::Backspace => {
                if self.cursor_position > 0 {
                    self.cursor_position -= 1;
                    self.value.remove(self.cursor_position);
                }
                true
            }
            KeyCode::Delete => {
                if self.cursor_position < self.value.len() {
                    self.value.remove(self.cursor_position);
                }
                true
            }
            KeyCode::Left => {
                if self.cursor_position > 0 {
                    self.cursor_position -= 1;
                }
                true
            }
            KeyCode::Right => {
                if self.cursor_position < self.value.len() {
                    self.cursor_position += 1;
                }
                true
            }
            KeyCode::Home => {
                self.cursor_position = 0;
                true
            }
            KeyCode::End => {
                self.cursor_position = self.value.len();
                true
            }
            _ => false,
        }
    }
    
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let style = if self.focused {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };
        
        let text = if self.value.is_empty() && !self.focused {
            self.placeholder.as_str()
        } else {
            self.value.as_str()
        };
        
        let paragraph = Paragraph::new(text)
            .style(style)
            .block(Block::default().borders(Borders::ALL));
            
        frame.render_widget(paragraph, area);
        
        // Render cursor if focused
        if self.focused {
            let cursor_x = area.x + 1 + self.cursor_position as u16;
            let cursor_y = area.y + 1;
            frame.set_cursor(cursor_x, cursor_y);
        }
    }
}
```

### 6. Enhanced UI Rendering (`src/tui/ui.rs`)

Updated rendering system to support overlays:

```rust
pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.size();
    
    // Render main screen
    match app.state.current_screen {
        Screen::Dashboard => Dashboard::render(frame, area, &app.state),
        Screen::Agents => AgentsScreen::render(frame, area, &app.state),
        Screen::Tasks => TasksScreen::render(frame, area, &app.state),
        Screen::Execution => ExecutionScreen::render(frame, area, &app.state),
        Screen::Logs => LogsScreen::render(frame, area, &app.state),
    }
    
    // Render overlays
    render_overlays(frame, area, app);
}

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
    
    // Render help overlay
    if app.state.help_visible {
        render_help_overlay(frame, area, &app.state.current_screen);
    }
    
    // Render messages
    render_messages(frame, area, app);
}

fn render_form_overlay(frame: &mut Frame, area: Rect, form_state: &FormState) {
    // Create centered overlay area
    let overlay_area = centered_rect(80, 60, area);
    
    // Clear background
    frame.render_widget(Clear, overlay_area);
    
    // Render form based on type
    match form_state {
        FormState::CreateAgent(form) => form.render(frame, overlay_area),
        FormState::EditAgent(form) => form.render(frame, overlay_area),
        FormState::CreateTask(form) => form.render(frame, overlay_area),
        FormState::EditTask(form) => form.render(frame, overlay_area),
    }
}

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
```

## Data Models

### Enhanced Form Models

```rust
#[derive(Debug, Clone)]
pub struct CreateAgentForm {
    pub name: TextInput,
    pub system_prompt: TextArea,
    pub resource_limits: ResourceLimitsForm,
    pub current_field: usize,
    pub validation_errors: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ResourceLimitsForm {
    pub max_memory_mb: TextInput,
    pub max_cpu_percent: TextInput,
}

#[derive(Debug, Clone)]
pub struct CreateTaskForm {
    pub title: TextInput,
    pub description: TextArea,
    pub agent_selector: AgentSelector,
    pub priority: PrioritySelector,
    pub current_field: usize,
    pub validation_errors: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum PrioritySelector {
    High,
    Medium,
    Low,
}
```

### Dialog Models

```rust
#[derive(Debug, Clone)]
pub struct ConfirmationDialog {
    pub title: String,
    pub message: String,
    pub details: Option<String>,
    pub confirm_text: String,
    pub cancel_text: String,
    pub selected_button: usize,
    pub destructive: bool,
}

#[derive(Debug, Clone)]
pub struct ProgressDialog {
    pub title: String,
    pub message: String,
    pub progress: Option<u8>,
    pub can_cancel: bool,
    pub start_time: Instant,
}
```

## Error Handling

### TUI-Specific Error Types

```rust
#[derive(Debug, thiserror::Error)]
pub enum TuiError {
    #[error("Form validation failed: {errors:?}")]
    FormValidation { errors: Vec<String> },
    
    #[error("Operation cancelled by user")]
    OperationCancelled,
    
    #[error("Invalid input: {input}")]
    InvalidInput { input: String },
    
    #[error("Agent operation failed: {reason}")]
    AgentOperationFailed { reason: String },
    
    #[error("Task operation failed: {reason}")]
    TaskOperationFailed { reason: String },
    
    #[error("UI rendering error: {reason}")]
    RenderingError { reason: String },
}
```

### Error Recovery Strategies

1. **Form Validation Errors**: Display inline validation messages and prevent submission
2. **Operation Failures**: Show error messages with retry options
3. **Network Errors**: Display user-friendly messages with troubleshooting hints
4. **Input Errors**: Provide immediate feedback and correction suggestions
5. **State Corruption**: Reset to safe state and notify user

## Testing Strategy

### Unit Tests

1. **Form Component Tests**
   - Input validation and sanitization
   - Field navigation and focus management
   - Form submission and cancellation

2. **Dialog Component Tests**
   - Button navigation and selection
   - Confirmation and cancellation flows
   - Progress dialog updates

3. **Input Component Tests**
   - Text input handling and cursor management
   - Text area multi-line editing
   - Selector component filtering and selection

### Integration Tests

1. **End-to-End Operation Tests**
   - Complete agent creation workflow
   - Complete task management workflow
   - Error handling and recovery scenarios

2. **UI Interaction Tests**
   - Keyboard navigation between components
   - Form validation and submission
   - Dialog confirmation flows

3. **State Management Tests**
   - Context preservation during operations
   - Filter and search state management
   - Error and success message handling

The design ensures a complete, user-friendly TUI experience while maintaining the existing architecture and integrating seamlessly with the core Nox functionality.