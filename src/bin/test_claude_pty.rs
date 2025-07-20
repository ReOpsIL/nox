use anyhow::Result;
use log::{error, info};
use portable_pty::{CommandBuilder, PtySize};
use std::io::{Read, Write};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc;
use tokio::time::timeout;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    env_logger::init();
    
    println!("🧪 Testing Claude CLI PTY Communication");
    println!("{}", "=".repeat(50));
    
    // Test 1: Basic Claude CLI PTY Setup
    println!("\n1️⃣ Testing PTY setup and Claude CLI spawn...");
    test_pty_setup().await?;
    
    // Test 2: Simple Input/Output Communication
    println!("\n2️⃣ Testing basic input/output communication...");
    test_basic_communication().await?;
    
    // Test 3: Streaming Communication with Channels
    println!("\n3️⃣ Testing streaming communication with channels...");
    test_streaming_communication().await?;
    
    // Test 4: Full Integration Test (simulating task execution)
    println!("\n4️⃣ Testing full task execution simulation...");
    test_full_integration().await?;
    
    println!("\n✅ All tests completed successfully!");
    println!("{}", "=".repeat(50));
    
    Ok(())
}

async fn test_pty_setup() -> Result<()> {
    let pty_size = PtySize {
        rows: 24,
        cols: 80,
        pixel_width: 0,
        pixel_height: 0,
    };
    
    let pty_system = portable_pty::native_pty_system();
    let pty_pair = pty_system.openpty(pty_size)?;
    
    let mut cmd = CommandBuilder::new("claude");
    cmd.arg("--dangerously-skip-permissions");
    
    let child = pty_pair.slave.spawn_command(cmd)?;
    
    println!("  ✓ PTY created successfully");
    println!("  ✓ Claude CLI process spawned (PID: {})", child.process_id().unwrap_or(0));
    
    // Clean up
    drop(child);
    
    Ok(())
}

async fn test_basic_communication() -> Result<()> {
    let pty_size = PtySize {
        rows: 24,
        cols: 80,
        pixel_width: 0,
        pixel_height: 0,
    };
    
    let pty_system = portable_pty::native_pty_system();
    let pty_pair = pty_system.openpty(pty_size)?;
    
    let mut cmd = CommandBuilder::new("claude");
    cmd.arg("--dangerously-skip-permissions");
    
    let mut child = pty_pair.slave.spawn_command(cmd)?;
    let master = pty_pair.master;
    
    // Get reader and writer
    let master_reader = master.try_clone_reader()?;
    let mut master_writer = master.take_writer()?;
    
    // Send a simple test message
    let test_message = "Hello Claude! Please respond with 'Hello from Claude'.\n";
    println!("  📤 Sending: {}", test_message.trim());
    
    master_writer.write_all(test_message.as_bytes())?;
    master_writer.flush()?;
    
    // Read response with timeout
    let response = tokio::task::spawn_blocking(move || -> Result<String> {
        let mut reader = master_reader;
        let mut buffer = [0u8; 4096];
        let mut accumulated = String::new();
        
        // Read for up to 10 seconds
        for _ in 0..100 {
            match reader.read(&mut buffer) {
                Ok(0) => break, // EOF
                Ok(n) => {
                    let chunk = String::from_utf8_lossy(&buffer[..n]);
                    accumulated.push_str(&chunk);
                    println!("  📥 Received chunk: {}", chunk.trim());
                    
                    // Check if we got a response
                    if accumulated.len() > 50 {
                        break;
                    }
                }
                Err(e) => {
                    println!("  ⚠️ Read error: {}", e);
                    break;
                }
            }
            std::thread::sleep(Duration::from_millis(100));
        }
        
        Ok(accumulated)
    }).await??;
    
    println!("  ✓ Received response ({} chars)", response.len());
    if !response.is_empty() {
        println!("  📝 Response preview: {}", response.chars().take(100).collect::<String>());
    }
    
    // Clean up
    let _ = child.kill();
    let _ = child.wait();
    
    Ok(())
}

async fn test_streaming_communication() -> Result<()> {
    let pty_size = PtySize {
        rows: 24,
        cols: 80,
        pixel_width: 0,
        pixel_height: 0,
    };
    
    let pty_system = portable_pty::native_pty_system();
    let pty_pair = pty_system.openpty(pty_size)?;
    
    let mut cmd = CommandBuilder::new("claude");
    cmd.arg("--dangerously-skip-permissions");
    
    let mut child = pty_pair.slave.spawn_command(cmd)?;
    let master = pty_pair.master;
    
    // Create channels
    let (input_tx, mut input_rx) = mpsc::channel::<String>(100);
    let (output_tx, mut output_rx) = mpsc::channel::<String>(1000);
    
    // Get reader and writer
    let master_reader = master.try_clone_reader()?;
    let master_writer = master.take_writer()?;
    let master_writer = Arc::new(std::sync::Mutex::new(master_writer));
    
    let cancellation_flag = Arc::new(AtomicBool::new(false));
    
    // Spawn input handler (channel to PTY)
    let writer_handle = master_writer.clone();
    let cancellation_flag_input = Arc::clone(&cancellation_flag);
    let input_handle = tokio::task::spawn_blocking(move || {
        loop {
            if cancellation_flag_input.load(Ordering::Relaxed) {
                break;
            }
            
            match input_rx.blocking_recv() {
                Some(input) => {
                    if let Ok(mut writer) = writer_handle.lock() {
                        if let Err(e) = writer.write_all(input.as_bytes()) {
                            error!("Failed to write to PTY: {}", e);
                            break;
                        }
                        if let Err(e) = writer.flush() {
                            error!("Failed to flush PTY writer: {}", e);
                            break;
                        }
                        println!("  📤 Sent: {}", input.trim());
                    }
                }
                None => break,
            }
        }
    });
    
    // Spawn output handler (PTY to channel)
    let cancellation_flag_output = Arc::clone(&cancellation_flag);
    let output_tx_clone = output_tx.clone();
    let output_handle = tokio::task::spawn_blocking(move || {
        let mut reader = master_reader;
        let mut buffer = [0u8; 4096];
        
        loop {
            if cancellation_flag_output.load(Ordering::Relaxed) {
                break;
            }
            
            match reader.read(&mut buffer) {
                Ok(0) => {
                    info!("PTY reader reached EOF");
                    break;
                }
                Ok(n) => {
                    let output = String::from_utf8_lossy(&buffer[..n]).to_string();
                    println!("  📥 Received: {}", output.trim());
                    
                    if let Err(_) = output_tx_clone.blocking_send(output) {
                        break; // Channel closed
                    }
                }
                Err(e) => {
                    error!("Failed to read from PTY: {}", e);
                    break;
                }
            }
        }
    });
    
    // Send test messages through channels
    let test_messages = vec![
        "What is 2 + 2?\n".to_string(),
        "Please explain what you are.\n".to_string(),
    ];
    
    for msg in test_messages {
        input_tx.send(msg).await?;
        
        // Wait for some output
        let response_timeout = timeout(Duration::from_secs(5), output_rx.recv()).await;
        match response_timeout {
            Ok(Some(response)) => {
                println!("  ✓ Got response through channel: {}", response.chars().take(50).collect::<String>());
            }
            Ok(None) => {
                println!("  ⚠️ Output channel closed");
            }
            Err(_) => {
                println!("  ⚠️ Timeout waiting for response");
            }
        }
    }
    
    // Clean up
    cancellation_flag.store(true, Ordering::Relaxed);
    drop(input_tx);
    drop(output_tx);
    
    // Wait a bit for cleanup
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    input_handle.abort();
    output_handle.abort();
    
    let _ = child.kill();
    let _ = child.wait();
    
    println!("  ✓ Streaming communication test completed");
    
    Ok(())
}

async fn test_full_integration() -> Result<()> {
    // This simulates the full task execution flow
    println!("  🔄 Simulating full task execution...");
    
    // Create test log directory
    let log_dir = std::path::Path::new("./test_logs");
    if !log_dir.exists() {
        std::fs::create_dir_all(log_dir)?;
    }
    let log_file_path = log_dir.join("test_task.md");
    
    // Setup PTY
    let pty_size = PtySize {
        rows: 24,
        cols: 80,
        pixel_width: 0,
        pixel_height: 0,
    };
    
    let pty_system = portable_pty::native_pty_system();
    let pty_pair = pty_system.openpty(pty_size)?;
    
    let mut cmd = CommandBuilder::new("claude");
    cmd.arg("--dangerously-skip-permissions");
    
    let mut child = pty_pair.slave.spawn_command(cmd)?;
    let master = pty_pair.master;
    
    // Create log file
    let mut log_file = tokio::fs::File::create(&log_file_path).await?;
    let log_header = format!("# Test Task Execution Log\n\n**Started:** {}\n\n", 
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"));
    log_file.write_all(log_header.as_bytes()).await?;
    
    // Setup I/O
    let master_reader = master.try_clone_reader()?;
    let master_writer = master.take_writer()?;
    let master_writer = Arc::new(std::sync::Mutex::new(master_writer));
    
    // Send initial task message
    let task_message = "Create a simple Python script that prints 'Hello from automated task!'";
    println!("  📝 Task: {}", task_message);
    
    {
        let mut writer = master_writer.lock().unwrap();
        writer.write_all(format!("{}\n", task_message).as_bytes())?;
        writer.flush()?;
    }
    
    // Log the input
    log_file.write_all(format!("**Input:** {}\n\n", task_message).as_bytes()).await?;
    log_file.write_all("**Output:**\n".as_bytes()).await?;
    
    // Collect output for a reasonable time
    let log_file_path_clone = log_file_path.clone();
    let output_handle = tokio::task::spawn_blocking(move || -> Result<String> {
        let mut reader = master_reader;
        let mut buffer = [0u8; 1024];
        let mut accumulated = String::new();
        
        // Read for up to 30 seconds or until we get substantial output
        for i in 0..300 {
            match reader.read(&mut buffer) {
                Ok(0) => {
                    println!("  📄 PTY EOF reached");
                    break;
                }
                Ok(n) => {
                    let chunk = String::from_utf8_lossy(&buffer[..n]).to_string();
                    accumulated.push_str(&chunk);
                    println!("  📥 Output chunk: {}", chunk.trim());
                    
                    // Sync log writing to avoid async issues in blocking context
                    if let Ok(mut file) = std::fs::OpenOptions::new()
                        .append(true)
                        .open(&log_file_path_clone)
                    {
                        let _ = file.write_all(&buffer[..n]);
                    }
                    
                    // Stop if we have a reasonable amount of output
                    if accumulated.len() > 500 || (accumulated.len() > 100 && i > 100) {
                        println!("  ✅ Got substantial output, stopping collection");
                        break;
                    }
                }
                Err(e) => {
                    println!("  ⚠️ Read error: {}", e);
                    break;
                }
            }
            
            std::thread::sleep(Duration::from_millis(100));
        }
        
        Ok(accumulated)
    });
    
    // Wait for output
    let output_result = timeout(Duration::from_secs(30), output_handle).await;
    
    match output_result {
        Ok(Ok(output)) => {
            println!("  ✅ Collected {} characters of output", output.len());
            if !output.is_empty() {
                println!("  📄 Output preview: {}", output.chars().take(200).collect::<String>());
            }
        }
        Ok(Err(e)) => {
            println!("  ❌ Error collecting output: {}", e);
        }
        Err(_) => {
            println!("  ⏰ Timeout collecting output");
        }
    }
    
    // Check log file
    if log_file_path.exists() {
        let log_size = std::fs::metadata(&log_file_path)?.len();
        println!("  📋 Log file created: {} ({} bytes)", log_file_path.display(), log_size);
        
        if log_size > 100 {
            let log_content = tokio::fs::read_to_string(&log_file_path).await?;
            println!("  📖 Log preview: {}", log_content.chars().take(300).collect::<String>());
        }
    }
    
    // Clean up
    let _ = child.kill();
    let _ = child.wait();
    
    println!("  ✓ Full integration test completed");
    
    Ok(())
}