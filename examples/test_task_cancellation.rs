//! Test example for task cancellation
//! 
//! This example demonstrates how to cancel a running task and properly
//! terminate its execution process.

use anyhow::Result;
use nox::core::{agent_manager, task_manager};
use nox::types::{Agent, Task};
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    env_logger::init();
    
    println!("🚀 Testing Task Cancellation");
    
    // Create a test agent
    let agent = Agent::new(
        "Test Agent".to_string(),
        "You are a helpful assistant. Write a very long and detailed response that takes time to generate.".to_string(),
    );
    
    // Save the agent
    agent_manager::add_agent(agent.clone()).await?;
    println!("✅ Agent created: {}", agent.name);
    
    // Create a test task that should take some time
    let task = Task::new(
        agent.id.clone(),
        "Long Running Task".to_string(),
        "Write a comprehensive 10,000-word essay about the future of artificial intelligence, including detailed examples, case studies, and technical explanations.".to_string(),
    );
    
    // Save the task
    task_manager::add_task(task.clone()).await?;
    println!("✅ Task created: {}", task.title);
    
    // Start the task execution (non-blocking)
    println!("🔄 Starting long-running task execution...");
    task_manager::execute_task(&task.id).await?;
    println!("✅ Task execution started in background");
    
    // Wait for task to start
    sleep(Duration::from_secs(1)).await;
    
    // Check if the task is running
    let is_running = task_manager::is_task_running(&task.id).await;
    println!("📊 Task running status: {}", is_running);
    
    // Wait a bit to let the task run
    println!("⏰ Waiting 3 seconds before cancelling...");
    sleep(Duration::from_secs(3)).await;
    
    // Cancel the task
    println!("🛑 Cancelling task...");
    task_manager::cancel_task(&task.id).await?;
    println!("✅ Task cancelled");
    
    // Check if task is still running
    let is_still_running = task_manager::is_task_running(&task.id).await;
    println!("📊 Task still running after cancellation: {}", is_still_running);
    
    // Check final task status
    if let Some(final_task) = task_manager::get_task(&task.id).await? {
        println!("📊 Final task status: {}", final_task.status);
        
        // Check if there's any partial response
        if let Some(response) = final_task.metadata.get("claude_response") {
            println!("📝 Partial response: {}", response);
        } else {
            println!("📝 No response captured (task was cancelled before completion)");
        }
    }
    
    // Clean up
    task_manager::cleanup_finished_tasks().await?;
    println!("🧹 Cleaned up finished tasks");
    
    println!("✅ Test completed successfully!");
    Ok(())
}