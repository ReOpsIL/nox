use anyhow::Result;
use portable_pty::{CommandBuilder, PtySize};
use std::io::{Read, Write};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🧪 Simple Claude CLI PTY Test");
    println!("{}", "=".repeat(40));
    
    // Test Claude CLI PTY communication
    test_claude_communication().await?;
    
    println!("\n✅ Test completed!");
    Ok(())
}

async fn test_claude_communication() -> Result<()> {
    println!("Testing Claude CLI PTY communication...");
    
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
    
    println!("✓ PTY created and Claude CLI spawned");
    
    // Get reader and writer
    let master_reader = master.try_clone_reader()?;
    let mut master_writer = master.take_writer()?;
    
    // Send test message
    let test_message = "What is 2 + 2? Please provide a brief answer.\r\n";
    println!("📤 Sending: {}", test_message.trim());
    
    master_writer.write_all(test_message.as_bytes())?;
    master_writer.flush()?;
    
    // Read response in blocking task
    let response = tokio::task::spawn_blocking(move || -> Result<String> {
        let mut reader = master_reader;
        let mut buffer = [0u8; 2048];
        let mut accumulated = String::new();
        
        // Read for up to 15 seconds
        for i in 0..150 {
            match reader.read(&mut buffer) {
                Ok(0) => {
                    println!("📄 PTY EOF reached");
                    break;
                }
                Ok(n) => {
                    let chunk = String::from_utf8_lossy(&buffer[..n]);
                    accumulated.push_str(&chunk);
                    
                    if !chunk.trim().is_empty() {
                        println!("📥 Received: {}", chunk.trim());
                    }
                    
                    // Stop if we have enough output
                    if accumulated.len() > 100 || (accumulated.len() > 20 && i > 50) {
                        println!("✅ Got sufficient output");
                        break;
                    }
                }
                Err(e) => {
                    println!("⚠️ Read error: {}", e);
                    break;
                }
            }
            
            std::thread::sleep(Duration::from_millis(100));
        }
        
        Ok(accumulated)
    }).await??;
    
    println!("✓ Received {} characters total", response.len());
    
    if !response.is_empty() {
        let preview = if response.len() > 200 {
            format!("{}...", &response[..200])
        } else {
            response.clone()
        };
        println!("📝 Response preview:\n{}", preview);
    } else {
        println!("❌ No response received from Claude CLI");
    }
    
    // Test log creation
    println!("\nTesting log file creation...");
    let log_dir = std::path::Path::new("./test_logs");
    if !log_dir.exists() {
        std::fs::create_dir_all(log_dir)?;
    }
    
    let log_file_path = log_dir.join("simple_test.md");
    std::fs::write(&log_file_path, format!(
        "# Simple Claude Test Log\n\n**Input:** {}\n\n**Output:**\n{}\n",
        test_message.trim(),
        response
    ))?;
    
    println!("✓ Log file created: {}", log_file_path.display());
    
    // Clean up
    let _ = child.kill();
    let _ = child.wait();
    
    // Check if we got any meaningful response
    if response.len() > 10 {
        println!("✅ Communication test PASSED - Claude CLI responded");
    } else {
        println!("❌ Communication test FAILED - No meaningful response");
    }
    
    Ok(())
}