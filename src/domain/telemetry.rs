use opentelemetry::{KeyValue, Value};
use std::collections::HashMap;

/// Core telemetry event representation
#[derive(Debug, Clone)]
pub struct TelemetryEvent {
    pub name: String,
    pub attributes: Vec<KeyValue>,
    pub timestamp: Option<std::time::SystemTime>,
    pub level: Level,
}

/// Log level enum
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl From<Level> for opentelemetry::logs::Severity {
    fn from(level: Level) -> Self {
        match level {
            Level::Trace => opentelemetry::logs::Severity::Trace,
            Level::Debug => opentelemetry::logs::Severity::Debug,
            Level::Info => opentelemetry::logs::Severity::Info,
            Level::Warn => opentelemetry::logs::Severity::Warn,
            Level::Error => opentelemetry::logs::Severity::Error,
        }
    }
}

impl TelemetryEvent {
    /// Create a new telemetry event with the given name and level
    pub fn new(name: impl Into<String>, level: Level) -> Self {
        Self {
            name: name.into(),
            attributes: Vec::new(),
            timestamp: Some(std::time::SystemTime::now()),
            level,
        }
    }

    /// Add an attribute to the telemetry event
    pub fn add_attribute(&mut self, key: impl Into<String>, value: impl Into<Value>) {
        self.attributes.push(KeyValue::new(key.into(), value.into()));
    }

    /// Add an attribute to the telemetry event and return self (builder pattern)
    pub fn with_attribute(mut self, key: impl Into<String>, value: impl Into<Value>) -> Self {
        self.add_attribute(key, value);
        self
    }
}
