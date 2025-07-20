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
    // Check if we already have agents
    let existing_agents = agent_manager::get_all_agents().await?;
    if !existing_agents.is_empty() {
        info!("Registry already contains {} agents, skipping seeding", existing_agents.len());
        return Ok(());
    }

    info!("Registry is empty, seeding with default Rust programming agents and tasks");

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
            info!("Creating default task '{}' for agent '{}'", task_title, agent.name);
            if let Err(e) = task_manager::add_task(task).await {
                warn!("Failed to create task '{}': {}", task_title, e);
            }
        }
    }

    info!("Default seeding completed: {} agents created with 5 tasks each", agents.len());
    Ok(())
}

/// Create 5 default Rust programming agents
fn create_default_agents() -> Vec<Agent> {
    vec![
        Agent::new(
            "RustCodeReviewer".to_string(),
            "You are a Rust code review specialist. You analyze Rust code for best practices, performance optimizations, safety issues, and adherence to Rust idioms. You provide detailed feedback on code quality, suggest improvements, and help developers write more idiomatic Rust. Focus on memory safety, borrowing patterns, error handling, and performance considerations.".to_string()
        ),
        Agent::new(
            "RustPerformanceOptimizer".to_string(),
            "You are a Rust performance optimization expert. You specialize in analyzing Rust code for performance bottlenecks, memory usage patterns, and CPU optimization opportunities. You help developers write faster, more efficient Rust code by suggesting algorithmic improvements, data structure optimizations, and compiler optimization techniques.".to_string()
        ),
        Agent::new(
            "RustTestingEngineer".to_string(),
            "You are a Rust testing and quality assurance specialist. You focus on writing comprehensive test suites, implementing property-based testing, benchmarking, and setting up CI/CD pipelines for Rust projects. You help ensure code reliability through unit tests, integration tests, and automated testing strategies.".to_string()
        ),
        Agent::new(
            "RustSystemArchitect".to_string(),
            "You are a Rust systems architecture expert. You design scalable, maintainable Rust applications and help with architectural decisions. You specialize in API design, module organization, dependency management, and system integration. You help developers structure large Rust projects and choose appropriate design patterns.".to_string()
        ),
        Agent::new(
            "RustWebDeveloper".to_string(),
            "You are a Rust web development specialist. You work with web frameworks like Actix-web, Axum, Warp, and Rocket. You help build REST APIs, GraphQL endpoints, WebSocket services, and full-stack web applications. You focus on security, scalability, and modern web development practices in Rust.".to_string()
        ),
    ]
}

/// Create 5 default tasks for a specific agent based on their specialty
fn create_default_tasks_for_agent(agent_id: &str, agent_name: &str) -> Vec<Task> {
    match agent_name {
        "RustCodeReviewer" => vec![
            Task::new(
                agent_id.to_string(),
                "Review async/await patterns in web handler".to_string(),
                "Analyze a web service handler that uses async/await and provide feedback on proper error handling, resource management, and async best practices.".to_string()
            ),
            Task::new(
                agent_id.to_string(),
                "Audit unsafe code block usage".to_string(),
                "Review unsafe code blocks in a systems programming project and ensure they are properly justified, documented, and minimize safety risks.".to_string()
            ),
            Task::new(
                agent_id.to_string(),
                "Optimize borrowing and lifetime annotations".to_string(),
                "Examine a complex data structure implementation and suggest improvements to borrowing patterns and lifetime management.".to_string()
            ),
            Task::new(
                agent_id.to_string(),
                "Review error handling strategy".to_string(),
                "Analyze error handling throughout a Rust application and recommend improvements using Result types, custom errors, and the ? operator.".to_string()
            ),
            Task::new(
                agent_id.to_string(),
                "Validate trait implementations and generics".to_string(),
                "Review trait design and generic implementations to ensure they follow Rust best practices and provide good API ergonomics.".to_string()
            ),
        ],
        "RustPerformanceOptimizer" => vec![
            Task::new(
                agent_id.to_string(),
                "Optimize hot path in data processing pipeline".to_string(),
                "Identify and optimize performance bottlenecks in a high-throughput data processing system using profiling and algorithmic improvements.".to_string()
            ),
            Task::new(
                agent_id.to_string(),
                "Reduce memory allocations in parser".to_string(),
                "Analyze a text parser for unnecessary allocations and implement zero-copy parsing techniques where possible.".to_string()
            ),
            Task::new(
                agent_id.to_string(),
                "Implement SIMD optimizations for calculations".to_string(),
                "Add SIMD instructions to speed up mathematical computations in a numerical processing library.".to_string()
            ),
            Task::new(
                agent_id.to_string(),
                "Optimize concurrent data structures".to_string(),
                "Improve performance of thread-safe data structures by reducing lock contention and implementing lock-free algorithms where appropriate.".to_string()
            ),
            Task::new(
                agent_id.to_string(),
                "Profile and optimize async runtime usage".to_string(),
                "Analyze async task scheduling and optimize runtime configuration for better throughput and latency characteristics.".to_string()
            ),
        ],
        "RustTestingEngineer" => vec![
            Task::new(
                agent_id.to_string(),
                "Implement property-based tests for API".to_string(),
                "Create comprehensive property-based tests using quickcheck or proptest to validate API behavior under various input conditions.".to_string()
            ),
            Task::new(
                agent_id.to_string(),
                "Set up integration test suite".to_string(),
                "Design and implement integration tests that validate the entire system behavior including database interactions and external services.".to_string()
            ),
            Task::new(
                agent_id.to_string(),
                "Create performance benchmarks".to_string(),
                "Develop criterion.rs benchmarks to track performance regressions and validate optimization improvements over time.".to_string()
            ),
            Task::new(
                agent_id.to_string(),
                "Implement mutation testing strategy".to_string(),
                "Set up mutation testing to verify the quality and effectiveness of the existing test suite and identify gaps in test coverage.".to_string()
            ),
            Task::new(
                agent_id.to_string(),
                "Configure CI/CD pipeline for Rust project".to_string(),
                "Set up GitHub Actions or similar CI/CD pipeline with proper caching, testing, linting, and automated deployment for a Rust project.".to_string()
            ),
        ],
        "RustSystemArchitect" => vec![
            Task::new(
                agent_id.to_string(),
                "Design microservices architecture".to_string(),
                "Architect a distributed system using Rust microservices with proper service boundaries, communication patterns, and data consistency strategies.".to_string()
            ),
            Task::new(
                agent_id.to_string(),
                "Implement plugin system with dynamic loading".to_string(),
                "Design and implement a safe plugin architecture that allows dynamic loading of Rust modules while maintaining safety guarantees.".to_string()
            ),
            Task::new(
                agent_id.to_string(),
                "Optimize dependency management strategy".to_string(),
                "Analyze and optimize Cargo.toml dependencies, features, and workspace configuration for better build times and binary size.".to_string()
            ),
            Task::new(
                agent_id.to_string(),
                "Design event-driven architecture".to_string(),
                "Implement an event sourcing system with CQRS patterns using Rust, ensuring proper event ordering and consistency guarantees.".to_string()
            ),
            Task::new(
                agent_id.to_string(),
                "Create cross-platform compatibility layer".to_string(),
                "Design abstraction layers for cross-platform functionality while leveraging platform-specific optimizations where beneficial.".to_string()
            ),
        ],
        "RustWebDeveloper" => vec![
            Task::new(
                agent_id.to_string(),
                "Build REST API with Axum framework".to_string(),
                "Develop a production-ready REST API using Axum with proper middleware, authentication, validation, and OpenAPI documentation.".to_string()
            ),
            Task::new(
                agent_id.to_string(),
                "Implement GraphQL server with async-graphql".to_string(),
                "Create a GraphQL API server with subscriptions, federation support, and efficient N+1 query resolution using DataLoader patterns.".to_string()
            ),
            Task::new(
                agent_id.to_string(),
                "Add WebSocket real-time features".to_string(),
                "Implement real-time features using WebSockets with proper connection management, broadcasting, and state synchronization.".to_string()
            ),
            Task::new(
                agent_id.to_string(),
                "Secure web application with authentication".to_string(),
                "Implement JWT-based authentication with refresh tokens, rate limiting, and proper security headers for a web application.".to_string()
            ),
            Task::new(
                agent_id.to_string(),
                "Optimize web server for high concurrency".to_string(),
                "Configure and optimize an Actix-web or Axum server for high-concurrency workloads with proper connection pooling and resource management.".to_string()
            ),
        ],
        _ => {
            // Fallback generic tasks
            vec![
                Task::new(
                    agent_id.to_string(),
                    "Analyze Rust code structure".to_string(),
                    "Review the overall structure and organization of a Rust project and provide recommendations for improvement.".to_string()
                ),
                Task::new(
                    agent_id.to_string(),
                    "Implement Rust best practices".to_string(),
                    "Apply Rust best practices and idioms to improve code quality and maintainability.".to_string()
                ),
                Task::new(
                    agent_id.to_string(),
                    "Optimize Rust performance".to_string(),
                    "Identify and resolve performance bottlenecks in Rust code.".to_string()
                ),
                Task::new(
                    agent_id.to_string(),
                    "Add comprehensive tests".to_string(),
                    "Implement thorough test coverage for Rust modules and functions.".to_string()
                ),
                Task::new(
                    agent_id.to_string(),
                    "Document Rust API".to_string(),
                    "Create comprehensive documentation for Rust APIs and public interfaces.".to_string()
                ),
            ]
        }
    }.into_iter().map(|mut task| {
        // Set appropriate priorities for different types of tasks
        task.priority = match task.title.to_lowercase() {
            title if title.contains("security") || title.contains("unsafe") => TaskPriority::High,
            title if title.contains("performance") || title.contains("optimize") => TaskPriority::High,
            title if title.contains("test") || title.contains("ci") => TaskPriority::Medium,
            _ => TaskPriority::Medium,
        };
        task
    }).collect()
}