//! Input components for TUI forms
//! 
//! This module provides reusable input components for interactive forms in the TUI.

use crossterm::event::KeyCode;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
    style::{Color, Modifier, Style},
};
use regex::Regex;
use crate::types::{Agent, TaskPriority};

/// Result of handling input in a component
#[derive(Debug, Clone, PartialEq)]
pub enum InputResult {
    /// Continue processing input
    Continue,
    /// Move to next field
    NextField,
    /// Move to previous field
    PreviousField,
    /// Submit the form
    Submit,
    /// Cancel the operation
    Cancel,
}

/// Single-line text input component with cursor management and validation
#[derive(Debug, Clone)]
pub struct TextInput {
    /// Current text value
    pub value: String,
    /// Placeholder text when empty
    pub placeholder: String,
    /// Current cursor position
    pub cursor_position: usize,
    /// Whether this input is currently focused
    pub focused: bool,
    /// Maximum allowed length (None for unlimited)
    pub max_length: Option<usize>,
    /// Optional validation regex
    pub validation_regex: Option<Regex>,
    /// Label for the input field
    pub label: String,
    /// Whether this field is required
    pub required: bool,
    /// Current validation error message
    pub error_message: Option<String>,
}

impl TextInput {
    /// Create a new text input with the given label
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            value: String::new(),
            placeholder: String::new(),
            cursor_position: 0,
            focused: false,
            max_length: None,
            validation_regex: None,
            label: label.into(),
            required: false,
            error_message: None,
        }
    }

    /// Set placeholder text
    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    /// Set maximum length
    pub fn max_length(mut self, max_length: usize) -> Self {
        self.max_length = Some(max_length);
        self
    }

    /// Set validation regex
    pub fn validation_regex(mut self, regex: Regex) -> Self {
        self.validation_regex = Some(regex);
        self
    }

    /// Mark as required field
    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    /// Set the current value
    pub fn set_value(&mut self, value: String) {
        self.value = value;
        self.cursor_position = self.value.len().min(self.cursor_position);
        self.validate();
    }

    /// Get the current value
    pub fn value(&self) -> &str {
        &self.value
    }

    /// Set focus state
    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    /// Check if the input is valid
    pub fn is_valid(&self) -> bool {
        self.error_message.is_none()
    }

    /// Validate the current input
    pub fn validate(&mut self) {
        self.error_message = None;

        // Check required field
        if self.required && self.value.trim().is_empty() {
            self.error_message = Some(format!("{} is required", self.label));
            return;
        }

        // Check regex validation
        if let Some(regex) = &self.validation_regex {
            if !self.value.is_empty() && !regex.is_match(&self.value) {
                self.error_message = Some(format!("{} format is invalid", self.label));
                return;
            }
        }

        // Check max length
        if let Some(max_len) = self.max_length {
            if self.value.len() > max_len {
                self.error_message = Some(format!("{} must be {} characters or less", self.label, max_len));
                return;
            }
        }
    }

    /// Handle keyboard input
    pub fn handle_input(&mut self, key: KeyCode) -> InputResult {
        match key {
            KeyCode::Char(c) => {
                if let Some(max_len) = self.max_length {
                    if self.value.len() >= max_len {
                        return InputResult::Continue;
                    }
                }
                self.value.insert(self.cursor_position, c);
                self.cursor_position += 1;
                self.validate();
                InputResult::Continue
            }
            KeyCode::Backspace => {
                if self.cursor_position > 0 {
                    self.cursor_position -= 1;
                    self.value.remove(self.cursor_position);
                    self.validate();
                }
                InputResult::Continue
            }
            KeyCode::Delete => {
                if self.cursor_position < self.value.len() {
                    self.value.remove(self.cursor_position);
                    self.validate();
                }
                InputResult::Continue
            }
            KeyCode::Left => {
                if self.cursor_position > 0 {
                    self.cursor_position -= 1;
                }
                InputResult::Continue
            }
            KeyCode::Right => {
                if self.cursor_position < self.value.len() {
                    self.cursor_position += 1;
                }
                InputResult::Continue
            }
            KeyCode::Home => {
                self.cursor_position = 0;
                InputResult::Continue
            }
            KeyCode::End => {
                self.cursor_position = self.value.len();
                InputResult::Continue
            }
            KeyCode::Tab => InputResult::NextField,
            KeyCode::BackTab => InputResult::PreviousField,
            KeyCode::Enter => InputResult::NextField,
            KeyCode::Esc => InputResult::Cancel,
            _ => InputResult::Continue,
        }
    }

    /// Render the text input
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Label
                Constraint::Length(3), // Input box
                Constraint::Length(1), // Error message
            ])
            .split(area);

        // Render label
        let label_style = if self.required {
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };
        let label_text = if self.required {
            format!("{} *", self.label)
        } else {
            self.label.clone()
        };
        frame.render_widget(
            Paragraph::new(label_text).style(label_style),
            chunks[0]
        );

        // Render input box
        let input_style = if self.focused {
            Style::default().fg(Color::Yellow)
        } else if self.error_message.is_some() {
            Style::default().fg(Color::Red)
        } else {
            Style::default().fg(Color::White)
        };

        let border_style = if self.focused {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else if self.error_message.is_some() {
            Style::default().fg(Color::Red)
        } else {
            Style::default().fg(Color::Gray)
        };

        let display_text = if self.value.is_empty() && !self.focused {
            &self.placeholder
        } else {
            &self.value
        };

        let text_style = if self.value.is_empty() && !self.focused {
            Style::default().fg(Color::DarkGray)
        } else {
            input_style
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(border_style);

        frame.render_widget(
            Paragraph::new(display_text.as_str())
                .style(text_style)
                .block(block),
            chunks[1]
        );

        // Render cursor if focused
        if self.focused {
            let cursor_x = chunks[1].x + 1 + self.cursor_position as u16;
            let cursor_y = chunks[1].y + 1;
            if cursor_x < chunks[1].x + chunks[1].width - 1 {
                frame.set_cursor(cursor_x, cursor_y);
            }
        }

        // Render error message
        if let Some(error) = &self.error_message {
            frame.render_widget(
                Paragraph::new(error.as_str())
                    .style(Style::default().fg(Color::Red)),
                chunks[2]
            );
        }
    }
}

/// Multi-line text area component with line navigation
#[derive(Debug, Clone)]
pub struct TextArea {
    /// Lines of text
    pub lines: Vec<String>,
    /// Current cursor line
    pub cursor_line: usize,
    /// Current cursor column
    pub cursor_column: usize,
    /// Whether this text area is focused
    pub focused: bool,
    /// Maximum number of lines (None for unlimited)
    pub max_lines: Option<usize>,
    /// Label for the text area
    pub label: String,
    /// Whether this field is required
    pub required: bool,
    /// Current validation error message
    pub error_message: Option<String>,
    /// Scroll offset for vertical scrolling
    pub scroll_offset: usize,
}

impl TextArea {
    /// Create a new text area with the given label
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            lines: vec![String::new()],
            cursor_line: 0,
            cursor_column: 0,
            focused: false,
            max_lines: None,
            label: label.into(),
            required: false,
            error_message: None,
            scroll_offset: 0,
        }
    }

    /// Set maximum number of lines
    pub fn max_lines(mut self, max_lines: usize) -> Self {
        self.max_lines = Some(max_lines);
        self
    }

    /// Mark as required field
    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    /// Set the current value
    pub fn set_value(&mut self, value: String) {
        self.lines = if value.is_empty() {
            vec![String::new()]
        } else {
            value.lines().map(|s| s.to_string()).collect()
        };
        self.cursor_line = 0;
        self.cursor_column = 0;
        self.scroll_offset = 0;
        self.validate();
    }

    /// Get the current value as a single string
    pub fn value(&self) -> String {
        self.lines.join("\n")
    }

    /// Set focus state
    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    /// Check if the text area is valid
    pub fn is_valid(&self) -> bool {
        self.error_message.is_none()
    }

    /// Validate the current input
    pub fn validate(&mut self) {
        self.error_message = None;

        // Check required field
        if self.required {
            let content = self.value();
            let trimmed_content = content.trim();
            if trimmed_content.is_empty() {
                self.error_message = Some(format!("{} is required", self.label));
                return;
            }
        }

        // Check max lines
        if let Some(max_lines) = self.max_lines {
            if self.lines.len() > max_lines {
                self.error_message = Some(format!("{} must be {} lines or less", self.label, max_lines));
                return;
            }
        }
    }

    /// Handle keyboard input
    pub fn handle_input(&mut self, key: KeyCode) -> InputResult {
        match key {
            KeyCode::Char(c) => {
                if c == '\n' || c == '\r' {
                    // Handle Enter key
                    if let Some(max_lines) = self.max_lines {
                        if self.lines.len() >= max_lines {
                            return InputResult::Continue;
                        }
                    }
                    
                    let current_line = self.lines[self.cursor_line].clone();
                    let (before, after) = current_line.split_at(self.cursor_column);
                    
                    self.lines[self.cursor_line] = before.to_string();
                    self.lines.insert(self.cursor_line + 1, after.to_string());
                    
                    self.cursor_line += 1;
                    self.cursor_column = 0;
                    
                    self.validate();
                } else {
                    // Regular character input
                    if self.cursor_line < self.lines.len() {
                        self.lines[self.cursor_line].insert(self.cursor_column, c);
                        self.cursor_column += 1;
                        self.validate();
                    }
                }
                InputResult::Continue
            }
            KeyCode::Enter => {
                // Handle Enter key for new line
                if let Some(max_lines) = self.max_lines {
                    if self.lines.len() >= max_lines {
                        return InputResult::Continue;
                    }
                }
                
                let current_line = self.lines[self.cursor_line].clone();
                let (before, after) = current_line.split_at(self.cursor_column);
                
                self.lines[self.cursor_line] = before.to_string();
                self.lines.insert(self.cursor_line + 1, after.to_string());
                
                self.cursor_line += 1;
                self.cursor_column = 0;
                
                self.validate();
                InputResult::Continue
            }
            KeyCode::Backspace => {
                if self.cursor_column > 0 {
                    // Remove character from current line
                    self.cursor_column -= 1;
                    self.lines[self.cursor_line].remove(self.cursor_column);
                } else if self.cursor_line > 0 {
                    // Merge with previous line
                    let current_line = self.lines.remove(self.cursor_line);
                    self.cursor_line -= 1;
                    self.cursor_column = self.lines[self.cursor_line].len();
                    self.lines[self.cursor_line].push_str(&current_line);
                }
                self.validate();
                InputResult::Continue
            }
            KeyCode::Delete => {
                if self.cursor_column < self.lines[self.cursor_line].len() {
                    // Remove character from current line
                    self.lines[self.cursor_line].remove(self.cursor_column);
                } else if self.cursor_line < self.lines.len() - 1 {
                    // Merge with next line
                    let next_line = self.lines.remove(self.cursor_line + 1);
                    self.lines[self.cursor_line].push_str(&next_line);
                }
                self.validate();
                InputResult::Continue
            }
            KeyCode::Left => {
                if self.cursor_column > 0 {
                    self.cursor_column -= 1;
                } else if self.cursor_line > 0 {
                    self.cursor_line -= 1;
                    self.cursor_column = self.lines[self.cursor_line].len();
                }
                InputResult::Continue
            }
            KeyCode::Right => {
                if self.cursor_column < self.lines[self.cursor_line].len() {
                    self.cursor_column += 1;
                } else if self.cursor_line < self.lines.len() - 1 {
                    self.cursor_line += 1;
                    self.cursor_column = 0;
                }
                InputResult::Continue
            }
            KeyCode::Up => {
                if self.cursor_line > 0 {
                    self.cursor_line -= 1;
                    self.cursor_column = self.cursor_column.min(self.lines[self.cursor_line].len());
                }
                InputResult::Continue
            }
            KeyCode::Down => {
                if self.cursor_line < self.lines.len() - 1 {
                    self.cursor_line += 1;
                    self.cursor_column = self.cursor_column.min(self.lines[self.cursor_line].len());
                }
                InputResult::Continue
            }
            KeyCode::Home => {
                self.cursor_column = 0;
                InputResult::Continue
            }
            KeyCode::End => {
                self.cursor_column = self.lines[self.cursor_line].len();
                InputResult::Continue
            }
            KeyCode::Tab => InputResult::NextField,
            KeyCode::BackTab => InputResult::PreviousField,
            KeyCode::Esc => InputResult::Cancel,
            _ => InputResult::Continue,
        }
    }

    /// Render the text area
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Label
                Constraint::Min(3),    // Text area
                Constraint::Length(1), // Error message
            ])
            .split(area);

        // Render label
        let label_style = if self.required {
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };
        let label_text = if self.required {
            format!("{} *", self.label)
        } else {
            self.label.clone()
        };
        frame.render_widget(
            Paragraph::new(label_text).style(label_style),
            chunks[0]
        );

        // Render text area
        let text_area_height = chunks[1].height.saturating_sub(2) as usize; // Account for borders
        let visible_lines: Vec<String> = self.lines
            .iter()
            .skip(self.scroll_offset)
            .take(text_area_height)
            .cloned()
            .collect();

        let border_style = if self.focused {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else if self.error_message.is_some() {
            Style::default().fg(Color::Red)
        } else {
            Style::default().fg(Color::Gray)
        };

        let text_style = if self.focused {
            Style::default().fg(Color::Yellow)
        } else if self.error_message.is_some() {
            Style::default().fg(Color::Red)
        } else {
            Style::default().fg(Color::White)
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(border_style);

        frame.render_widget(
            Paragraph::new(visible_lines.join("\n"))
                .style(text_style)
                .block(block),
            chunks[1]
        );

        // Render cursor if focused
        if self.focused && self.cursor_line >= self.scroll_offset && self.cursor_line < self.scroll_offset + text_area_height {
            let cursor_x = chunks[1].x + 1 + self.cursor_column as u16;
            let cursor_y = chunks[1].y + 1 + (self.cursor_line - self.scroll_offset) as u16;
            if cursor_x < chunks[1].x + chunks[1].width - 1 && cursor_y < chunks[1].y + chunks[1].height - 1 {
                frame.set_cursor(cursor_x, cursor_y);
            }
        }

        // Render error message
        if let Some(error) = &self.error_message {
            frame.render_widget(
                Paragraph::new(error.as_str())
                    .style(Style::default().fg(Color::Red)),
                chunks[2]
            );
        }
    }
}

/// Agent selector component with filtering and selection capabilities
#[derive(Debug, Clone)]
pub struct AgentSelector {
    /// Available agents
    pub agents: Vec<Agent>,
    /// Currently selected agent index
    pub selected_index: Option<usize>,
    /// Whether this selector is focused
    pub focused: bool,
    /// Filter text for searching agents
    pub filter_text: String,
    /// Filtered agent indices
    pub filtered_indices: Vec<usize>,
    /// Label for the selector
    pub label: String,
    /// Whether this field is required
    pub required: bool,
    /// Current validation error message
    pub error_message: Option<String>,
    /// List state for rendering
    pub list_state: ListState,
    /// Whether the dropdown is open
    pub dropdown_open: bool,
}

impl AgentSelector {
    /// Create a new agent selector with the given label
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            agents: Vec::new(),
            selected_index: None,
            focused: false,
            filter_text: String::new(),
            filtered_indices: Vec::new(),
            label: label.into(),
            required: false,
            error_message: None,
            list_state: ListState::default(),
            dropdown_open: false,
        }
    }

    /// Mark as required field
    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    /// Set the available agents
    pub fn set_agents(&mut self, agents: Vec<Agent>) {
        self.agents = agents;
        self.update_filtered_indices();
        self.validate();
    }

    /// Get the currently selected agent
    pub fn selected_agent(&self) -> Option<&Agent> {
        self.selected_index.and_then(|idx| self.agents.get(idx))
    }

    /// Set the selected agent by ID
    pub fn set_selected_agent(&mut self, agent_id: &str) {
        self.selected_index = self.agents.iter().position(|a| a.id == agent_id);
        self.validate();
    }

    /// Set focus state
    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
        if !focused {
            self.dropdown_open = false;
        }
    }

    /// Check if the selector is valid
    pub fn is_valid(&self) -> bool {
        self.error_message.is_none()
    }

    /// Validate the current selection
    pub fn validate(&mut self) {
        self.error_message = None;

        if self.required && self.selected_index.is_none() {
            self.error_message = Some(format!("{} is required", self.label));
        }
    }

    /// Update filtered indices based on filter text
    fn update_filtered_indices(&mut self) {
        if self.filter_text.is_empty() {
            self.filtered_indices = (0..self.agents.len()).collect();
        } else {
            let filter_lower = self.filter_text.to_lowercase();
            self.filtered_indices = self.agents
                .iter()
                .enumerate()
                .filter(|(_, agent)| {
                    agent.name.to_lowercase().contains(&filter_lower) ||
                    agent.id.to_lowercase().contains(&filter_lower)
                })
                .map(|(idx, _)| idx)
                .collect();
        }
    }

    /// Handle keyboard input
    pub fn handle_input(&mut self, key: KeyCode) -> InputResult {
        if !self.dropdown_open {
            match key {
                KeyCode::Enter | KeyCode::Down => {
                    self.dropdown_open = true;
                    if !self.filtered_indices.is_empty() {
                        self.list_state.select(Some(0));
                    }
                    InputResult::Continue
                }
                KeyCode::Tab => InputResult::NextField,
                KeyCode::BackTab => InputResult::PreviousField,
                KeyCode::Esc => InputResult::Cancel,
                _ => InputResult::Continue,
            }
        } else {
            match key {
                KeyCode::Up => {
                    if !self.filtered_indices.is_empty() {
                        let selected = self.list_state.selected().unwrap_or(0);
                        let new_selected = if selected == 0 {
                            self.filtered_indices.len() - 1
                        } else {
                            selected - 1
                        };
                        self.list_state.select(Some(new_selected));
                    }
                    InputResult::Continue
                }
                KeyCode::Down => {
                    if !self.filtered_indices.is_empty() {
                        let selected = self.list_state.selected().unwrap_or(0);
                        let new_selected = (selected + 1) % self.filtered_indices.len();
                        self.list_state.select(Some(new_selected));
                    }
                    InputResult::Continue
                }
                KeyCode::Enter => {
                    if let Some(list_selected) = self.list_state.selected() {
                        if let Some(&agent_idx) = self.filtered_indices.get(list_selected) {
                            self.selected_index = Some(agent_idx);
                            self.validate();
                        }
                    }
                    self.dropdown_open = false;
                    InputResult::Continue
                }
                KeyCode::Esc => {
                    self.dropdown_open = false;
                    InputResult::Continue
                }
                KeyCode::Char(c) => {
                    self.filter_text.push(c);
                    self.update_filtered_indices();
                    if !self.filtered_indices.is_empty() {
                        self.list_state.select(Some(0));
                    }
                    InputResult::Continue
                }
                KeyCode::Backspace => {
                    if !self.filter_text.is_empty() {
                        self.filter_text.pop();
                        self.update_filtered_indices();
                        if !self.filtered_indices.is_empty() {
                            self.list_state.select(Some(0));
                        }
                    }
                    InputResult::Continue
                }
                KeyCode::Tab => {
                    self.dropdown_open = false;
                    InputResult::NextField
                }
                KeyCode::BackTab => {
                    self.dropdown_open = false;
                    InputResult::PreviousField
                }
                _ => InputResult::Continue,
            }
        }
    }

    /// Render the agent selector
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Label
                Constraint::Length(3), // Selector box
                Constraint::Min(0),    // Dropdown (if open)
                Constraint::Length(1), // Error message
            ])
            .split(area);

        // Render label
        let label_style = if self.required {
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };
        let label_text = if self.required {
            format!("{} *", self.label)
        } else {
            self.label.clone()
        };
        frame.render_widget(
            Paragraph::new(label_text).style(label_style),
            chunks[0]
        );

        // Render selector box
        let border_style = if self.focused {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else if self.error_message.is_some() {
            Style::default().fg(Color::Red)
        } else {
            Style::default().fg(Color::Gray)
        };

        let display_text = if let Some(agent) = self.selected_agent() {
            format!("{} ({})", agent.name, agent.id)
        } else {
            "Select an agent...".to_string()
        };

        let text_style = if self.selected_agent().is_some() {
            if self.focused {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::White)
            }
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(border_style);

        frame.render_widget(
            Paragraph::new(display_text)
                .style(text_style)
                .block(block),
            chunks[1]
        );

        // Render dropdown if open
        if self.dropdown_open && chunks[2].height > 0 {
            let dropdown_height = (self.filtered_indices.len() + 2).min(chunks[2].height as usize);
            let dropdown_area = Rect {
                x: chunks[1].x,
                y: chunks[1].y + chunks[1].height,
                width: chunks[1].width,
                height: dropdown_height as u16,
            };

            // Clear the dropdown area
            frame.render_widget(Clear, dropdown_area);

            // Create list items
            let items: Vec<ListItem> = self.filtered_indices
                .iter()
                .map(|&idx| {
                    let agent = &self.agents[idx];
                    ListItem::new(format!("{} ({})", agent.name, agent.id))
                })
                .collect();

            let list = List::new(items)
                .block(Block::default().borders(Borders::ALL).border_style(border_style))
                .highlight_style(Style::default().bg(Color::Blue).fg(Color::White))
                .highlight_symbol("► ");

            let mut list_state = self.list_state.clone();
            frame.render_stateful_widget(list, dropdown_area, &mut list_state);

            // Show filter text if any
            if !self.filter_text.is_empty() {
                let filter_area = Rect {
                    x: dropdown_area.x + 1,
                    y: dropdown_area.y + dropdown_area.height - 1,
                    width: dropdown_area.width - 2,
                    height: 1,
                };
                frame.render_widget(
                    Paragraph::new(format!("Filter: {}", self.filter_text))
                        .style(Style::default().fg(Color::Cyan)),
                    filter_area
                );
            }
        }

        // Render error message
        if let Some(error) = &self.error_message {
            let error_chunk_idx = if self.dropdown_open { 3 } else { 2 };
            if error_chunk_idx < chunks.len() {
                frame.render_widget(
                    Paragraph::new(error.as_str())
                        .style(Style::default().fg(Color::Red)),
                    chunks[error_chunk_idx]
                );
            }
        }
    }
}

/// Priority selector component for task priority selection
#[derive(Debug, Clone)]
pub struct PrioritySelector {
    /// Currently selected priority
    pub selected_priority: TaskPriority,
    /// Whether this selector is focused
    pub focused: bool,
    /// Label for the selector
    pub label: String,
    /// List state for rendering
    pub list_state: ListState,
    /// Whether the dropdown is open
    pub dropdown_open: bool,
}

impl PrioritySelector {
    /// Create a new priority selector with the given label
    pub fn new(label: impl Into<String>) -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(1)); // Default to Medium priority
        
        Self {
            selected_priority: TaskPriority::Medium,
            focused: false,
            label: label.into(),
            list_state,
            dropdown_open: false,
        }
    }

    /// Get the currently selected priority
    pub fn selected_priority(&self) -> &TaskPriority {
        &self.selected_priority
    }

    /// Set the selected priority
    pub fn set_selected_priority(&mut self, priority: TaskPriority) {
        self.selected_priority = priority.clone();
        let index = match priority {
            TaskPriority::High => 0,
            TaskPriority::Medium => 1,
            TaskPriority::Low => 2,
        };
        self.list_state.select(Some(index));
    }

    /// Set focus state
    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
        if !focused {
            self.dropdown_open = false;
        }
    }

    /// Handle keyboard input
    pub fn handle_input(&mut self, key: KeyCode) -> InputResult {
        if !self.dropdown_open {
            match key {
                KeyCode::Enter | KeyCode::Down => {
                    self.dropdown_open = true;
                    InputResult::Continue
                }
                KeyCode::Tab => InputResult::NextField,
                KeyCode::BackTab => InputResult::PreviousField,
                KeyCode::Esc => InputResult::Cancel,
                _ => InputResult::Continue,
            }
        } else {
            match key {
                KeyCode::Up => {
                    let selected = self.list_state.selected().unwrap_or(1);
                    let new_selected = if selected == 0 { 2 } else { selected - 1 };
                    self.list_state.select(Some(new_selected));
                    InputResult::Continue
                }
                KeyCode::Down => {
                    let selected = self.list_state.selected().unwrap_or(1);
                    let new_selected = (selected + 1) % 3;
                    self.list_state.select(Some(new_selected));
                    InputResult::Continue
                }
                KeyCode::Enter => {
                    if let Some(selected) = self.list_state.selected() {
                        self.selected_priority = match selected {
                            0 => TaskPriority::High,
                            1 => TaskPriority::Medium,
                            2 => TaskPriority::Low,
                            _ => TaskPriority::Medium,
                        };
                    }
                    self.dropdown_open = false;
                    InputResult::Continue
                }
                KeyCode::Esc => {
                    self.dropdown_open = false;
                    InputResult::Continue
                }
                KeyCode::Tab => {
                    self.dropdown_open = false;
                    InputResult::NextField
                }
                KeyCode::BackTab => {
                    self.dropdown_open = false;
                    InputResult::PreviousField
                }
                _ => InputResult::Continue,
            }
        }
    }

    /// Render the priority selector
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Label
                Constraint::Length(3), // Selector box
                Constraint::Min(0),    // Dropdown (if open)
            ])
            .split(area);

        // Render label
        let label_style = Style::default().fg(Color::Gray);
        frame.render_widget(
            Paragraph::new(self.label.as_str()).style(label_style),
            chunks[0]
        );

        // Render selector box
        let border_style = if self.focused {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };

        let priority_color = match self.selected_priority {
            TaskPriority::High => Color::Red,
            TaskPriority::Medium => Color::Yellow,
            TaskPriority::Low => Color::Green,
        };

        let text_style = if self.focused {
            Style::default().fg(priority_color).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(priority_color)
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(border_style);

        frame.render_widget(
            Paragraph::new(self.selected_priority.to_string())
                .style(text_style)
                .block(block),
            chunks[1]
        );

        // Render dropdown if open
        if self.dropdown_open && chunks[2].height > 0 {
            let dropdown_area = Rect {
                x: chunks[1].x,
                y: chunks[1].y + chunks[1].height,
                width: chunks[1].width,
                height: 5, // 3 items + 2 for borders
            };

            // Clear the dropdown area
            frame.render_widget(Clear, dropdown_area);

            // Create list items
            let items = vec![
                ListItem::new("High").style(Style::default().fg(Color::Red)),
                ListItem::new("Medium").style(Style::default().fg(Color::Yellow)),
                ListItem::new("Low").style(Style::default().fg(Color::Green)),
            ];

            let list = List::new(items)
                .block(Block::default().borders(Borders::ALL).border_style(border_style))
                .highlight_style(Style::default().bg(Color::Blue).fg(Color::White))
                .highlight_symbol("► ");

            let mut list_state = self.list_state.clone();
            frame.render_stateful_widget(list, dropdown_area, &mut list_state);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Agent, AgentStatus, TaskPriority, ResourceLimits};
    use crossterm::event::KeyCode;
    use regex::Regex;
    use chrono::Utc;

    // Helper function to create a test agent
    fn create_test_agent(id: &str, name: &str) -> Agent {
        Agent {
            id: id.to_string(),
            name: name.to_string(),
            system_prompt: "Test prompt".to_string(),
            status: AgentStatus::Inactive,
            created_at: Utc::now(),
            resource_limits: ResourceLimits {
                max_memory_mb: 512,
                max_cpu_percent: 50,
            },
        }
    }

    #[test]
    fn test_text_input_creation() {
        let input = TextInput::new("Test Label");
        assert_eq!(input.label, "Test Label");
        assert_eq!(input.value, "");
        assert_eq!(input.cursor_position, 0);
        assert!(!input.focused);
        assert!(!input.required);
        assert!(input.error_message.is_none());
    }

    #[test]
    fn test_text_input_builder_pattern() {
        let input = TextInput::new("Test")
            .placeholder("Enter text")
            .max_length(10)
            .required();
        
        assert_eq!(input.placeholder, "Enter text");
        assert_eq!(input.max_length, Some(10));
        assert!(input.required);
    }

    #[test]
    fn test_text_input_character_input() {
        let mut input = TextInput::new("Test");
        
        // Test character input
        let result = input.handle_input(KeyCode::Char('a'));
        assert_eq!(result, InputResult::Continue);
        assert_eq!(input.value, "a");
        assert_eq!(input.cursor_position, 1);
        
        // Test another character
        let result = input.handle_input(KeyCode::Char('b'));
        assert_eq!(result, InputResult::Continue);
        assert_eq!(input.value, "ab");
        assert_eq!(input.cursor_position, 2);
    }

    #[test]
    fn test_text_input_backspace() {
        let mut input = TextInput::new("Test");
        input.set_value("hello".to_string());
        input.cursor_position = 5;
        
        let result = input.handle_input(KeyCode::Backspace);
        assert_eq!(result, InputResult::Continue);
        assert_eq!(input.value, "hell");
        assert_eq!(input.cursor_position, 4);
        
        // Test backspace at beginning
        input.cursor_position = 0;
        let result = input.handle_input(KeyCode::Backspace);
        assert_eq!(result, InputResult::Continue);
        assert_eq!(input.value, "hell"); // Should not change
        assert_eq!(input.cursor_position, 0);
    }

    #[test]
    fn test_text_input_delete() {
        let mut input = TextInput::new("Test");
        input.set_value("hello".to_string());
        input.cursor_position = 2;
        
        let result = input.handle_input(KeyCode::Delete);
        assert_eq!(result, InputResult::Continue);
        assert_eq!(input.value, "helo");
        assert_eq!(input.cursor_position, 2);
    }

    #[test]
    fn test_text_input_cursor_movement() {
        let mut input = TextInput::new("Test");
        input.set_value("hello".to_string());
        input.cursor_position = 2;
        
        // Test left arrow
        let result = input.handle_input(KeyCode::Left);
        assert_eq!(result, InputResult::Continue);
        assert_eq!(input.cursor_position, 1);
        
        // Test right arrow
        let result = input.handle_input(KeyCode::Right);
        assert_eq!(result, InputResult::Continue);
        assert_eq!(input.cursor_position, 2);
        
        // Test home
        let result = input.handle_input(KeyCode::Home);
        assert_eq!(result, InputResult::Continue);
        assert_eq!(input.cursor_position, 0);
        
        // Test end
        let result = input.handle_input(KeyCode::End);
        assert_eq!(result, InputResult::Continue);
        assert_eq!(input.cursor_position, 5);
    }

    #[test]
    fn test_text_input_navigation_keys() {
        let mut input = TextInput::new("Test");
        
        // Test tab
        let result = input.handle_input(KeyCode::Tab);
        assert_eq!(result, InputResult::NextField);
        
        // Test shift+tab
        let result = input.handle_input(KeyCode::BackTab);
        assert_eq!(result, InputResult::PreviousField);
        
        // Test enter
        let result = input.handle_input(KeyCode::Enter);
        assert_eq!(result, InputResult::NextField);
        
        // Test escape
        let result = input.handle_input(KeyCode::Esc);
        assert_eq!(result, InputResult::Cancel);
    }

    #[test]
    fn test_text_input_max_length() {
        let mut input = TextInput::new("Test").max_length(3);
        
        // Add characters up to max length
        input.handle_input(KeyCode::Char('a'));
        input.handle_input(KeyCode::Char('b'));
        input.handle_input(KeyCode::Char('c'));
        assert_eq!(input.value, "abc");
        
        // Try to add one more character - should be ignored
        input.handle_input(KeyCode::Char('d'));
        assert_eq!(input.value, "abc");
    }

    #[test]
    fn test_text_input_validation_required() {
        let mut input = TextInput::new("Test").required();
        
        // Should be invalid when empty
        input.validate();
        assert!(!input.is_valid());
        assert!(input.error_message.is_some());
        
        // Should be valid when not empty
        input.set_value("test".to_string());
        assert!(input.is_valid());
        assert!(input.error_message.is_none());
    }

    #[test]
    fn test_text_input_validation_regex() {
        let email_regex = Regex::new(r"^[^@]+@[^@]+\.[^@]+$").unwrap();
        let mut input = TextInput::new("Email").validation_regex(email_regex);
        
        // Should be valid when empty (regex only applies to non-empty values)
        input.validate();
        assert!(input.is_valid());
        
        // Should be invalid with bad email
        input.set_value("invalid-email".to_string());
        assert!(!input.is_valid());
        assert!(input.error_message.is_some());
        
        // Should be valid with good email
        input.set_value("test@example.com".to_string());
        assert!(input.is_valid());
        assert!(input.error_message.is_none());
    }

    #[test]
    fn test_text_area_creation() {
        let textarea = TextArea::new("Test Label");
        assert_eq!(textarea.label, "Test Label");
        assert_eq!(textarea.lines, vec![String::new()]);
        assert_eq!(textarea.cursor_line, 0);
        assert_eq!(textarea.cursor_column, 0);
        assert!(!textarea.focused);
        assert!(!textarea.required);
    }

    #[test]
    fn test_text_area_character_input() {
        let mut textarea = TextArea::new("Test");
        
        let result = textarea.handle_input(KeyCode::Char('a'));
        assert_eq!(result, InputResult::Continue);
        assert_eq!(textarea.lines[0], "a");
        assert_eq!(textarea.cursor_column, 1);
        
        let result = textarea.handle_input(KeyCode::Char('b'));
        assert_eq!(result, InputResult::Continue);
        assert_eq!(textarea.lines[0], "ab");
        assert_eq!(textarea.cursor_column, 2);
    }

    #[test]
    fn test_text_area_enter_key() {
        let mut textarea = TextArea::new("Test");
        textarea.set_value("hello world".to_string());
        textarea.cursor_line = 0;
        textarea.cursor_column = 5; // After "hello"
        
        let result = textarea.handle_input(KeyCode::Enter);
        assert_eq!(result, InputResult::Continue);
        assert_eq!(textarea.lines.len(), 2);
        assert_eq!(textarea.lines[0], "hello");
        assert_eq!(textarea.lines[1], " world");
        assert_eq!(textarea.cursor_line, 1);
        assert_eq!(textarea.cursor_column, 0);
    }

    #[test]
    fn test_text_area_backspace() {
        let mut textarea = TextArea::new("Test");
        textarea.set_value("line1\nline2".to_string());
        textarea.cursor_line = 1;
        textarea.cursor_column = 2; // After "li" in "line2"
        
        let result = textarea.handle_input(KeyCode::Backspace);
        assert_eq!(result, InputResult::Continue);
        assert_eq!(textarea.lines[1], "lne2");
        assert_eq!(textarea.cursor_column, 1);
        
        // Test backspace at beginning of line (should merge with previous line)
        textarea.cursor_column = 0;
        let result = textarea.handle_input(KeyCode::Backspace);
        assert_eq!(result, InputResult::Continue);
        assert_eq!(textarea.lines.len(), 1);
        assert_eq!(textarea.lines[0], "line1lne2");
        assert_eq!(textarea.cursor_line, 0);
        assert_eq!(textarea.cursor_column, 5);
    }

    #[test]
    fn test_text_area_cursor_movement() {
        let mut textarea = TextArea::new("Test");
        textarea.set_value("line1\nline2\nline3".to_string());
        textarea.cursor_line = 1;
        textarea.cursor_column = 2;
        
        // Test up arrow
        let result = textarea.handle_input(KeyCode::Up);
        assert_eq!(result, InputResult::Continue);
        assert_eq!(textarea.cursor_line, 0);
        assert_eq!(textarea.cursor_column, 2);
        
        // Test down arrow
        let result = textarea.handle_input(KeyCode::Down);
        assert_eq!(result, InputResult::Continue);
        assert_eq!(textarea.cursor_line, 1);
        assert_eq!(textarea.cursor_column, 2);
        
        // Test left arrow
        let result = textarea.handle_input(KeyCode::Left);
        assert_eq!(result, InputResult::Continue);
        assert_eq!(textarea.cursor_column, 1);
        
        // Test right arrow
        let result = textarea.handle_input(KeyCode::Right);
        assert_eq!(result, InputResult::Continue);
        assert_eq!(textarea.cursor_column, 2);
    }

    #[test]
    fn test_text_area_max_lines() {
        let mut textarea = TextArea::new("Test").max_lines(2);
        textarea.set_value("line1".to_string());
        
        // Add second line - should work
        textarea.cursor_column = 5;
        let result = textarea.handle_input(KeyCode::Enter);
        assert_eq!(result, InputResult::Continue);
        assert_eq!(textarea.lines.len(), 2);
        
        // Try to add third line - should be ignored
        let result = textarea.handle_input(KeyCode::Enter);
        assert_eq!(result, InputResult::Continue);
        assert_eq!(textarea.lines.len(), 2);
    }

    #[test]
    fn test_text_area_validation_required() {
        let mut textarea = TextArea::new("Test").required();
        
        // Should be invalid when empty
        textarea.validate();
        assert!(!textarea.is_valid());
        assert!(textarea.error_message.is_some());
        
        // Should be valid when not empty
        textarea.set_value("test content".to_string());
        assert!(textarea.is_valid());
        assert!(textarea.error_message.is_none());
    }

    #[test]
    fn test_agent_selector_creation() {
        let selector = AgentSelector::new("Test Label");
        assert_eq!(selector.label, "Test Label");
        assert!(selector.agents.is_empty());
        assert!(selector.selected_index.is_none());
        assert!(!selector.focused);
        assert!(!selector.required);
        assert!(!selector.dropdown_open);
    }

    #[test]
    fn test_agent_selector_set_agents() {
        let mut selector = AgentSelector::new("Test");
        let agents = vec![
            create_test_agent("agent1", "Agent One"),
            create_test_agent("agent2", "Agent Two"),
        ];
        
        selector.set_agents(agents.clone());
        assert_eq!(selector.agents.len(), 2);
        assert_eq!(selector.filtered_indices, vec![0, 1]);
    }

    #[test]
    fn test_agent_selector_selection() {
        let mut selector = AgentSelector::new("Test");
        let agents = vec![
            create_test_agent("agent1", "Agent One"),
            create_test_agent("agent2", "Agent Two"),
        ];
        selector.set_agents(agents);
        
        // Test selecting by ID
        selector.set_selected_agent("agent2");
        assert_eq!(selector.selected_index, Some(1));
        assert_eq!(selector.selected_agent().unwrap().name, "Agent Two");
        
        // Test selecting non-existent agent
        selector.set_selected_agent("nonexistent");
        assert_eq!(selector.selected_index, None);
        assert!(selector.selected_agent().is_none());
    }

    #[test]
    fn test_agent_selector_filtering() {
        let mut selector = AgentSelector::new("Test");
        let agents = vec![
            create_test_agent("agent1", "Database Agent"),
            create_test_agent("agent2", "Web Scraper"),
            create_test_agent("agent3", "Data Processor"),
        ];
        selector.set_agents(agents);
        
        // Initially all agents should be visible
        assert_eq!(selector.filtered_indices, vec![0, 1, 2]);
        
        // Filter by "data" - should match "Database Agent" and "Data Processor"
        selector.filter_text = "data".to_string();
        selector.update_filtered_indices();
        assert_eq!(selector.filtered_indices, vec![0, 2]);
        
        // Filter by "web" - should match "Web Scraper"
        selector.filter_text = "web".to_string();
        selector.update_filtered_indices();
        assert_eq!(selector.filtered_indices, vec![1]);
        
        // Filter by something that doesn't match
        selector.filter_text = "xyz".to_string();
        selector.update_filtered_indices();
        assert!(selector.filtered_indices.is_empty());
    }

    #[test]
    fn test_agent_selector_keyboard_navigation() {
        let mut selector = AgentSelector::new("Test");
        let agents = vec![
            create_test_agent("agent1", "Agent One"),
            create_test_agent("agent2", "Agent Two"),
        ];
        selector.set_agents(agents);
        selector.focused = true;
        
        // Test opening dropdown
        let result = selector.handle_input(KeyCode::Enter);
        assert_eq!(result, InputResult::Continue);
        assert!(selector.dropdown_open);
        
        // Test navigation in dropdown
        let result = selector.handle_input(KeyCode::Down);
        assert_eq!(result, InputResult::Continue);
        assert_eq!(selector.list_state.selected(), Some(1));
        
        let result = selector.handle_input(KeyCode::Up);
        assert_eq!(result, InputResult::Continue);
        assert_eq!(selector.list_state.selected(), Some(0));
        
        // Test selection
        let result = selector.handle_input(KeyCode::Enter);
        assert_eq!(result, InputResult::Continue);
        assert!(!selector.dropdown_open);
        assert_eq!(selector.selected_index, Some(0));
    }

    #[test]
    fn test_agent_selector_validation_required() {
        let mut selector = AgentSelector::new("Test").required();
        let agents = vec![create_test_agent("agent1", "Agent One")];
        selector.set_agents(agents);
        
        // Should be invalid when no agent selected
        selector.validate();
        assert!(!selector.is_valid());
        assert!(selector.error_message.is_some());
        
        // Should be valid when agent selected
        selector.set_selected_agent("agent1");
        assert!(selector.is_valid());
        assert!(selector.error_message.is_none());
    }

    #[test]
    fn test_priority_selector_creation() {
        let selector = PrioritySelector::new("Priority");
        assert_eq!(selector.label, "Priority");
        assert_eq!(*selector.selected_priority(), TaskPriority::Medium);
        assert!(!selector.focused);
        assert!(!selector.dropdown_open);
        assert_eq!(selector.list_state.selected(), Some(1)); // Medium is index 1
    }

    #[test]
    fn test_priority_selector_selection() {
        let mut selector = PrioritySelector::new("Priority");
        
        // Test setting high priority
        selector.set_selected_priority(TaskPriority::High);
        assert_eq!(*selector.selected_priority(), TaskPriority::High);
        assert_eq!(selector.list_state.selected(), Some(0));
        
        // Test setting low priority
        selector.set_selected_priority(TaskPriority::Low);
        assert_eq!(*selector.selected_priority(), TaskPriority::Low);
        assert_eq!(selector.list_state.selected(), Some(2));
    }

    #[test]
    fn test_priority_selector_keyboard_navigation() {
        let mut selector = PrioritySelector::new("Priority");
        selector.focused = true;
        
        // Test opening dropdown
        let result = selector.handle_input(KeyCode::Enter);
        assert_eq!(result, InputResult::Continue);
        assert!(selector.dropdown_open);
        
        // Test navigation in dropdown
        let result = selector.handle_input(KeyCode::Up);
        assert_eq!(result, InputResult::Continue);
        assert_eq!(selector.list_state.selected(), Some(0)); // High
        
        let result = selector.handle_input(KeyCode::Down);
        assert_eq!(result, InputResult::Continue);
        assert_eq!(selector.list_state.selected(), Some(1)); // Medium
        
        let result = selector.handle_input(KeyCode::Down);
        assert_eq!(result, InputResult::Continue);
        assert_eq!(selector.list_state.selected(), Some(2)); // Low
        
        // Test wrapping
        let result = selector.handle_input(KeyCode::Down);
        assert_eq!(result, InputResult::Continue);
        assert_eq!(selector.list_state.selected(), Some(0)); // Back to High
        
        // Test selection
        let result = selector.handle_input(KeyCode::Enter);
        assert_eq!(result, InputResult::Continue);
        assert!(!selector.dropdown_open);
        assert_eq!(*selector.selected_priority(), TaskPriority::High);
    }

    #[test]
    fn test_priority_selector_escape_closes_dropdown() {
        let mut selector = PrioritySelector::new("Priority");
        selector.focused = true;
        selector.dropdown_open = true;
        
        let result = selector.handle_input(KeyCode::Esc);
        assert_eq!(result, InputResult::Continue);
        assert!(!selector.dropdown_open);
    }

    #[test]
    fn test_input_result_enum() {
        // Test that InputResult enum variants are correctly defined
        let results = vec![
            InputResult::Continue,
            InputResult::NextField,
            InputResult::PreviousField,
            InputResult::Submit,
            InputResult::Cancel,
        ];
        
        // Test equality
        assert_eq!(InputResult::Continue, InputResult::Continue);
        assert_ne!(InputResult::Continue, InputResult::NextField);
        
        // Test that we can match on the enum
        for result in results {
            match result {
                InputResult::Continue => {},
                InputResult::NextField => {},
                InputResult::PreviousField => {},
                InputResult::Submit => {},
                InputResult::Cancel => {},
            }
        }
    }

    #[test]
    fn test_text_input_focus_management() {
        let mut input = TextInput::new("Test");
        assert!(!input.focused);
        
        input.set_focused(true);
        assert!(input.focused);
        
        input.set_focused(false);
        assert!(!input.focused);
    }

    #[test]
    fn test_text_area_focus_management() {
        let mut textarea = TextArea::new("Test");
        assert!(!textarea.focused);
        
        textarea.set_focused(true);
        assert!(textarea.focused);
        
        textarea.set_focused(false);
        assert!(!textarea.focused);
    }

    #[test]
    fn test_agent_selector_focus_closes_dropdown() {
        let mut selector = AgentSelector::new("Test");
        selector.dropdown_open = true;
        
        selector.set_focused(false);
        assert!(!selector.dropdown_open);
    }

    #[test]
    fn test_priority_selector_focus_closes_dropdown() {
        let mut selector = PrioritySelector::new("Test");
        selector.dropdown_open = true;
        
        selector.set_focused(false);
        assert!(!selector.dropdown_open);
    }
}