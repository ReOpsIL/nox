//! Nox Autonomous Agent Ecosystem Library
//! 
//! This library provides the core functionality for the Nox autonomous agent ecosystem,
//! including agent management, task management, Claude CLI integration, and more.

pub mod types;
pub mod core;
pub mod api;
pub mod commands;
pub mod tui;

pub use core::{
    agent_manager,
    claude_process_manager,
    config_manager,
    git_manager,
    registry_manager,
    task_manager,
};
// Re-export commonly used types and functions
pub use types::*;
