use anyhow::Result;
use tokio::io::{AsyncWriteExt, AsyncBufReadExt, BufReader};
use tokio::process::Command;
use std::process::Stdio;
use serde_json;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🧪 Testing Claude CLI with JSON Output Format");
    println!("{}", "=".repeat(50));
    
    // Test Claude CLI JSON communication
    test_claude_json_communication().await?;
    
    println!("\n✅ JSON test completed!");
    Ok(())
}

async fn test_claude_json_communication() -> Result<()> {
    println!("Testing Claude CLI with --output-format stream-json...");
    
    // Build Claude CLI command with JSON output format
    let mut cmd = Command::new("claude");
    cmd.args([
        "--dangerously-skip-permissions",
        "--output-format", "stream-json",
        "--verbose"
    ]);
    
    // Set environment variable
    cmd.env("CLAUDE_CODE_ENTRYPOINT", "nox-test");
    
    // Configure subprocess I/O
    cmd.stdin(Stdio::piped())
       .stdout(Stdio::piped())
       .stderr(Stdio::piped());
    
    // Spawn the process
    let mut child = cmd.spawn()
        .map_err(|e| anyhow::anyhow!("Failed to spawn Claude CLI: {}", e))?;
    
    println!("✓ Claude CLI process spawned with JSON output format");
    
    // Send a test prompt
    if let Some(mut stdin) = child.stdin.take() {
        let test_prompt = "What is 2 + 2? Please provide just the number.";
        println!("📤 Sending prompt: {}", test_prompt);
        
        stdin.write_all(test_prompt.as_bytes()).await
            .map_err(|e| anyhow::anyhow!("Failed to write to stdin: {}", e))?;
        stdin.shutdown().await
            .map_err(|e| anyhow::anyhow!("Failed to close stdin: {}", e))?;
    }
    
    let mut output_received = false;
    let mut json_buffer = String::new();
    
    // Handle stdout with JSON parsing
    if let Some(stdout) = child.stdout.take() {
        let reader = BufReader::new(stdout);
        let mut lines = reader.lines();
        
        println!("📥 Reading output...");
        let mut line_count = 0;
        
        while let Some(line) = lines.next_line().await
            .map_err(|e| anyhow::anyhow!("Failed to read line: {}", e))? {
            
            line_count += 1;
            if line_count > 100 { // Limit output to prevent infinite loops
                println!("⚠️ Stopping after 100 lines");
                break;
            }
            
            println!("  Raw line {}: {}", line_count, line.trim());
            
            // Try to parse JSON
            if let Some(json_result) = process_json_line(&mut json_buffer, line.clone()) {
                match json_result {
                    Ok(json_data) => {
                        println!("✅ Parsed JSON: {}", serde_json::to_string_pretty(&json_data)?);
                        output_received = true;
                        
                        // Extract content if available
                        if let Some(content) = extract_content_from_json(&json_data) {
                            println!("📝 Extracted content: {}", content);
                        }
                    }
                    Err(e) => {
                        println!("❌ JSON parse error: {}", e);
                    }
                }
            }
            
            // Stop if we got some output and waited a bit
            if line_count > 20 && !line.trim().is_empty() {
                println!("✅ Got sufficient output, stopping");
                break;
            }
        }
    }
    
    // Handle stderr
    if let Some(stderr) = child.stderr.take() {
        let reader = BufReader::new(stderr);
        let mut lines = reader.lines();
        
        println!("📥 Reading stderr...");
        let mut error_lines = 0;
        
        while let Some(line) = lines.next_line().await
            .map_err(|e| anyhow::anyhow!("Failed to read stderr line: {}", e))? {
            
            error_lines += 1;
            if error_lines > 10 { // Limit stderr output
                break;
            }
            
            if !line.trim().is_empty() {
                println!("  Stderr: {}", line.trim());
            }
        }
    }
    
    // Wait for process completion (with timeout)
    let result = tokio::time::timeout(
        std::time::Duration::from_secs(10),
        child.wait()
    ).await;
    
    match result {
        Ok(exit_status) => {
            let status = exit_status.map_err(|e| anyhow::anyhow!("Process wait failed: {}", e))?;
            println!("✓ Process completed with status: {:?}", status);
        }
        Err(_) => {
            println!("⏰ Process timed out, terminating...");
            let _ = child.kill().await;
        }
    }
    
    if output_received {
        println!("✅ JSON communication test PASSED - Received structured output");
    } else {
        println!("⚠️ JSON communication test PARTIAL - No JSON output received");
    }
    
    Ok(())
}

/// Process JSON line similar to reference implementation
fn process_json_line(json_buffer: &mut String, line: String) -> Option<Result<serde_json::Value, serde_json::Error>> {
    // Accumulate JSON across multiple lines
    if line.starts_with('{') && json_buffer.is_empty() {
        *json_buffer = line;
    } else if !json_buffer.is_empty() {
        json_buffer.push('\n');
        json_buffer.push_str(&line);
    } else {
        // Single line JSON or non-JSON line
        return None;
    }
    
    // Try to parse accumulated buffer
    match serde_json::from_str::<serde_json::Value>(json_buffer) {
        Ok(json_data) => {
            json_buffer.clear();
            Some(Ok(json_data))
        }
        Err(e) => {
            // If buffer is too large, clear it to prevent memory issues
            if json_buffer.len() > 1024 * 1024 { // 1MB limit
                json_buffer.clear();
                Some(Err(e))
            } else {
                None // Continue accumulating
            }
        }
    }
}

/// Extract content from Claude CLI JSON message
fn extract_content_from_json(json_data: &serde_json::Value) -> Option<String> {
    // Try different JSON structures that Claude CLI might use
    if let Some(content) = json_data.get("content").and_then(|c| c.as_str()) {
        return Some(content.to_string());
    }
    
    if let Some(text) = json_data.get("text").and_then(|t| t.as_str()) {
        return Some(text.to_string());
    }
    
    if let Some(message) = json_data.get("message").and_then(|m| m.as_str()) {
        return Some(message.to_string());
    }
    
    // Check for nested structures
    if let Some(data) = json_data.get("data") {
        if let Some(content) = data.get("content").and_then(|c| c.as_str()) {
            return Some(content.to_string());
        }
    }
    
    None
}