//! Implementation of the git history command

use crate::core::{config_manager, git_manager};
use anyhow::Result;
use log::info;

/// Execute the git history command
pub async fn execute(limit: usize) -> Result<()> {
    info!("Showing git commit history (limit: {})", limit);
    
    // Get the registry path
    let registry_path = config_manager::get_registry_path().await?;
    
    // Get the commit history
    let history = git_manager::get_commit_history(&registry_path, limit).await?;
    
    if history.is_empty() {
        info!("No commit history found.");
        return Ok(());
    }
    
    info!("Commit History:");
    info!("---------------");
    
    for (i, entry) in history.iter().enumerate() {
        info!("{}. {}", i + 1, entry);
    }
    
    info!("\nUse 'nox git rollback <commit-hash> --confirm' to revert to a specific commit.");
    
    Ok(())
}