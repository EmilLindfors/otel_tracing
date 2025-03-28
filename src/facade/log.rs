//! Facade for logging functionality.
//! 
//! This module provides functions for structured logging.

use std::collections::HashMap;

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
        timestamp: None,
        message: message.to_string(),
        target: target.map(|s| s.to_string()),
        attributes: HashMap::from_iter(attributes),
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
        timestamp: None,
        message: message.to_string(),
        target: target.map(|s| s.to_string()),
        attributes: HashMap::from_iter(attributes),
    })
}

/// Log a message at WARN level
pub fn warn(
    message: &str,
    target: Option<&str>,
    attributes: Vec<(String, AttributeValue)>
) {
    log(LogContext {
        timestamp: None,
        level: LogLevel::Warn,
        message: message.to_string(),
        target: target.map(|s| s.to_string()),
        attributes: HashMap::from_iter(attributes),
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
        timestamp: None,
        message: message.to_string(),
        target: target.map(|s| s.to_string()),
        attributes: HashMap::from_iter(attributes),
    })
}