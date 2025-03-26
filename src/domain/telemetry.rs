use std::error::Error;
use std::fmt;
use std::sync::OnceLock;

use opentelemetry::KeyValue;
use opentelemetry_resource_detectors::{
    HostResourceDetector, OsResourceDetector, ProcessResourceDetector,
};
use opentelemetry_sdk::resource::{
    EnvResourceDetector, SdkProvidedResourceDetector, TelemetryResourceDetector,
};
use opentelemetry_sdk::Resource;

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

#[derive(Debug, Clone, PartialEq)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
    Critical,
}

#[derive(Debug, Clone)]
pub struct MetricContext {
    pub name: String,
    pub description: Option<String>,
    pub unit: Option<String>,
    pub attributes: Vec<(String, AttributeValue)>,
}

#[derive(Debug, Clone)]
pub struct LogContext {
    pub level: LogLevel,
    pub message: String,
    pub target: Option<String>,
    pub attributes: Vec<(String, AttributeValue)>,
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