//! Seeding module for the Nox agent ecosystem
//!
//! This module handles creating default agents and tasks when the system is initialized
//! with an empty registry.

use crate::core::{agent_manager, task_manager};
use crate::types::{Agent, Task, TaskPriority};
use anyhow::Result;
use log::{info, warn};

/// Seed the system with default agents and tasks if the registry is empty
pub async fn seed_default_data() -> Result<()> {
    // Registry should already be initialized by the caller
    // Check if we already have agents
    let existing_agents = agent_manager::get_all_agents().await?;

    if !existing_agents.is_empty() {
        info!("Registry already contains {} agents, skipping seeding", existing_agents.len());
        return Ok(());
    }

    info!("Registry is empty, seeding with default Code Refactoring Pipeline agents and tasks");

    // Create default agents
    let agents = create_default_agents();
    
    for agent in &agents {
        info!("Creating default agent: {}", agent.name);
        if let Err(e) = agent_manager::add_agent(agent.clone()).await {
            warn!("Failed to create agent '{}': {}", agent.name, e);
            continue;
        }

        // Create tasks for this agent
        let tasks = create_default_tasks_for_agent(&agent.id, &agent.name);
        for task in tasks {
            let task_title = task.title.clone();
            if let Err(e) = task_manager::add_task(task).await {
                warn!("Failed to create task '{}': {}", task_title, e);
            }
        }
    }

    info!("Default seeding completed: {} agents created with 5 tasks each", agents.len());
    Ok(())
}

/// Create 5 default Code Refactoring Pipeline agents
fn create_default_agents() -> Vec<Agent> {
    vec![
        Agent::new(
            "CodeProfiler".to_string(),
            "You are a senior Python code quality analyst. Analyze the following code from the file {file_path}. Your analysis must be objective and focus on identifying code smells, complexity, and areas for improvement.

Evaluate each function based on the following criteria:
- Clarity: How easy is it to understand the function's purpose, inputs, and outputs? (1=very clear, 5=unclear).
- Complexity: Estimate the cyclomatic complexity. High complexity often comes from deep nesting or too many logical branches.
- Issues: Identify specific issues like magic numbers, poor variable names, long methods, or lack of comments.

Based on your analysis, produce a structured report. Do not suggest any code changes.".to_string()
        ),
        Agent::new(
            "RefactoringStrategist".to_string(),
            "You are a software architect specializing in code refactoring. Based on the following CodeProfileReport, create a detailed, step-by-step refactoring plan for each function with a clarity_score greater than 2 or a complexity_score greater than 5.

The plan must consist of specific, actionable steps. For each step, define a clear action type and provide precise details. The goal is to create a plan that a junior developer could follow perfectly.

Important: Do not write the refactored code yourself. Your sole output is the plan.".to_string()
        ),
        Agent::new(
            "CodeRefactorer".to_string(),
            "You are an expert programmer tasked with refactoring code. You will be given a block of original code and a RefactoringPlan.

Follow the plan meticulously. Apply each step in the provided order to the original code. Ensure that the logic and functionality of the code remain unchanged.

Your output must include the original code, the fully refactored code, and a brief summary of the changes you made.".to_string()
        ),
        Agent::new(
            "TestCaseGenerator".to_string(),
            "You are a QA engineer specializing in automated testing. You will be provided with both the original and the refactored code for a function.

Your task is to generate a comprehensive set of unit tests using the Python unittest framework. The tests should:
- Verify the core functionality of the function.
- Cover edge cases (e.g., empty inputs, null values, unexpected types).
- Confirm that the refactored code produces the exact same outputs as the original code for the same inputs.

For each test case, provide the test code and a clear description of what it is verifying.".to_string()
        ),
        Agent::new(
            "DocumentationWriter".to_string(),
            "You are a technical writer responsible for creating developer documentation. Based on the following refactored code, generate a high-quality Python docstring for each function.

The docstring should follow the Google Python Style Guide and include:
- A concise one-line summary.
- A more detailed description of the function's purpose and logic.
- An Args: section detailing each parameter, its type, and its description.
- A Returns: section describing the return value and its type.
- A simple, clear usage example in a Example: section.

The goal is to make the code instantly understandable to a new developer.".to_string()
        ),
    ]
}

/// Create 5 default tasks for a specific agent based on their specialty
fn create_default_tasks_for_agent(agent_id: &str, agent_name: &str) -> Vec<Task> {
    match agent_name {
        "CodeProfiler" => vec![
            Task::new(
                agent_id.to_string(),
                "Profile Python web service handlers".to_string(),
                "Analyze Python web service handlers for code quality, complexity, and maintainability issues. Focus on identifying functions that need refactoring.".to_string()
            ),
            Task::new(
                agent_id.to_string(),
                "Audit data processing functions".to_string(),
                "Review data processing functions for complexity, performance bottlenecks, and code clarity issues.".to_string()
            ),
            Task::new(
                agent_id.to_string(),
                "Analyze utility functions and helpers".to_string(),
                "Examine utility functions and helper methods for code smells, magic numbers, and clarity issues.".to_string()
            ),
            Task::new(
                agent_id.to_string(),
                "Review class methods and structure".to_string(),
                "Analyze class methods for complexity, cohesion, and adherence to single responsibility principle.".to_string()
            ),
            Task::new(
                agent_id.to_string(),
                "Profile algorithm implementations".to_string(),
                "Evaluate algorithm implementations for clarity, efficiency, and maintainability.".to_string()
            ),
        ],
        "RefactoringStrategist" => vec![
            Task::new(
                agent_id.to_string(),
                "Create refactoring plan for complex functions".to_string(),
                "Develop detailed step-by-step refactoring plans for functions with high complexity or low clarity scores.".to_string()
            ),
            Task::new(
                agent_id.to_string(),
                "Plan method extraction strategies".to_string(),
                "Create plans for extracting smaller, focused methods from large, complex functions.".to_string()
            ),
            Task::new(
                agent_id.to_string(),
                "Design variable naming improvements".to_string(),
                "Plan systematic improvements to variable and function naming for better code readability.".to_string()
            ),
            Task::new(
                agent_id.to_string(),
                "Strategy for reducing cyclomatic complexity".to_string(),
                "Develop plans to reduce cyclomatic complexity through guard clauses, early returns, and logical restructuring.".to_string()
            ),
            Task::new(
                agent_id.to_string(),
                "Plan code organization improvements".to_string(),
                "Create strategies for better code organization, including grouping related functionality and improving structure.".to_string()
            ),
        ],
        "CodeRefactorer" => vec![
            Task::new(
                agent_id.to_string(),
                "Execute method extraction refactoring".to_string(),
                "Apply refactoring plans to extract smaller, focused methods from complex functions while preserving functionality.".to_string()
            ),
            Task::new(
                agent_id.to_string(),
                "Implement variable renaming improvements".to_string(),
                "Execute variable and function renaming based on refactoring plans to improve code clarity.".to_string()
            ),
            Task::new(
                agent_id.to_string(),
                "Apply complexity reduction techniques".to_string(),
                "Implement refactoring changes to reduce cyclomatic complexity and improve code flow.".to_string()
            ),
            Task::new(
                agent_id.to_string(),
                "Refactor magic numbers and constants".to_string(),
                "Replace magic numbers with named constants and improve code maintainability.".to_string()
            ),
            Task::new(
                agent_id.to_string(),
                "Restructure conditional logic".to_string(),
                "Apply refactoring to improve conditional logic structure and readability.".to_string()
            ),
        ],
        "TestCaseGenerator" => vec![
            Task::new(
                agent_id.to_string(),
                "Generate unit tests for refactored functions".to_string(),
                "Create comprehensive unit tests that verify the functionality of refactored code matches the original implementation.".to_string()
            ),
            Task::new(
                agent_id.to_string(),
                "Create edge case test scenarios".to_string(),
                "Develop test cases that cover edge cases, boundary conditions, and error scenarios for refactored functions.".to_string()
            ),
            Task::new(
                agent_id.to_string(),
                "Implement regression test suites".to_string(),
                "Create test suites that ensure refactored code produces identical outputs to original code for all inputs.".to_string()
            ),
            Task::new(
                agent_id.to_string(),
                "Generate performance comparison tests".to_string(),
                "Create tests that compare performance characteristics between original and refactored code.".to_string()
            ),
            Task::new(
                agent_id.to_string(),
                "Develop integration test scenarios".to_string(),
                "Generate integration tests that verify refactored functions work correctly within the larger system context.".to_string()
            ),
        ],
        "DocumentationWriter" => vec![
            Task::new(
                agent_id.to_string(),
                "Write Google-style docstrings for refactored functions".to_string(),
                "Create comprehensive docstrings following Google Python Style Guide for all refactored functions.".to_string()
            ),
            Task::new(
                agent_id.to_string(),
                "Document function parameters and return values".to_string(),
                "Create detailed documentation for function parameters, types, and return values to improve code understanding.".to_string()
            ),
            Task::new(
                agent_id.to_string(),
                "Generate usage examples and code snippets".to_string(),
                "Create clear usage examples and code snippets that demonstrate how to use refactored functions.".to_string()
            ),
            Task::new(
                agent_id.to_string(),
                "Document complex algorithms and logic".to_string(),
                "Provide detailed explanations of complex algorithms and business logic within refactored functions.".to_string()
            ),
            Task::new(
                agent_id.to_string(),
                "Create API documentation for modules".to_string(),
                "Generate comprehensive API documentation for modules containing refactored functions.".to_string()
            ),
        ],
        _ => {
            // Fallback generic tasks
            vec![
                Task::new(
                    agent_id.to_string(),
                    "Analyze code quality and structure".to_string(),
                    "Review the overall structure and quality of code and provide recommendations for improvement.".to_string()
                ),
                Task::new(
                    agent_id.to_string(),
                    "Create refactoring strategy".to_string(),
                    "Develop a comprehensive refactoring strategy for improving code maintainability and readability.".to_string()
                ),
                Task::new(
                    agent_id.to_string(),
                    "Execute code improvements".to_string(),
                    "Apply refactoring changes to improve code quality while preserving functionality.".to_string()
                ),
                Task::new(
                    agent_id.to_string(),
                    "Generate comprehensive tests".to_string(),
                    "Create thorough test coverage for refactored code to ensure correctness.".to_string()
                ),
                Task::new(
                    agent_id.to_string(),
                    "Document improved code".to_string(),
                    "Create comprehensive documentation for refactored code and APIs.".to_string()
                ),
            ]
        }
    }.into_iter().map(|mut task| {
        // Set appropriate priorities for different types of tasks
        task.priority = match task.title.to_lowercase() {
            title if title.contains("profile") || title.contains("analyze") => TaskPriority::High,
            title if title.contains("refactor") || title.contains("plan") => TaskPriority::High,
            title if title.contains("test") || title.contains("document") => TaskPriority::Medium,
            _ => TaskPriority::Medium,
        };
        task
    }).collect()
}