//! Git manager module for the Nox agent ecosystem
//! 
//! This module handles Git operations for version control of the registry.

use anyhow::{anyhow, Result};
use log::{info, warn};
use std::path::Path;
use tokio::process::Command as TokioCommand;

/// Initialize a Git repository in the specified directory
pub async fn initialize_repo(repo_path: &Path) -> Result<()> {
    // Check if the directory is already a git repository
    if is_git_repo(repo_path).await? {
        info!("Git repository already initialized at {:?}", repo_path);
        return Ok(());
    }

    info!("Initializing git repository at {:?}", repo_path);

    // Initialize the repository
    let output = TokioCommand::new("git")
        .arg("init")
        .current_dir(repo_path)
        .output()
        .await?;

    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("Failed to initialize git repository: {}", error_msg));
    }

    // Configure git user if not already configured
    if !is_git_user_configured().await? {
        configure_git_user().await?;
    }

    // Create .gitignore file
    create_gitignore(repo_path).await?;

    // Make initial commit
    commit_changes(repo_path, "Initial commit").await?;

    info!("Git repository initialized successfully");
    Ok(())
}

/// Check if a directory is a git repository
async fn is_git_repo(repo_path: &Path) -> Result<bool> {
    let output = TokioCommand::new("git")
        .args(&["rev-parse", "--is-inside-work-tree"])
        .current_dir(repo_path)
        .output()
        .await;

    match output {
        Ok(output) => Ok(output.status.success()),
        Err(_) => Ok(false),
    }
}

/// Check if git user is configured
async fn is_git_user_configured() -> Result<bool> {
    let name_output = TokioCommand::new("git")
        .args(&["config", "--get", "user.name"])
        .output()
        .await?;

    let email_output = TokioCommand::new("git")
        .args(&["config", "--get", "user.email"])
        .output()
        .await?;

    Ok(name_output.status.success() && email_output.status.success())
}

/// Configure git user for the repository
async fn configure_git_user() -> Result<()> {
    // Set default git user for the repository
    let name_output = TokioCommand::new("git")
        .args(&["config", "--global", "user.name", "Nox Agent"])
        .output()
        .await?;

    if !name_output.status.success() {
        let error_msg = String::from_utf8_lossy(&name_output.stderr);
        warn!("Failed to configure git user.name: {}", error_msg);
    }

    let email_output = TokioCommand::new("git")
        .args(&["config", "--global", "user.email", "nox-agent@example.com"])
        .output()
        .await?;

    if !email_output.status.success() {
        let error_msg = String::from_utf8_lossy(&email_output.stderr);
        warn!("Failed to configure git user.email: {}", error_msg);
    }

    Ok(())
}

/// Create a .gitignore file in the repository
async fn create_gitignore(repo_path: &Path) -> Result<()> {
    let gitignore_path = repo_path.join(".gitignore");

    // Default content for .gitignore
    let gitignore_content = r#"# Nox registry gitignore
# Ignore temporary files
*.tmp
*.temp
*.swp
*~

# Ignore logs
*.log

# Ignore OS specific files
.DS_Store
Thumbs.db
"#;

    // Write the .gitignore file
    std::fs::write(&gitignore_path, gitignore_content)?;

    info!("Created .gitignore file at {:?}", gitignore_path);
    Ok(())
}

/// Commit changes to the repository
pub async fn commit_changes(repo_path: &Path, message: &str) -> Result<()> {
    // Stage all changes
    let stage_output = TokioCommand::new("git")
        .args(&["add", "."])
        .current_dir(repo_path)
        .output()
        .await?;

    if !stage_output.status.success() {
        let error_msg = String::from_utf8_lossy(&stage_output.stderr);
        return Err(anyhow!("Failed to stage changes: {}", error_msg));
    }

    // Check if there are changes to commit
    let status_output = TokioCommand::new("git")
        .args(&["status", "--porcelain"])
        .current_dir(repo_path)
        .output()
        .await?;

    if status_output.stdout.is_empty() {
        info!("No changes to commit");
        return Ok(());
    }

    // Add timestamp to commit message for better tracking
    let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC");
    let full_message = format!("{} [{}]", message, timestamp);

    // Commit changes
    let commit_output = TokioCommand::new("git")
        .args(&["commit", "-m", &full_message])
        .current_dir(repo_path)
        .output()
        .await?;

    if !commit_output.status.success() {
        let error_msg = String::from_utf8_lossy(&commit_output.stderr);

        // Check if it's a non-fatal error (like "nothing to commit")
        if error_msg.contains("nothing to commit") {
            info!("Nothing to commit: {}", error_msg.trim());
            return Ok(());
        }

        return Err(anyhow!("Failed to commit changes: {}", error_msg));
    }

    info!("Committed changes with message: {}", full_message);

    // Create a tag for significant changes if specified in the message
    if message.contains("[SIGNIFICANT]") {
        let tag_name = format!("v{}", timestamp.to_string().replace(" ", "-").replace(":", "-"));
        let tag_output = TokioCommand::new("git")
            .args(&["tag", &tag_name])
            .current_dir(repo_path)
            .output()
            .await?;

        if tag_output.status.success() {
            info!("Created tag: {}", tag_name);
        } else {
            let error_msg = String::from_utf8_lossy(&tag_output.stderr);
            warn!("Failed to create tag: {}", error_msg);
        }
    }

    Ok(())
}

/// Get the commit history for the repository
pub async fn get_commit_history(repo_path: &Path, limit: usize) -> Result<Vec<String>> {
    let output = TokioCommand::new("git")
        .args(&["log", "--pretty=format:%h %ad %s", "--date=short", &format!("-{}", limit)])
        .current_dir(repo_path)
        .output()
        .await?;

    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("Failed to get commit history: {}", error_msg));
    }

    let history = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(String::from)
        .collect();

    Ok(history)
}

/// Get the diff for a specific file
#[allow(dead_code)]
pub async fn get_file_diff(repo_path: &Path, file_path: &Path) -> Result<String> {
    let relative_path = file_path.strip_prefix(repo_path)?;

    let output = TokioCommand::new("git")
        .args(&["diff", "--", relative_path.to_str().unwrap()])
        .current_dir(repo_path)
        .output()
        .await?;

    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("Failed to get file diff: {}", error_msg));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Revert changes to a specific commit
pub async fn revert_to_commit(repo_path: &Path, commit_hash: &str) -> Result<()> {
    let output = TokioCommand::new("git")
        .args(&["reset", "--hard", commit_hash])
        .current_dir(repo_path)
        .output()
        .await?;

    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("Failed to revert to commit {}: {}", commit_hash, error_msg));
    }

    info!("Reverted to commit: {}", commit_hash);
    Ok(())
}

/// Create a new branch
pub async fn create_branch(repo_path: &Path, branch_name: &str) -> Result<()> {
    // Check if branch already exists
    let check_output = TokioCommand::new("git")
        .args(&["branch", "--list", branch_name])
        .current_dir(repo_path)
        .output()
        .await?;

    let check_result = String::from_utf8_lossy(&check_output.stdout);
    if check_result.contains(branch_name) {
        return Err(anyhow!("Branch {} already exists", branch_name));
    }

    // Create the branch
    let output = TokioCommand::new("git")
        .args(&["branch", branch_name])
        .current_dir(repo_path)
        .output()
        .await?;

    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("Failed to create branch {}: {}", branch_name, error_msg));
    }

    info!("Created branch: {}", branch_name);
    Ok(())
}

/// Switch to a branch
pub async fn switch_branch(repo_path: &Path, branch_name: &str) -> Result<()> {
    // Check if branch exists
    let check_output = TokioCommand::new("git")
        .args(&["branch", "--list", branch_name])
        .current_dir(repo_path)
        .output()
        .await?;

    let check_result = String::from_utf8_lossy(&check_output.stdout);
    if !check_result.contains(branch_name) {
        return Err(anyhow!("Branch {} does not exist", branch_name));
    }

    // Switch to the branch
    let output = TokioCommand::new("git")
        .args(&["checkout", branch_name])
        .current_dir(repo_path)
        .output()
        .await?;

    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("Failed to switch to branch {}: {}", branch_name, error_msg));
    }

    info!("Switched to branch: {}", branch_name);
    Ok(())
}

/// List all branches
pub async fn list_branches(repo_path: &Path) -> Result<Vec<String>> {
    let output = TokioCommand::new("git")
        .args(&["branch"])
        .current_dir(repo_path)
        .output()
        .await?;

    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("Failed to list branches: {}", error_msg));
    }

    let branches = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|line| line.trim().trim_start_matches('*').trim().to_string())
        .filter(|line| !line.is_empty())
        .collect();

    Ok(branches)
}

/// Get current branch
pub async fn get_current_branch(repo_path: &Path) -> Result<String> {
    let output = TokioCommand::new("git")
        .args(&["branch", "--show-current"])
        .current_dir(repo_path)
        .output()
        .await?;

    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("Failed to get current branch: {}", error_msg));
    }

    let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if branch.is_empty() {
        return Err(anyhow!("Not on a branch (detached HEAD state)"));
    }

    Ok(branch)
}

/// Merge a branch into the current branch
pub async fn merge_branch(repo_path: &Path, branch_name: &str) -> Result<()> {
    // Check if branch exists
    let check_output = TokioCommand::new("git")
        .args(&["branch", "--list", branch_name])
        .current_dir(repo_path)
        .output()
        .await?;

    let check_result = String::from_utf8_lossy(&check_output.stdout);
    if !check_result.contains(branch_name) {
        return Err(anyhow!("Branch {} does not exist", branch_name));
    }

    // Get current branch
    let current_branch = get_current_branch(repo_path).await?;

    // Merge the branch
    let output = TokioCommand::new("git")
        .args(&["merge", branch_name])
        .current_dir(repo_path)
        .output()
        .await?;

    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);

        // Check if there are merge conflicts
        if error_msg.contains("CONFLICT") || error_msg.contains("Automatic merge failed") {
            // Abort the merge
            let _ = TokioCommand::new("git")
                .args(&["merge", "--abort"])
                .current_dir(repo_path)
                .output()
                .await?;

            return Err(anyhow!("Merge conflicts detected. Merge aborted. Please resolve conflicts manually."));
        }

        return Err(anyhow!("Failed to merge branch {} into {}: {}", branch_name, current_branch, error_msg));
    }

    info!("Merged branch {} into {}", branch_name, current_branch);
    Ok(())
}

/// Delete a branch
pub async fn delete_branch(repo_path: &Path, branch_name: &str, force: bool) -> Result<()> {
    // Check if branch exists
    let check_output = TokioCommand::new("git")
        .args(&["branch", "--list", branch_name])
        .current_dir(repo_path)
        .output()
        .await?;

    let check_result = String::from_utf8_lossy(&check_output.stdout);
    if !check_result.contains(branch_name) {
        return Err(anyhow!("Branch {} does not exist", branch_name));
    }

    // Get current branch
    let current_branch = get_current_branch(repo_path).await?;
    if current_branch == branch_name {
        return Err(anyhow!("Cannot delete the current branch. Switch to another branch first."));
    }

    // Delete the branch
    let mut args = vec!["branch"];
    if force {
        args.push("-D");
    } else {
        args.push("-d");
    }
    args.push(branch_name);

    let output = TokioCommand::new("git")
        .args(&args)
        .current_dir(repo_path)
        .output()
        .await?;

    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);

        // Check if branch is not fully merged
        if error_msg.contains("not fully merged") {
            return Err(anyhow!("Branch {} is not fully merged. Use --force to delete anyway.", branch_name));
        }

        return Err(anyhow!("Failed to delete branch {}: {}", branch_name, error_msg));
    }

    info!("Deleted branch: {}", branch_name);
    Ok(())
}
