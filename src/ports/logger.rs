use crate::domain::telemetry::{Level, TelemetryEvent};
use std::error::Error;

/// Logger interface - primary port for logging operations
pub trait Logger: Send + Sync {
    /// Log an event
    fn log(&self, event: TelemetryEvent) -> Result<(), Box<dyn Error + Send + Sync>>;
    
    /// Log a message with the given level
    fn log_with_level(&self, level: Level, message: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.log(TelemetryEvent::new(message, level))
    }
    
    /// Log an info message
    fn info(&self, message: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.log_with_level(Level::Info, message)
    }
    
    /// Log a warning message
    fn warn(&self, message: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.log_with_level(Level::Warn, message)
    }
    
    /// Log an error message
    fn error(&self, message: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.log_with_level(Level::Error, message)
    }
    
    /// Log a debug message
    fn debug(&self, message: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.log_with_level(Level::Debug, message)
    }
    
    /// Log a trace message
    fn trace(&self, message: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.log_with_level(Level::Trace, message)
    }
    
    /// Shut down the logger
    fn shutdown(&self) -> Result<(), Box<dyn Error + Send + Sync>>;
}