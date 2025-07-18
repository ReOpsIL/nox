//! Test example for non-blocking task execution
//! 
//! This example demonstrates the new non-blocking task execution system
//! that can be properly stopped and terminated.

use anyhow::Result;
use nox::core::{agent_manager, task_manager};
use nox::types::{Agent, Task};
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    env_logger::init();
    
    println!("🚀 Testing Non-Blocking Task Execution");
    
    // Create a test agent
    let agent = Agent::new(
        "Test Agent".to_string(),
        "You are a helpful assistant. Respond with a detailed explanation of the task.".to_string(),
    );
    
    // Save the agent
    agent_manager::add_agent(agent.clone()).await?;
    println!("✅ Agent created: {}", agent.name);
    
    // Create a test task
    let task = Task::new(
        agent.id.clone(),
        "Test Task".to_string(),
        "Write a short poem about artificial intelligence and automation.".to_string(),
    );
    
    // Save the task
    task_manager::add_task(task.clone()).await?;
    println!("✅ Task created: {}", task.title);
    
    // Start the task execution (non-blocking)
    println!("🔄 Starting task execution...");
    task_manager::execute_task(&task.id).await?;
    println!("✅ Task execution started in background");
    
    // Check if the task is running
    let is_running = task_manager::is_task_running(&task.id).await;
    println!("📊 Task running status: {}", is_running);
    
    // Wait a bit to let the task start
    sleep(Duration::from_secs(2)).await;
    
    // Check running tasks
    let running_tasks = task_manager::get_running_task_ids().await;
    println!("📋 Running tasks: {:?}", running_tasks);
    
    // Wait a bit more
    sleep(Duration::from_secs(3)).await;
    
    // Check if task is still running
    let is_still_running = task_manager::is_task_running(&task.id).await;
    println!("📊 Task still running: {}", is_still_running);
    
    if is_still_running {
        println!("🛑 Stopping task execution...");
        task_manager::stop_task_execution(&task.id).await?;
        println!("✅ Task execution stopped");
    }
    
    // Check final task status
    if let Some(final_task) = task_manager::get_task(&task.id).await? {
        println!("📊 Final task status: {}", final_task.status);
        if let Some(response) = final_task.metadata.get("claude_response") {
            println!("📝 Claude response: {}", response);
        }
    }
    
    // Clean up
    task_manager::cleanup_finished_tasks().await?;
    println!("🧹 Cleaned up finished tasks");
    
    println!("✅ Test completed successfully!");
    Ok(())
}