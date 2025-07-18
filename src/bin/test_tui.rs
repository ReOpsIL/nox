use anyhow::Result;
use clap::{Parser, Subcommand};
use nox::testing_fw::{TestConfig, TuiTestFramework};
use nox::testing_fw_extended::ExtendedTuiTestFramework;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "nox-test-tui")]
#[command(about = "NOX TUI Testing Framework - Test keyboard shortcuts across all TUI screens")]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run all TUI tests
    RunAll {
        /// Output format (table, json, csv)
        #[arg(short, long, default_value = "table")]
        format: String,
        
        /// Output file for the report
        #[arg(short, long)]
        output: Option<PathBuf>,
        
        /// Timeout for each test in milliseconds
        #[arg(short, long, default_value = "5000")]
        timeout: u64,
        
        /// Delay between tests in milliseconds
        #[arg(short, long, default_value = "100")]
        delay: u64,
        
        /// Log level (DEBUG, INFO, WARN, ERROR)
        #[arg(short, long, default_value = "INFO")]
        log_level: String,
        
        /// Maximum number of retries for failed tests
        #[arg(short, long, default_value = "3")]
        retries: u32,
    },
    
    /// List all available keyboard shortcuts
    List {
        /// Screen to list shortcuts for (optional, lists all if not specified)
        #[arg(short, long)]
        screen: Option<String>,
        
        /// Output format (table, json)
        #[arg(short, long, default_value = "table")]
        format: String,
    },
    
    /// Test a specific screen
    TestScreen {
        /// Name of the screen to test
        screen: String,
        
        /// Output format (table, json, csv)
        #[arg(short, long, default_value = "table")]
        format: String,
        
        /// Output file for the report
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    
    /// Generate documentation for all keyboard shortcuts
    GenerateDocs {
        /// Output file for the documentation
        #[arg(short, long, default_value = "keyboard_shortcuts.md")]
        output: PathBuf,
        
        /// Include implementation status
        #[arg(short, long)]
        include_status: bool,
    },
    
    /// Run CRUD operations testing
    RunCrud {
        /// Output format (table, json, csv)
        #[arg(short, long, default_value = "table")]
        format: String,
        
        /// Output file for the report
        #[arg(short, long)]
        output: Option<PathBuf>,
        
        /// Timeout for each test in milliseconds
        #[arg(short, long, default_value = "5000")]
        timeout: u64,
        
        /// Delay between tests in milliseconds
        #[arg(short, long, default_value = "500")]
        delay: u64,
        
        /// Log level (DEBUG, INFO, WARN, ERROR)
        #[arg(short, long, default_value = "INFO")]
        log_level: String,
        
        /// Test specific operation type (create, read, update, delete, all)
        #[arg(long, default_value = "all")]
        operation: String,
        
        /// Test specific entity type (agent, task, all)
        #[arg(long, default_value = "all")]
        entity: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    
    let cli = Cli::parse();
    
    match cli.command {
        Commands::RunAll { 
            format, 
            output, 
            timeout, 
            delay, 
            log_level, 
            retries 
        } => {
            let config = TestConfig {
                timeout_ms: timeout,
                delay_between_tests_ms: delay,
                max_retries: retries,
                log_level,
                output_format: format,
            };
            
            println!("🧪 NOX TUI Testing Framework");
            println!("=============================\n");
            println!("Configuration:");
            println!("  Timeout: {}ms", config.timeout_ms);
            println!("  Delay: {}ms", config.delay_between_tests_ms);
            println!("  Max retries: {}", config.max_retries);
            println!("  Log level: {}", config.log_level);
            println!("  Output format: {}\n", config.output_format);
            
            let mut framework = TuiTestFramework::new(config);
            
            // Run all tests
            framework.run_all_tests().await?;
            
            // Generate and display report
            let report = framework.generate_report().await?;
            println!("{}", report);
            
            // Save report to file if specified
            if let Some(output_path) = output {
                framework.save_report_to_file(output_path.to_str().unwrap()).await?;
            } else {
                // Default filename with timestamp
                let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
                let filename = format!("nox_tui_test_report_{}.md", timestamp);
                framework.save_report_to_file(&filename).await?;
            }
        }
        
        Commands::List { screen, format } => {
            list_shortcuts(screen, &format).await?;
        }
        
        Commands::TestScreen { screen, format, output } => {
            test_specific_screen(&screen, &format, output).await?;
        }
        
        Commands::GenerateDocs { output, include_status } => {
            generate_documentation(&output, include_status).await?;
        }
        
        Commands::RunCrud { 
            format, 
            output, 
            timeout, 
            delay, 
            log_level, 
            operation, 
            entity 
        } => {
            let config = TestConfig {
                timeout_ms: timeout,
                delay_between_tests_ms: delay,
                max_retries: 3,
                log_level,
                output_format: format,
            };
            
            println!("🧪 NOX CRUD Operations Testing Framework");
            println!("==========================================\n");
            println!("Configuration:");
            println!("  Timeout: {}ms", config.timeout_ms);
            println!("  Delay: {}ms", config.delay_between_tests_ms);
            println!("  Log level: {}", config.log_level);
            println!("  Output format: {}", config.output_format);
            println!("  Operation filter: {}", operation);
            println!("  Entity filter: {}\n", entity);
            
            let mut framework = ExtendedTuiTestFramework::new(config);
            
            // Run CRUD tests
            framework.run_all_crud_tests().await?;
            
            // Generate and display report
            let report = framework.generate_crud_report().await?;
            println!("{}", report);
            
            // Save report to file if specified
            if let Some(output_path) = output {
                framework.save_crud_report_to_file(output_path.to_str().unwrap()).await?;
            } else {
                // Default filename with timestamp
                let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
                let filename = format!("nox_crud_test_report_{}.md", timestamp);
                framework.save_crud_report_to_file(&filename).await?;
            }
        }
    }
    
    Ok(())
}

async fn list_shortcuts(screen_filter: Option<String>, _format: &str) -> Result<()> {
    let _config = TestConfig::default();
    let _framework = TuiTestFramework::new(_config);
    
    // This would need to be exposed as a public method in the framework
    println!("📋 Available Keyboard Shortcuts\n");
    
    // For now, print a summary message
    if let Some(screen) = screen_filter {
        println!("Listing shortcuts for {} screen...", screen);
        println!("(Implementation pending - shortcuts are defined in the framework)");
    } else {
        println!("Available screens: Dashboard, Agents, Tasks, Execution, Logs");
        println!("Use --screen <name> to see shortcuts for a specific screen");
        println!("(Full listing implementation pending)");
    }
    
    Ok(())
}

async fn test_specific_screen(screen: &str, format: &str, _output: Option<PathBuf>) -> Result<()> {
    let _config = TestConfig {
        output_format: format.to_string(),
        ..TestConfig::default()
    };
    
    println!("🧪 Testing {} Screen", screen);
    println!("====================\n");
    
    // This would need screen-specific testing in the framework
    println!("Screen-specific testing not yet implemented.");
    println!("Use 'run-all' command to test all screens.");
    
    Ok(())
}

async fn generate_documentation(output: &PathBuf, include_status: bool) -> Result<()> {
    println!("📚 Generating Keyboard Shortcuts Documentation");
    println!("============================================\n");
    
    let doc_content = generate_shortcut_documentation(include_status);
    
    std::fs::write(output, doc_content)?;
    println!("Documentation saved to: {}", output.display());
    
    Ok(())
}

fn generate_shortcut_documentation(include_status: bool) -> String {
    let mut doc = String::new();
    
    doc.push_str("# NOX TUI Keyboard Shortcuts Reference\n\n");
    doc.push_str(&format!("Generated: {}\n\n", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")));
    
    doc.push_str("## Global Shortcuts\n\n");
    doc.push_str("These shortcuts work on all screens:\n\n");
    doc.push_str("| Key | Action | Description |\n");
    doc.push_str("|-----|--------|--------------|\n");
    doc.push_str("| `q` / `Q` | Quit | Exit the application |\n");
    doc.push_str("| `?` / `F1` / `h` / `H` | Help | Show help dialog |\n");
    doc.push_str("| `Tab` | Next Screen | Navigate to next screen |\n");
    doc.push_str("| `Shift+Tab` | Previous Screen | Navigate to previous screen |\n");
    doc.push_str("| `1` | Dashboard | Switch to Dashboard screen |\n");
    doc.push_str("| `2` | Agents | Switch to Agents screen |\n");
    doc.push_str("| `3` | Tasks | Switch to Tasks screen |\n");
    doc.push_str("| `4` | Execution | Switch to Execution screen |\n");
    doc.push_str("| `5` | Logs | Switch to Logs screen |\n");
    doc.push_str("| `Up` | Navigate Up | Move selection up in lists |\n");
    doc.push_str("| `Down` | Navigate Down | Move selection down in lists |\n\n");
    
    doc.push_str("## Dashboard Screen\n\n");
    doc.push_str("| Key | Action | Description |\n");
    doc.push_str("|-----|--------|--------------|\n");
    doc.push_str("| `Left` | Navigate Left | Move dashboard focus left |\n");
    doc.push_str("| `Right` | Navigate Right | Move dashboard focus right |\n");
    doc.push_str("| `Enter` | Select Item | Activate selected dashboard item |\n\n");
    
    doc.push_str("## Agents Screen\n\n");
    doc.push_str("| Key | Action | Description |\n");
    doc.push_str("|-----|--------|--------------|\n");
    doc.push_str("| `n` / `N` | New Agent | Open create agent form |\n");
    doc.push_str("| `e` / `E` | Edit Agent | Open edit agent form |\n");
    doc.push_str("| `s` / `S` | Start Agent | Start selected agent |\n");
    doc.push_str("| `t` / `T` | Stop Agent | Show stop agent confirmation |\n");
    doc.push_str("| `d` / `D` | Delete Agent | Show delete agent confirmation |\n");
    doc.push_str("| `r` / `R` | Restart Agent | Show restart agent confirmation |\n");
    doc.push_str("| `Enter` | View Details | Show agent details dialog |\n");
    doc.push_str("| `/` | Search | Activate search mode |\n");
    doc.push_str("| `f` / `F` | Filter | Show filter options |\n\n");
    
    doc.push_str("## Tasks Screen\n\n");
    doc.push_str("| Key | Action | Description |\n");
    doc.push_str("|-----|--------|--------------|\n");
    doc.push_str("| `n` / `N` | New Task | Open create task form |\n");
    doc.push_str("| `e` / `E` | Execute Task | Execute selected task |\n");
    doc.push_str("| `u` / `U` | Update Task | Open edit task form |\n");
    doc.push_str("| `d` / `D` | Delete Task | Show delete task confirmation |\n");
    doc.push_str("| `c` / `C` | Cancel Task | Show cancel task confirmation |\n");
    doc.push_str("| `Enter` | View Details | Show task details dialog |\n");
    doc.push_str("| `a` / `A` | Filter All | Show all tasks |\n");
    doc.push_str("| `r` / `R` | Filter Running | Show running tasks |\n");
    doc.push_str("| `p` / `P` | Filter Pending | Show pending tasks |\n");
    doc.push_str("| `/` | Search | Activate search mode |\n");
    doc.push_str("| `f` / `F` | Filter | Show filter options |\n\n");
    
    doc.push_str("## Execution Screen\n\n");
    doc.push_str("| Key | Action | Description |\n");
    doc.push_str("|-----|--------|--------------|\n");
    doc.push_str("| `Space` | Pause/Resume | Toggle execution pause |\n");
    doc.push_str("| `Delete` | Cancel | Cancel execution |\n");
    doc.push_str("| `Enter` | View Details | Show execution details |\n");
    doc.push_str("| `p` / `P` | Pause | Pause execution |\n");
    doc.push_str("| `r` / `R` | Resume | Resume execution |\n");
    doc.push_str("| `c` / `C` | Cancel | Cancel execution |\n");
    doc.push_str("| `/` | Search | Activate search mode |\n");
    doc.push_str("| `f` / `F` | Filter | Show filter options |\n\n");
    
    doc.push_str("## Logs Screen\n\n");
    doc.push_str("| Key | Action | Description |\n");
    doc.push_str("|-----|--------|--------------|\n");
    doc.push_str("| `f` / `F` | Toggle Filter | Toggle log filter panel |\n");
    doc.push_str("| `c` / `C` | Clear Logs | Show clear logs confirmation |\n");
    doc.push_str("| `s` / `S` | Save Logs | Save logs to file |\n");
    doc.push_str("| `/` | Search | Search in logs |\n");
    doc.push_str("| `Space` | Auto-scroll | Toggle auto-scroll |\n");
    doc.push_str("| `Home` | Jump to Start | Jump to beginning of logs |\n");
    doc.push_str("| `End` | Jump to End | Jump to end of logs |\n");
    doc.push_str("| `Enter` | View Details | Show log entry details |\n");
    doc.push_str("| `r` / `R` | Refresh | Refresh logs |\n");
    doc.push_str("| `a` / `A` | Auto-scroll | Toggle auto-scroll |\n\n");
    
    if include_status {
        doc.push_str("## Implementation Status\n\n");
        doc.push_str("This documentation is generated from the testing framework specifications.\n");
        doc.push_str("Run `nox-test-tui run-all` to test the actual implementation status of these shortcuts.\n\n");
    }
    
    doc.push_str("## Usage Notes\n\n");
    doc.push_str("- Keys are case-insensitive unless otherwise noted\n");
    doc.push_str("- Some actions may require selecting an item first\n");
    doc.push_str("- Global shortcuts work from any screen\n");
    doc.push_str("- Form and dialog interactions may temporarily override these shortcuts\n");
    
    doc
}