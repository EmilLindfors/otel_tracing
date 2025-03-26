//! Facade for logging functionality.
//! 
//! This module provides functions for structured logging.

use crate::{domain::telemetry::LogContext, AttributeValue, LogLevel};
use super::service;

/// Log a message.
pub fn log(context: LogContext) {
    service().log(context)
}

/// Log a message at DEBUG level
pub fn debug(
    message: &str,
    target: Option<&str>,
    attributes: Vec<(String, AttributeValue)>
) {
    log(LogContext {
        level: LogLevel::Debug,
        message: message.to_string(),
        target: target.map(|s| s.to_string()),
        attributes,
    })
}

/// Log a message at INFO level
pub fn info(
    message: &str,
    target: Option<&str>,
    attributes: Vec<(String, AttributeValue)>
) {
    log(LogContext {
        level: LogLevel::Info,
        message: message.to_string(),
        target: target.map(|s| s.to_string()),
        attributes,
    })
}

/// Log a message at WARN level
pub fn warn(
    message: &str,
    target: Option<&str>,
    attributes: Vec<(String, AttributeValue)>
) {
    log(LogContext {
        level: LogLevel::Warn,
        message: message.to_string(),
        target: target.map(|s| s.to_string()),
        attributes,
    })
}

/// Log a message at ERROR level
pub fn error(
    message: &str,
    target: Option<&str>,
    attributes: Vec<(String, AttributeValue)>
) {
    log(LogContext {
        level: LogLevel::Error,
        message: message.to_string(),
        target: target.map(|s| s.to_string()),
        attributes,
    })
}