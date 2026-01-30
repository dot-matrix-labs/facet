use super::{get_storage, LogEntry, LogLevel};
use log::{Metadata, Record};

/// Custom logger that writes to both console and encrypted log file
pub struct FacetLogger {
    console_logger: env_logger::Logger,
}

impl FacetLogger {
    pub fn new() -> Self {
        let console_logger =
            env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug"))
                .format_timestamp_millis()
                .build();

        Self { console_logger }
    }
}

impl log::Log for FacetLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        self.console_logger.enabled(metadata)
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        // Write to console
        self.console_logger.log(record);

        // Write to encrypted log file if storage is initialized
        if let Some(storage) = get_storage() {
            let entry = LogEntry::new(
                LogLevel::from(record.level()),
                "rust".to_string(),
                format!("[{}] {}", record.target(), record.args()),
            );

            // Don't let logging errors crash the app
            if let Err(e) = storage.append_log(entry) {
                eprintln!("Failed to write log to file: {}", e);
            }
        }
    }

    fn flush(&self) {
        self.console_logger.flush();
    }
}

/// Initialize the logger
pub fn init_logger() {
    let logger = FacetLogger::new();
    log::set_boxed_logger(Box::new(logger))
        .map(|()| log::set_max_level(log::LevelFilter::Debug))
        .expect("Failed to initialize logger");
}

/// Log a message from the frontend
pub fn log_frontend_message(level: LogLevel, message: String) -> Result<(), String> {
    if let Some(storage) = get_storage() {
        let entry = LogEntry::new(level, "frontend".to_string(), message);

        storage
            .append_log(entry)
            .map_err(|e| format!("Failed to log frontend message: {}", e))?;

        Ok(())
    } else {
        // If logging not initialized, just write to console
        match level {
            LogLevel::Error => log::error!("[Frontend] {}", message),
            LogLevel::Warn => log::warn!("[Frontend] {}", message),
            LogLevel::Info => log::info!("[Frontend] {}", message),
            LogLevel::Debug => log::debug!("[Frontend] {}", message),
        }
        Ok(())
    }
}
