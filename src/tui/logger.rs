use crate::tui::app::{LogEntry, LogStorage};
use chrono::Utc;
use log::{Level, Log, Metadata, Record};
use std::sync::{Arc, Mutex};

/// Custom logger that captures log messages for TUI display
pub struct TuiLogger {
    log_storage: Arc<Mutex<LogStorage>>,
    base_logger: env_logger::Logger,
}

impl TuiLogger {
    pub fn new(log_storage: Arc<Mutex<LogStorage>>) -> Self {
        let base_logger = env_logger::Builder::from_default_env()
            .build();
            
        Self {
            log_storage,
            base_logger,
        }
    }
}

impl Log for TuiLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        self.base_logger.enabled(metadata)
    }

    fn log(&self, record: &Record) {
        // Log to the original env_logger first
        self.base_logger.log(record);
        
        // Only capture logs if they're enabled
        // if !self.enabled(record.metadata()) {
        //     return;
        // }
        
        // Create a log entry for TUI display
        let entry = LogEntry {
            timestamp: Utc::now(),
            level: record.level().to_string(),
            target: record.target().to_string(),
            message: record.args().to_string(),
        };
        
        // Add to TUI log storage
        if let Ok(mut storage) = self.log_storage.lock() {
            storage.add_entry(entry);
        }
    }

    fn flush(&self) {
        self.base_logger.flush();
    }
}

/// Initialize the TUI logger with the given log storage
pub fn init_tui_logger(log_storage: Arc<Mutex<LogStorage>>) -> Result<(), log::SetLoggerError> {
    let logger = TuiLogger::new(log_storage);
    log::set_boxed_logger(Box::new(logger))?;
    log::set_max_level(log::LevelFilter::Debug);
    Ok(())
}