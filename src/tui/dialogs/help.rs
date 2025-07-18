use super::{Dialog, DialogResult, render_utils};
use crate::tui::app::Screen;
use crossterm::event::KeyCode;
use ratatui::prelude::*;
use ratatui::widgets::{List, ListItem, Block, Borders};
use ratatui::text::{Line, Span};

/// Help dialog showing context-sensitive keybindings and instructions
#[derive(Debug, Clone)]
pub struct HelpDialog {
    /// Dialog title
    pub title: String,
    /// Current screen context
    pub screen: Screen,
    /// Current scroll position
    pub scroll_position: usize,
    /// Custom help sections
    pub custom_sections: Vec<HelpSection>,
}

/// A section of help content
#[derive(Debug, Clone)]
pub struct HelpSection {
    /// Section title
    pub title: String,
    /// Keybinding entries
    pub entries: Vec<KeybindingEntry>,
}

/// A single keybinding entry
#[derive(Debug, Clone)]
pub struct KeybindingEntry {
    /// The key combination
    pub key: String,
    /// Description of what it does
    pub description: String,
}

impl KeybindingEntry {
    pub fn new(key: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            description: description.into(),
        }
    }
}

impl HelpSection {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            entries: Vec::new(),
        }
    }
    
    pub fn add_entry(mut self, key: impl Into<String>, description: impl Into<String>) -> Self {
        self.entries.push(KeybindingEntry::new(key, description));
        self
    }
}

impl HelpDialog {
    /// Create a new help dialog for the current screen
    pub fn new(screen: Screen) -> Self {
        Self {
            title: format!("Help - {}", Self::screen_name(&screen)),
            screen,
            scroll_position: 0,
            custom_sections: Vec::new(),
        }
    }
    
    /// Add a custom help section
    pub fn add_section(mut self, section: HelpSection) -> Self {
        self.custom_sections.push(section);
        self
    }
    
    /// Get the display name for a screen
    fn screen_name(screen: &Screen) -> &'static str {
        match screen {
            Screen::Dashboard => "Dashboard",
            Screen::Agents => "Agents",
            Screen::Tasks => "Tasks",
            Screen::Execution => "Execution",
            Screen::Logs => "Logs"
        }
    }
    
    /// Get global keybindings that work in all screens
    fn get_global_keybindings() -> HelpSection {
        HelpSection::new("Global Keys")
            .add_entry("Q", "Quit application")
            .add_entry("?", "Show this help")
            .add_entry("F1", "Show this help")
            .add_entry("Tab", "Next screen")
            .add_entry("Shift+Tab", "Previous screen")
            .add_entry("1", "Go to Dashboard")
            .add_entry("2", "Go to Agents")
            .add_entry("3", "Go to Tasks")
            .add_entry("4", "Go to Execution")
            .add_entry("5", "Go to Logs")
            .add_entry("Esc", "Cancel current operation")
    }
    
    /// Get navigation keybindings
    fn get_navigation_keybindings() -> HelpSection {
        HelpSection::new("Navigation")
            .add_entry("↑/↓", "Navigate list items")
            .add_entry("Page Up/Down", "Scroll page")
            .add_entry("Home", "Go to top")
            .add_entry("End", "Go to bottom")
            .add_entry("Enter", "Select/Activate item")
    }
    
    /// Get screen-specific keybindings
    fn get_screen_keybindings(&self) -> Vec<HelpSection> {
        match self.screen {
            Screen::Dashboard => vec![
                HelpSection::new("Dashboard")
                    .add_entry("R", "Refresh system status")
                    .add_entry("V", "View system details")
                    .add_entry("C", "Clear status messages")
            ],
            Screen::Agents => vec![
                HelpSection::new("Agent Management")
                    .add_entry("N", "Create new agent")
                    .add_entry("E", "Edit selected agent")
                    .add_entry("S", "Start selected agent")
                    .add_entry("T", "Stop selected agent")
                    .add_entry("D", "Delete selected agent")
                    .add_entry("R", "Refresh agent list"),
                HelpSection::new("Agent Operations")
                    .add_entry("V", "View agent details")
                    .add_entry("L", "View agent logs")
                    .add_entry("M", "Monitor agent performance")
                    .add_entry("C", "Copy agent configuration"),
                HelpSection::new("Search & Filter")
                    .add_entry("/", "Search agents")
                    .add_entry("F", "Filter by status")
                    .add_entry("Ctrl+F", "Advanced filters")
                    .add_entry("Ctrl+C", "Clear filters")
            ],
            Screen::Tasks => vec![
                HelpSection::new("Task Management")
                    .add_entry("N", "Create new task")
                    .add_entry("E", "Execute selected task")
                    .add_entry("U", "Update selected task")
                    .add_entry("D", "Delete selected task")
                    .add_entry("C", "Cancel running task")
                    .add_entry("R", "Refresh task list"),
                HelpSection::new("Task Operations")
                    .add_entry("V", "View task details")
                    .add_entry("L", "View task logs")
                    .add_entry("P", "Set task priority")
                    .add_entry("A", "Assign to agent")
                    .add_entry("S", "Schedule task"),
                HelpSection::new("Search & Filter")
                    .add_entry("/", "Search tasks")
                    .add_entry("F", "Filter by status/priority")
                    .add_entry("Ctrl+F", "Advanced filters")
                    .add_entry("Ctrl+C", "Clear filters")
            ],
            Screen::Execution => vec![
                HelpSection::new("Execution Control")
                    .add_entry("Space", "Pause/Resume execution")
                    .add_entry("Del", "Cancel execution")
                    .add_entry("R", "Restart execution")
                    .add_entry("V", "View execution details")
                    .add_entry("L", "View execution logs"),
                HelpSection::new("Monitoring")
                    .add_entry("M", "Monitor performance")
                    .add_entry("T", "Toggle real-time updates")
                    .add_entry("F", "Follow execution progress")
                    .add_entry("S", "Save execution report")
            ],
            Screen::Logs => vec![
                HelpSection::new("Log Viewing")
                    .add_entry("Space", "Toggle log filtering")
                    .add_entry("F", "Filter logs")
                    .add_entry("C", "Clear log display")
                    .add_entry("S", "Save logs to file")
                    .add_entry("R", "Refresh logs"),
                HelpSection::new("Log Navigation")
                    .add_entry("G", "Go to line")
                    .add_entry("J/K", "Scroll up/down")
                    .add_entry("Ctrl+F", "Search in logs")
                    .add_entry("Ctrl+G", "Find next")
                    .add_entry("Ctrl+Shift+G", "Find previous"),
                HelpSection::new("Log Levels")
                    .add_entry("1", "Show only errors")
                    .add_entry("2", "Show warnings and errors")
                    .add_entry("3", "Show info, warnings, and errors")
                    .add_entry("4", "Show debug and all levels")
                    .add_entry("0", "Show all log levels")
            ],
        }
    }
    
    /// Get form-specific keybindings
    fn get_form_keybindings() -> HelpSection {
        HelpSection::new("Form Navigation")
            .add_entry("Tab", "Next field")
            .add_entry("Shift+Tab", "Previous field")
            .add_entry("Enter", "Submit form or next field")
            .add_entry("Esc", "Cancel form")
            .add_entry("Ctrl+S", "Save form")
            .add_entry("Ctrl+R", "Reset form")
    }
    
    /// Get dialog-specific keybindings
    fn get_dialog_keybindings() -> HelpSection {
        HelpSection::new("Dialog Navigation")
            .add_entry("Tab/→", "Next button")
            .add_entry("Shift+Tab/←", "Previous button")
            .add_entry("Enter", "Activate selected button")
            .add_entry("Esc", "Cancel dialog")
            .add_entry("Y", "Yes (confirmation dialogs)")
            .add_entry("N", "No (confirmation dialogs)")
    }
    
    /// Get all help sections
    fn get_all_sections(&self) -> Vec<HelpSection> {
        let mut sections = vec![Self::get_global_keybindings()];
        sections.extend(self.get_screen_keybindings());
        sections.push(Self::get_navigation_keybindings());
        sections.push(Self::get_form_keybindings());
        sections.push(Self::get_dialog_keybindings());
        sections.extend(self.custom_sections.clone());
        sections
    }
    
    /// Get the maximum scroll position
    fn get_max_scroll(&self) -> usize {
        let sections = self.get_all_sections();
        let total_lines = sections.iter()
            .map(|s| s.entries.len() + 2) // +2 for title and spacing
            .sum::<usize>();
        
        total_lines.saturating_sub(10) // Assume 10 visible lines
    }
    
    /// Scroll up
    pub fn scroll_up(&mut self) {
        if self.scroll_position > 0 {
            self.scroll_position -= 1;
        }
    }
    
    /// Scroll down
    pub fn scroll_down(&mut self) {
        let max_scroll = self.get_max_scroll();
        if self.scroll_position < max_scroll {
            self.scroll_position += 1;
        }
    }
    
    /// Scroll to top
    pub fn scroll_to_top(&mut self) {
        self.scroll_position = 0;
    }
    
    /// Scroll to bottom
    pub fn scroll_to_bottom(&mut self) {
        self.scroll_position = self.get_max_scroll();
    }
    
    /// Create a general help dialog
    pub fn general() -> Self {
        Self::new(Screen::Dashboard)
            .add_section(
                HelpSection::new("Getting Started")
                    .add_entry("Tab", "Navigate between screens")
                    .add_entry("?", "Show context-sensitive help")
                    .add_entry("Q", "Quit application")
                    .add_entry("Enter", "Activate selected item")
            )
            .add_section(
                HelpSection::new("Tips")
                    .add_entry("", "Use arrow keys to navigate lists")
                    .add_entry("", "Press ? in any screen for specific help")
                    .add_entry("", "Most operations can be cancelled with Esc")
                    .add_entry("", "Use Ctrl+C to interrupt long operations")
            )
    }
}

impl Dialog for HelpDialog {
    fn render(&self, frame: &mut Frame, area: Rect) {
        render_utils::render_dialog_overlay(frame, area, &self.title, |frame, inner_area| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(0),     // Help content
                    Constraint::Length(2),  // Instructions
                ])
                .split(inner_area);
            
            // Render help content
            let sections = self.get_all_sections();
            let mut help_items = Vec::new();
            
            for section in sections {
                // Add section title
                help_items.push(ListItem::new(section.title.clone())
                    .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)));
                
                // Add keybinding entries
                for entry in section.entries {
                    let key_style = Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD);
                    let desc_style = Style::default().fg(Color::White);
                    
                    if entry.key.is_empty() {
                        // Tip or note without key
                        help_items.push(ListItem::new(format!("  • {}", entry.description))
                            .style(desc_style));
                    } else {
                        // Regular keybinding
                        help_items.push(ListItem::new(Line::from(vec![
                            Span::styled(format!("  {:<12} ", entry.key), key_style),
                            Span::styled(entry.description, desc_style),
                        ])));
                    }
                }
                
                // Add spacing between sections
                help_items.push(ListItem::new(""));
            }
            
            let help_list = List::new(help_items)
                .block(Block::default().borders(Borders::ALL).title("Keybindings"))
                .style(Style::default().fg(Color::White));
            
            frame.render_widget(help_list, chunks[0]);
            
            // Render instructions
            let instructions = if self.get_max_scroll() > 0 {
                "↑/↓ Scroll • Home/End Jump • Esc Close"
            } else {
                "Press Esc to close"
            };
            
            render_utils::render_centered_text(
                frame,
                chunks[1],
                instructions,
                Style::default().fg(Color::Gray),
            );
        }, Some(Style::default().fg(Color::Cyan)));
    }
    
    fn handle_input(&mut self, key: KeyCode) -> DialogResult {
        match key {
            KeyCode::Up => {
                self.scroll_up();
                DialogResult::Continue
            }
            KeyCode::Down => {
                self.scroll_down();
                DialogResult::Continue
            }
            KeyCode::PageUp => {
                for _ in 0..5 {
                    self.scroll_up();
                }
                DialogResult::Continue
            }
            KeyCode::PageDown => {
                for _ in 0..5 {
                    self.scroll_down();
                }
                DialogResult::Continue
            }
            KeyCode::Home => {
                self.scroll_to_top();
                DialogResult::Continue
            }
            KeyCode::End => {
                self.scroll_to_bottom();
                DialogResult::Continue
            }
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => {
                DialogResult::Close
            }
            _ => DialogResult::Continue,
        }
    }
    
    fn title(&self) -> &str {
        &self.title
    }
    
    fn can_cancel(&self) -> bool {
        false // Help dialogs are just closed, not cancelled
    }
    
    fn can_confirm(&self) -> bool {
        false // Help dialogs don't have confirm actions
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_keybinding_entry() {
        let entry = KeybindingEntry::new("Ctrl+C", "Copy");
        assert_eq!(entry.key, "Ctrl+C");
        assert_eq!(entry.description, "Copy");
    }
    
    #[test]
    fn test_help_section() {
        let section = HelpSection::new("Test Section")
            .add_entry("A", "Action A")
            .add_entry("B", "Action B");
        
        assert_eq!(section.title, "Test Section");
        assert_eq!(section.entries.len(), 2);
        assert_eq!(section.entries[0].key, "A");
        assert_eq!(section.entries[0].description, "Action A");
        assert_eq!(section.entries[1].key, "B");
        assert_eq!(section.entries[1].description, "Action B");
    }
    
    #[test]
    fn test_help_dialog_new() {
        let dialog = HelpDialog::new(Screen::Agents);
        assert_eq!(dialog.title, "Help - Agents");
        assert_eq!(dialog.screen, Screen::Agents);
        assert_eq!(dialog.scroll_position, 0);
        assert!(dialog.custom_sections.is_empty());
    }
    
    #[test]
    fn test_help_dialog_add_section() {
        let section = HelpSection::new("Custom Section")
            .add_entry("X", "Custom action");
        
        let dialog = HelpDialog::new(Screen::Dashboard)
            .add_section(section.clone());
        
        assert_eq!(dialog.custom_sections.len(), 1);
        assert_eq!(dialog.custom_sections[0].title, "Custom Section");
    }
    
    #[test]
    fn test_screen_name() {
        assert_eq!(HelpDialog::screen_name(&Screen::Dashboard), "Dashboard");
        assert_eq!(HelpDialog::screen_name(&Screen::Agents), "Agents");
        assert_eq!(HelpDialog::screen_name(&Screen::Tasks), "Tasks");
        assert_eq!(HelpDialog::screen_name(&Screen::Execution), "Execution");
        assert_eq!(HelpDialog::screen_name(&Screen::Logs), "Logs");
    }
    
    #[test]
    fn test_help_dialog_scrolling() {
        let mut dialog = HelpDialog::new(Screen::Agents);
        
        // Test initial state
        assert_eq!(dialog.scroll_position, 0);
        
        // Test scroll down
        dialog.scroll_down();
        assert!(dialog.scroll_position >= 0);
        
        // Test scroll up
        dialog.scroll_position = 5;
        dialog.scroll_up();
        assert_eq!(dialog.scroll_position, 4);
        
        // Test scroll to top
        dialog.scroll_position = 10;
        dialog.scroll_to_top();
        assert_eq!(dialog.scroll_position, 0);
        
        // Test scroll to bottom
        dialog.scroll_to_bottom();
        assert!(dialog.scroll_position >= 0);
    }
    
    #[test]
    fn test_help_dialog_input_handling() {
        let mut dialog = HelpDialog::new(Screen::Agents);
        
        // Test scroll up
        dialog.scroll_position = 5;
        let result = dialog.handle_input(KeyCode::Up);
        assert_eq!(result, DialogResult::Continue);
        assert_eq!(dialog.scroll_position, 4);
        
        // Test scroll down
        let result = dialog.handle_input(KeyCode::Down);
        assert_eq!(result, DialogResult::Continue);
        assert_eq!(dialog.scroll_position, 5);
        
        // Test page up
        let result = dialog.handle_input(KeyCode::PageUp);
        assert_eq!(result, DialogResult::Continue);
        assert_eq!(dialog.scroll_position, 0);
        
        // Test page down
        let result = dialog.handle_input(KeyCode::PageDown);
        assert_eq!(result, DialogResult::Continue);
        assert_eq!(dialog.scroll_position, 5);
        
        // Test home
        dialog.scroll_position = 10;
        let result = dialog.handle_input(KeyCode::Home);
        assert_eq!(result, DialogResult::Continue);
        assert_eq!(dialog.scroll_position, 0);
        
        // Test end
        let result = dialog.handle_input(KeyCode::End);
        assert_eq!(result, DialogResult::Continue);
        assert!(dialog.scroll_position >= 0);
        
        // Test close
        let result = dialog.handle_input(KeyCode::Esc);
        assert_eq!(result, DialogResult::Close);
        
        let result = dialog.handle_input(KeyCode::Char('q'));
        assert_eq!(result, DialogResult::Close);
        
        let result = dialog.handle_input(KeyCode::Char('Q'));
        assert_eq!(result, DialogResult::Close);
        
        // Test other keys
        let result = dialog.handle_input(KeyCode::Char('a'));
        assert_eq!(result, DialogResult::Continue);
    }
    
    #[test]
    fn test_dialog_trait_implementation() {
        let dialog = HelpDialog::new(Screen::Agents);
        
        assert_eq!(dialog.title(), "Help - Agents");
        assert!(!dialog.can_cancel());
        assert!(!dialog.can_confirm());
    }
    
    #[test]
    fn test_general_help_dialog() {
        let dialog = HelpDialog::general();
        assert_eq!(dialog.title, "Help - Dashboard");
        assert!(!dialog.custom_sections.is_empty());
    }
    
    #[test]
    fn test_get_all_sections() {
        let dialog = HelpDialog::new(Screen::Agents);
        let sections = dialog.get_all_sections();
        
        // Should have at least global, screen-specific, navigation, form, and dialog sections
        assert!(sections.len() >= 5);
        
        // First section should be global
        assert_eq!(sections[0].title, "Global Keys");
        
        // Should contain agent-specific sections
        assert!(sections.iter().any(|s| s.title == "Agent Management"));
    }
    
    #[test]
    fn test_screen_specific_keybindings() {
        // Test agents screen
        let dialog = HelpDialog::new(Screen::Agents);
        let sections = dialog.get_screen_keybindings();
        assert!(sections.iter().any(|s| s.title == "Agent Management"));
        
        // Test tasks screen
        let dialog = HelpDialog::new(Screen::Tasks);
        let sections = dialog.get_screen_keybindings();
        assert!(sections.iter().any(|s| s.title == "Task Management"));
        
        // Test dashboard screen
        let dialog = HelpDialog::new(Screen::Dashboard);
        let sections = dialog.get_screen_keybindings();
        assert!(sections.iter().any(|s| s.title == "Dashboard"));
    }
}