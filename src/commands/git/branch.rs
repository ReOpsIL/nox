//! Implementation of the git branch command

use crate::core::git_manager;
use anyhow::Result;
use log::{info, warn};
use std::path::PathBuf;

/// Execute the git branch command
pub async fn execute(action: &str, branch_name: Option<String>, force: bool) -> Result<()> {
    // Get the registry path
    let registry_path = PathBuf::from(".nox-registry");
    
    match action {
        "list" => {
            info!("Listing git branches");
            
            // Get the current branch
            let current_branch = git_manager::get_current_branch(&registry_path).await?;
            
            // Get all branches
            let branches = git_manager::list_branches(&registry_path).await?;
            
            if branches.is_empty() {
                println!("No branches found.");
                return Ok(());
            }
            
            println!("Git Branches:");
            println!("-------------");
            
            for branch in branches {
                if branch == current_branch {
                    println!("* {} (current)", branch);
                } else {
                    println!("  {}", branch);
                }
            }
        },
        "create" => {
            let branch_name = branch_name.ok_or_else(|| anyhow::anyhow!("Branch name is required for create action"))?;
            info!("Creating git branch: {}", branch_name);
            
            // Create the branch
            match git_manager::create_branch(&registry_path, &branch_name).await {
                Ok(_) => println!("Branch '{}' created successfully", branch_name),
                Err(e) => {
                    warn!("Failed to create branch: {}", e);
                    println!("Error: {}", e);
                    return Ok(());
                }
            }
        },
        "switch" => {
            let branch_name = branch_name.ok_or_else(|| anyhow::anyhow!("Branch name is required for switch action"))?;
            info!("Switching to git branch: {}", branch_name);
            
            // Switch to the branch
            match git_manager::switch_branch(&registry_path, &branch_name).await {
                Ok(_) => println!("Switched to branch '{}'", branch_name),
                Err(e) => {
                    warn!("Failed to switch branch: {}", e);
                    println!("Error: {}", e);
                    return Ok(());
                }
            }
        },
        "delete" => {
            let branch_name = branch_name.ok_or_else(|| anyhow::anyhow!("Branch name is required for delete action"))?;
            info!("Deleting git branch: {}", branch_name);
            
            // Delete the branch
            match git_manager::delete_branch(&registry_path, &branch_name, force).await {
                Ok(_) => println!("Branch '{}' deleted successfully", branch_name),
                Err(e) => {
                    warn!("Failed to delete branch: {}", e);
                    println!("Error: {}", e);
                    return Ok(());
                }
            }
        },
        _ => {
            println!("Unknown action: {}. Valid actions are: list, create, switch, delete", action);
        }
    }
    
    Ok(())
}