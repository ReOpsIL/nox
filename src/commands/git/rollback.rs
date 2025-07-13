//! Implementation of the git rollback command

use anyhow::Result;
use log::{info, warn};
use crate::core::git_manager;
use std::path::PathBuf;

/// Execute the git rollback command
pub async fn execute(commit_hash: String, confirm: bool) -> Result<()> {
    info!("Rolling back registry to commit: {}", commit_hash);
    
    if !confirm {
        println!("WARNING: This operation will revert the registry to a previous state.");
        println!("All changes made after commit {} will be lost.", commit_hash);
        println!("Use --confirm flag to proceed with the rollback.");
        return Ok(());
    }
    
    // Get the registry path
    let registry_path = PathBuf::from(".nox-registry");
    
    // Verify the commit exists
    let history = git_manager::get_commit_history(&registry_path, 100).await?;
    let commit_exists = history.iter().any(|entry| entry.contains(&commit_hash));
    
    if !commit_exists {
        warn!("Commit {} not found in the recent history", commit_hash);
        println!("Error: Commit {} not found in the recent history.", commit_hash);
        println!("Use 'nox git history' to see available commits.");
        return Ok(());
    }
    
    // Perform the rollback
    git_manager::revert_to_commit(&registry_path, &commit_hash).await?;
    
    println!("Successfully rolled back registry to commit: {}", commit_hash);
    println!("Note: You may need to restart the system for changes to take effect.");
    
    Ok(())
}