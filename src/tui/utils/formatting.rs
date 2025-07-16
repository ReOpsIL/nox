use crate::types::{AgentStatus, TaskStatus, TaskPriority};
use chrono::{DateTime, Utc};

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

pub fn format_memory(mb: u32) -> String {
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