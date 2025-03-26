//! Facade for logging functionality.
//! 
//! This module provides functions for structured logging.

use crate::{domain::telemetry::LogContext, LogLevel};
use super::service;

/// Log a message.
pub fn log(context: LogContext) {
    service().log(context)
}

/// Log a message with a specific target and attributes.
pub fn log_with_target(
    level: LogLevel,
    message: &str,
    target: &str,
    attributes: Vec<(String, crate::domain::telemetry::AttributeValue)>
) {
    log(LogContext {
        level,
        message: message.to_string(),
        target: Some(target.to_string()),
        attributes,
    });
}