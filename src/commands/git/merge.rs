//! Implementation of the git merge command

use anyhow::Result;
use log::{info, warn};
use crate::core::git_manager;
use std::path::PathBuf;

/// Execute the git merge command
pub async fn execute(branch_name: String) -> Result<()> {
    info!("Merging branch: {}", branch_name);
    
    // Get the registry path
    let registry_path = PathBuf::from(".nox-registry");
    
    // Get the current branch
    let current_branch = git_manager::get_current_branch(&registry_path).await?;
    
    println!("Merging branch '{}' into current branch '{}'...", branch_name, current_branch);
    
    // Perform the merge
    match git_manager::merge_branch(&registry_path, &branch_name).await {
        Ok(_) => {
            println!("Successfully merged branch '{}' into '{}'", branch_name, current_branch);
        },
        Err(e) => {
            warn!("Merge failed: {}", e);
            println!("Error: {}", e);
            
            if e.to_string().contains("Merge conflicts detected") {
                println!("\nTo resolve conflicts manually:");
                println!("1. Use 'git status' to see conflicted files");
                println!("2. Edit the files to resolve conflicts");
                println!("3. Use 'git add <file>' to mark as resolved");
                println!("4. Use 'git commit' to complete the merge");
            }
        }
    }
    
    Ok(())
}