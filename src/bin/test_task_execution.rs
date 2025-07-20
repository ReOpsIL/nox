use anyhow::Result;
use nox::core::{agent_manager, task_manager};
use nox::types::{Agent, Task, TaskStatus, AgentStatus, TaskPriority, ResourceLimits};
use std::time::Duration;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    env_logger::init();
    
    println!("🧪 Testing Task Execution with Claude CLI");
    println!("{}", "=".repeat(50));
    
    // Test task execution with real agent and task
    test_task_execution().await?;
    
    println!("\n✅ Task execution test completed!");
    Ok(())
}

async fn test_task_execution() -> Result<()> {
    println!("Setting up test agent and task...");
    
    // Create a test agent
    let agent = Agent {
        id: Uuid::new_v4().to_string(),
        name: "Test Agent".to_string(),
        system_prompt: "You are a helpful assistant that provides clear, concise answers.".to_string(),
        status: AgentStatus::Active,
        created_at: chrono::Utc::now(),
        resource_limits: ResourceLimits {
            max_memory_mb: 512,
            max_cpu_percent: 50,
        },
    };
    
    // Save the agent
    agent_manager::add_agent(agent.clone()).await?;
    println!("✓ Test agent created: {}", agent.name);
    
    // Create a test task
    let task = Task {
        id: Uuid::new_v4().to_string(),
        title: "Simple Math Question".to_string(),
        description: "What is 5 + 3? Please provide just the number.".to_string(),
        agent_id: agent.id.clone(),
        status: TaskStatus::Todo,
        priority: TaskPriority::High,
        progress: 0,
        created_at: chrono::Utc::now(),
        started_at: None,
        completed_at: None,
        metadata: std::collections::HashMap::new(),
    };
    
    // Save the task
    task_manager::add_task(task.clone()).await?;
    println!("✓ Test task created: {}", task.title);
    
    // Execute the task
    println!("\nExecuting task with Claude CLI...");
    task_manager::execute_task(&task.id).await?;
    println!("✓ Task execution started in background");
    
    // Wait and poll for output
    println!("\nPolling for task output...");
    for i in 0..30 {  // Poll for 30 seconds
        tokio::time::sleep(Duration::from_secs(1)).await;
        
        if let Some(output_lines) = task_manager::poll_task_output(&task.id).await {
            if !output_lines.is_empty() {
                println!("📥 Got {} new output lines:", output_lines.len());
                for line in &output_lines {
                    println!("   {}", line.trim());
                }
            }
        }
        
        // Check task status
        if let Ok(Some(updated_task)) = task_manager::get_task(&task.id).await {
            match updated_task.status {
                TaskStatus::Done => {
                    println!("✅ Task completed successfully!");
                    if let Some(response) = updated_task.metadata.get("claude_response") {
                        println!("📝 Final response: {}", response.chars().take(200).collect::<String>());
                    }
                    break;
                }
                TaskStatus::Failed => {
                    println!("❌ Task failed");
                    break;
                }
                TaskStatus::InProgress => {
                    if i % 5 == 0 {
                        println!("⏳ Task still in progress... ({}s)", i);
                    }
                }
                _ => {}
            }
        }
    }
    
    // Check final status
    if let Ok(Some(final_task)) = task_manager::get_task(&task.id).await {
        println!("\nFinal task status: {:?}", final_task.status);
        println!("Task progress: {}%", final_task.progress);
        
        // Check if log file was created
        let log_file = format!("./logs/task_{}.md", task.id);
        if std::path::Path::new(&log_file).exists() {
            println!("✓ Log file created: {}", log_file);
            
            let log_content = tokio::fs::read_to_string(&log_file).await?;
            let preview = if log_content.len() > 300 {
                format!("{}...", &log_content[..300])
            } else {
                log_content
            };
            println!("📋 Log file preview:\n{}", preview);
        } else {
            println!("❌ Log file not found");
        }
    }
    
    // Clean up
    println!("\nCleaning up...");
    let _ = task_manager::cancel_task(&task.id).await;
    let _ = task_manager::delete_task(&task.id).await;
    let _ = agent_manager::delete_agent(&agent.id).await;
    println!("✓ Cleanup completed");
    
    Ok(())
}