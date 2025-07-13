//! Implementation of the git history command

use crate::core::git_manager;
use anyhow::Result;
use log::info;
use std::path::PathBuf;

/// Execute the git history command
pub async fn execute(limit: usize) -> Result<()> {
    info!("Showing git commit history (limit: {})", limit);
    
    // Get the registry path
    let registry_path = PathBuf::from(".nox-registry");
    
    // Get the commit history
    let history = git_manager::get_commit_history(&registry_path, limit).await?;
    
    if history.is_empty() {
        println!("No commit history found.");
        return Ok(());
    }
    
    println!("Commit History:");
    println!("---------------");
    
    for (i, entry) in history.iter().enumerate() {
        println!("{}. {}", i + 1, entry);
    }
    
    println!("\nUse 'nox git rollback <commit-hash> --confirm' to revert to a specific commit.");
    
    Ok(())
}