//! Implementation of the git rollback command

use crate::core::git_manager;
use anyhow::Result;
use log::{info, warn};
use std::path::PathBuf;

/// Execute the git rollback command
pub async fn execute(commit_hash: String, confirm: bool) -> Result<()> {
    info!("Rolling back registry to commit: {}", commit_hash);
    
    if !confirm {
        info!("WARNING: This operation will revert the registry to a previous state.");
        info!("All changes made after commit {} will be lost.", commit_hash);
        info!("Use --confirm flag to proceed with the rollback.");
        return Ok(());
    }
    
    // Get the registry path
    let registry_path = PathBuf::from(".nox-registry");
    
    // Verify the commit exists
    let history = git_manager::get_commit_history(&registry_path, 100).await?;
    let commit_exists = history.iter().any(|entry| entry.contains(&commit_hash));
    
    if !commit_exists {
        warn!("Commit {} not found in the recent history", commit_hash);
        info!("Error: Commit {} not found in the recent history.", commit_hash);
        info!("Use 'nox git history' to see available commits.");
        return Ok(());
    }
    
    // Perform the rollback
    git_manager::revert_to_commit(&registry_path, &commit_hash).await?;
    
    info!("Successfully rolled back registry to commit: {}", commit_hash);
    info!("Note: You may need to restart the system for changes to take effect.");
    
    Ok(())
}