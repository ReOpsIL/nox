//! Implementation of the git branch command

use crate::core::{config_manager, git_manager};
use anyhow::Result;
use log::{info, warn};

/// Execute the git branch command
pub async fn execute(action: &str, branch_name: Option<String>, force: bool) -> Result<()> {
    // Get the registry path
    let registry_path = config_manager::get_registry_path().await?;
    
    match action {
        "list" => {
            info!("Listing git branches");
            
            // Get the current branch
            let current_branch = git_manager::get_current_branch(&registry_path).await?;
            
            // Get all branches
            let branches = git_manager::list_branches(&registry_path).await?;
            
            if branches.is_empty() {
                info!("No branches found.");
                return Ok(());
            }
            
            info!("Git Branches:");
            info!("-------------");
            
            for branch in branches {
                if branch == current_branch {
                    info!("* {} (current)", branch);
                } else {
                    info!("  {}", branch);
                }
            }
        },
        "create" => {
            let branch_name = branch_name.ok_or_else(|| anyhow::anyhow!("Branch name is required for create action"))?;
            info!("Creating git branch: {}", branch_name);
            
            // Create the branch
            match git_manager::create_branch(&registry_path, &branch_name).await {
                Ok(_) => info!("Branch '{}' created successfully", branch_name),
                Err(e) => {
                    warn!("Failed to create branch: {}", e);
                    info!("Error: {}", e);
                    return Ok(());
                }
            }
        },
        "switch" => {
            let branch_name = branch_name.ok_or_else(|| anyhow::anyhow!("Branch name is required for switch action"))?;
            info!("Switching to git branch: {}", branch_name);
            
            // Switch to the branch
            match git_manager::switch_branch(&registry_path, &branch_name).await {
                Ok(_) => info!("Switched to branch '{}'", branch_name),
                Err(e) => {
                    warn!("Failed to switch branch: {}", e);
                    info!("Error: {}", e);
                    return Ok(());
                }
            }
        },
        "delete" => {
            let branch_name = branch_name.ok_or_else(|| anyhow::anyhow!("Branch name is required for delete action"))?;
            info!("Deleting git branch: {}", branch_name);
            
            // Delete the branch
            match git_manager::delete_branch(&registry_path, &branch_name, force).await {
                Ok(_) => info!("Branch '{}' deleted successfully", branch_name),
                Err(e) => {
                    warn!("Failed to delete branch: {}", e);
                    info!("Error: {}", e);
                    return Ok(());
                }
            }
        },
        _ => {
            info!("Unknown action: {}. Valid actions are: list, create, switch, delete", action);
        }
    }
    
    Ok(())
}