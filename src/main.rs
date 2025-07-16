use anyhow::Result;
use clap::{Parser, Subcommand};
use log::info;

mod api;
mod core;
mod commands;
mod types;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize the Nox agent ecosystem and registry
    Init,

    /// Start the Nox agent ecosystem
    Start {
        /// Run in development mode
        #[arg(long)]
        dev: bool,
    },

    /// Stop the Nox agent ecosystem gracefully
    Stop,

    /// Show the current status of all running components
    Status,

    /// Check the system health and report any issues
    Health,

    /// Start the API server for frontend integration
    Serve,

    /// Agent management commands
    Agent {
        #[command(subcommand)]
        subcommand: AgentCommands,
    },

    /// Task management commands
    Task {
        #[command(subcommand)]
        subcommand: TaskCommands,
    },

    /// Git version control commands
    Git {
        #[command(subcommand)]
        subcommand: GitCommands,
    },
}

#[derive(Subcommand)]
enum AgentCommands {
    /// Create a new agent
    Add {
        /// Name of the agent
        name: String,
        /// System prompt for the agent
        prompt: String,
    },

    /// List all registered agents
    List,

    /// Show detailed information about a specific agent
    Show {
        /// Name of the agent
        name: String,
    },

    /// Update an existing agent's system prompt
    Update {
        /// Name of the agent
        name: String,
        /// New system prompt for the agent
        prompt: String,
    },

    /// Remove an agent
    Delete {
        /// Name of the agent
        name: String,
        /// Skip confirmation
        #[arg(long)]
        force: bool,
    },

    /// Start a specific, inactive agent
    Start {
        /// Name of the agent
        name: String,
    },

    /// Stop a specific, active agent
    Stop {
        /// Name of the agent
        name: String,
    },
}

#[derive(Subcommand)]
enum TaskCommands {
    /// Create a new task and assign it to an agent
    Create {
        /// ID of the agent
        agent_id: String,
        /// Title of the task
        title: String,
        /// Description of the task
        desc: String,
    },

    /// List all tasks or tasks for a specific agent
    List {
        /// Optional agent ID to filter tasks
        agent_id: Option<String>,
    },

    /// Update the status of a task
    Update {
        /// ID of the task
        task_id: String,
        /// New status for the task
        #[arg(long)]
        status: String,
    },

    /// Show a summary of task statuses across all agents
    Overview,

    /// Cancel a pending or in-progress task
    Cancel {
        /// ID of the task
        task_id: String,
    },

    /// Execute a task using Claude CLI
    Execute {
        /// ID of the task
        task_id: String,
    },

    /// Show detailed information about a task
    Show {
        /// ID of the task
        task_id: String,
    },
}

#[derive(Subcommand)]
enum GitCommands {
    /// Show commit history
    History {
        /// Maximum number of commits to show
        #[arg(long, default_value = "10")]
        limit: usize,
    },

    /// Roll back to a previous commit
    Rollback {
        /// Commit hash to roll back to
        commit_hash: String,
        /// Confirm the rollback operation
        #[arg(long)]
        confirm: bool,
    },

    /// Branch operations (list, create, switch, delete)
    Branch {
        /// Action to perform (list, create, switch, delete)
        action: String,
        /// Branch name (required for create, switch, delete)
        branch_name: Option<String>,
        /// Force delete even if branch is not fully merged
        #[arg(long)]
        force: bool,
    },

    /// Merge a branch into the current branch
    Merge {
        /// Name of the branch to merge
        branch_name: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Init => {
            info!("Initializing Nox agent ecosystem");
            commands::init::execute().await
        },
        Commands::Start { dev } => {
            info!("Starting Nox agent ecosystem");
            commands::start::execute(dev).await
        },
        Commands::Stop => {
            info!("Stopping Nox agent ecosystem");
            commands::stop::execute().await
        },
        Commands::Status => {
            info!("Showing system status");
            commands::status::execute().await
        },
        Commands::Health => {
            info!("Checking system health");
            commands::health::execute().await
        },
        Commands::Serve  => {
            info!("Starting API server",);
            commands::serve::execute().await
        },
        Commands::Agent { subcommand } => {
            match subcommand {
                AgentCommands::Add { name, prompt } => {
                    info!("Creating new agent: {}", name);
                    commands::agent::add::execute(name, prompt).await
                },
                AgentCommands::List => {
                    info!("Listing all agents");
                    commands::agent::list::execute().await
                },
                AgentCommands::Show { name } => {
                    info!("Showing agent details: {}", name);
                    commands::agent::show::execute(name).await
                },
                AgentCommands::Update { name, prompt } => {
                    info!("Updating agent: {}", name);
                    commands::agent::update::execute(name, prompt).await
                },
                AgentCommands::Delete { name, force } => {
                    info!("Deleting agent: {}", name);
                    commands::agent::delete::execute(name, force).await
                },
                AgentCommands::Start { name } => {
                    info!("Starting agent: {}", name);
                    commands::agent::start::execute(name).await
                },
                AgentCommands::Stop { name } => {
                    info!("Stopping agent: {}", name);
                    commands::agent::stop::execute(name).await
                },
            }
        },
        Commands::Task { subcommand } => {
            match subcommand {
                TaskCommands::Create { agent_id, title, desc } => {
                    info!("Creating new task for agent: {}", agent_id);
                    commands::task::create::execute(agent_id, title, desc).await
                },
                TaskCommands::List { agent_id } => {
                    info!("Listing tasks");
                    commands::task::list::execute(agent_id).await
                },
                TaskCommands::Update { task_id, status } => {
                    info!("Updating task status: {}", task_id);
                    commands::task::update::execute(task_id, status).await
                },
                TaskCommands::Overview => {
                    info!("Showing task overview");
                    commands::task::overview::execute().await
                },
                TaskCommands::Cancel { task_id } => {
                    info!("Cancelling task: {}", task_id);
                    commands::task::cancel::execute(task_id).await
                },
                TaskCommands::Execute { task_id } => {
                    info!("Executing task 3: {}", task_id);
                    commands::task::execute::execute(task_id).await
                },
                TaskCommands::Show { task_id } => {
                    info!("Showing task details: {}", task_id);
                    commands::task::show::execute(task_id).await
                },
            }
        },
        Commands::Git { subcommand } => {
            match subcommand {
                GitCommands::History { limit } => {
                    info!("Showing git history");
                    commands::git::history::execute(limit).await
                },
                GitCommands::Rollback { commit_hash, confirm } => {
                    info!("Rolling back to commit: {}", commit_hash);
                    commands::git::rollback::execute(commit_hash, confirm).await
                },
                GitCommands::Branch { action, branch_name, force } => {
                    info!("Performing git branch operation: {}", action);
                    commands::git::branch::execute(&action, branch_name, force).await
                },
                GitCommands::Merge { branch_name } => {
                    info!("Merging branch: {}", branch_name);
                    commands::git::merge::execute(branch_name).await
                },
            }
        },
    }
}
