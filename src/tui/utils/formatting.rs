use crate::types::{AgentStatus, TaskStatus, TaskPriority};
use chrono::{DateTime, Utc};
use ratatui::text::{Line, Span};
use ratatui::style::Style;

pub fn format_agent_status(status: &AgentStatus) -> (&'static str, &'static str) {
    match status {
        AgentStatus::Active => ("●", "Active"),
        AgentStatus::Inactive => ("○", "Inactive"),
        AgentStatus::Starting => ("⏸", "Starting"),
        AgentStatus::Stopping => ("⏹", "Stopping"),
        AgentStatus::Error => ("⚠", "Error"),
    }
}

pub fn format_task_status(status: &TaskStatus) -> (&'static str, &'static str) {
    match status {
        TaskStatus::Todo => ("📋", "Todo"),
        TaskStatus::InProgress => ("🔄", "Running"),
        TaskStatus::Done => ("✅", "Done"),
        TaskStatus::Cancelled => ("❌", "Cancelled"),
        TaskStatus::Failed => ("⚠️", "Failed"),
    }
}

pub fn format_task_priority(priority: &TaskPriority) -> (&'static str, &'static str) {
    match priority {
        TaskPriority::High => ("🔴", "High"),
        TaskPriority::Medium => ("🟡", "Medium"),
        TaskPriority::Low => ("🟢", "Low"),
    }
}

pub fn format_duration(seconds: u64) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;

    if hours > 0 {
        format!("{}h {}m", hours, minutes)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, secs)
    } else {
        format!("{}s", secs)
    }
}

pub fn format_memory(mb: usize) -> String {
    if mb >= 1024 {
        format!("{:.1}GB", mb as f32 / 1024.0)
    } else {
        format!("{}MB", mb)
    }
}

pub fn format_percentage(value: f32) -> String {
    format!("{:.0}%", value)
}

pub fn format_datetime(dt: &DateTime<Utc>) -> String {
    dt.format("%Y-%m-%d %H:%M:%S").to_string()
}

pub fn format_progress_bar(progress: u8, width: usize) -> String {
    let filled = (progress as usize * width) / 100;
    let empty = width - filled;
    format!("{}{}", "█".repeat(filled), "░".repeat(empty))
}

/// Wraps text to fit within a specified width, returning a vector of Lines
pub fn wrap_text(text: &str, width: usize, style: Style) -> Vec<Line<'static>> {
    if text.is_empty() {
        return vec![Line::from("")];
    }

    let mut lines = Vec::new();
    
    // Split by existing newlines first
    for paragraph in text.split('\n') {
        if paragraph.is_empty() {
            lines.push(Line::from(""));
            continue;
        }
        
        // Wrap each paragraph
        let words: Vec<&str> = paragraph.split_whitespace().collect();
        let mut current_line = String::new();
        
        for word in words {
            // Check if adding this word would exceed the width
            let potential_length = if current_line.is_empty() {
                word.len()
            } else {
                current_line.len() + 1 + word.len() // +1 for space
            };
            
            if potential_length <= width || current_line.is_empty() {
                // Add word to current line
                if !current_line.is_empty() {
                    current_line.push(' ');
                }
                current_line.push_str(word);
            } else {
                // Start new line with this word
                lines.push(Line::from(Span::styled(current_line.clone(), style)));
                current_line = word.to_string();
            }
        }
        
        // Add the last line if it's not empty
        if !current_line.is_empty() {
            lines.push(Line::from(Span::styled(current_line, style)));
        }
    }
    
    if lines.is_empty() {
        lines.push(Line::from(""));
    }
    
    lines
}