use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

use opentelemetry::KeyValue;
use opentelemetry_resource_detectors::{
    HostResourceDetector, OsResourceDetector, ProcessResourceDetector,
};
use opentelemetry_sdk::resource::{
    EnvResourceDetector, SdkProvidedResourceDetector, TelemetryResourceDetector,
};
use opentelemetry_sdk::Resource;

use super::metrics::MetricUnit;

#[derive(Debug)]
pub enum TelemetryError {
    TracerInitError(String),
    MetricsInitError(String),
    LoggerInitError(String),
    ShutdownError(String),
}

impl fmt::Display for TelemetryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TelemetryError::TracerInitError(msg) => write!(f, "Tracer initialization error: {}", msg),
            TelemetryError::MetricsInitError(msg) => write!(f, "Metrics initialization error: {}", msg),
            TelemetryError::LoggerInitError(msg) => write!(f, "Logger initialization error: {}", msg),
            TelemetryError::ShutdownError(msg) => write!(f, "Shutdown error: {}", msg),
        }
    }
}

impl Error for TelemetryError {}

#[derive(Debug, Clone)]
pub struct SpanContext {
    pub name: String,
    pub attributes: Vec<(String, AttributeValue)>,
}

#[derive(Debug, Clone)]
pub enum AttributeValue {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
}

/// Log level for log events
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    /// Very detailed logs, potentially of all levels.
    Trace,
    /// Detailed information, typically of interest only when diagnosing problems.
    Debug,
    
    /// Confirmation that things are working as expected.
    Info,
    
    /// An indication that something unexpected happened, or indicative of some problem.
    /// The software is still working as expected.
    Warn,
    
    /// Due to a more serious problem, the software has not been able to perform some function.
    Error,

    /// A serious error that requires immediate attention.
    Critical
}

impl Default for LogLevel {
    fn default() -> Self {
        Self::Info
    }
}


#[derive(Debug, Clone)]
pub struct MetricContext {
    pub name: String,
    pub description: Option<String>,
    pub unit: Option<MetricUnit>,
    pub attributes: Vec<(String, AttributeValue)>,
}

#[derive(Debug, Clone)]
pub struct LogContext {
    pub level: LogLevel,
    pub timestamp: Option<u128>,
    pub message: String,
    pub target: Option<String>,
    pub attributes: HashMap<String, AttributeValue>,
}

impl LogContext {
    pub fn new(message: String, level: LogLevel) -> Self {
        Self {
            message,
            level,
            target: None,
            attributes: HashMap::new(),
            timestamp: Some(SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_nanos()),
        }
    }
    
    pub fn with_target(mut self, target: &str) -> Self {
        self.target = Some(target.to_string());
        self
    }
    
    pub fn with_attribute(mut self, key: &str, value: AttributeValue) -> Self {
        self.attributes.insert(key.to_string(), value);
        self
    }
    
    pub fn with_timestamp(mut self, timestamp: u128) -> Self {
        self.timestamp = Some(timestamp);
        self
    }
}

impl From<&str> for AttributeValue {
    fn from(value: &str) -> Self {
        Self::String(value.to_string())
    }
}

impl From<String> for AttributeValue {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<i64> for AttributeValue {
    fn from(value: i64) -> Self {
        Self::Int(value)
    }
}

impl From<usize> for AttributeValue {
    fn from(value: usize) -> Self {
        Self::Int(value as i64)
    }
}

impl From<i32> for AttributeValue {
    fn from(value: i32) -> Self {
        Self::Int(value as i64)
    }
}

impl From<u64> for AttributeValue {
    fn from(value: u64) -> Self {
        Self::Int(value as i64)
    }
}

impl From<u32> for AttributeValue {
    fn from(value: u32) -> Self {
        Self::Int(value as i64)
    }
}

impl From<f64> for AttributeValue {
    fn from(value: f64) -> Self {
        Self::Float(value)
    }
}

impl From<f32> for AttributeValue {
    fn from(value: f32) -> Self {
        Self::Float(value as f64)
    }
}

impl From<bool> for AttributeValue {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

pub fn get_resource() -> Resource {
    static RESOURCE: OnceLock<Resource> = OnceLock::new();
    RESOURCE
        .get_or_init(|| {
            Resource::builder()
                .with_detector(Box::new(HostResourceDetector::default()))
                .with_detector(Box::new(OsResourceDetector))
                .with_detector(Box::new(ProcessResourceDetector))
                .with_detector(Box::new(SdkProvidedResourceDetector))
                .with_detector(Box::new(EnvResourceDetector::new()))
                .with_detector(Box::new(TelemetryResourceDetector))
                .with_service_name(std::env::var("OTEL_SERVICE_NAME").unwrap_or_else(|_| "unknown".to_string()))
                .with_attribute(
                   KeyValue::new("service.version", std::env::var("OTEL_SERVICE_VERSION").unwrap_or_else(|_| "unknown".to_string()))
                )
                .with_attribute(
                    KeyValue::new("deployment.environment", std::env::var("OTEL_DEPLOYMENT_ENVIRONMENT").unwrap_or_else(|_| "unknown".to_string()))
                )
                .build()
        })
        .clone()
}

// Convert AttributeValue to OpenTelemetry KeyValue
pub fn to_key_value(key: String, value: &AttributeValue) -> KeyValue {
    match value {
        AttributeValue::String(s) => KeyValue::new(key, s.clone()),
        AttributeValue::Int(i) => KeyValue::new(key, *i),
        AttributeValue::Float(f) => KeyValue::new(key, *f),
        AttributeValue::Bool(b) => KeyValue::new(key, *b),
    }
}