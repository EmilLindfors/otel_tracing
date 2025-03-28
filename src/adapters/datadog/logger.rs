use crate::domain::telemetry::{get_resource, AttributeValue, LogContext, TelemetryError};
use crate::ports::logger::LoggerPort;
use crate::LogLevel;
use async_trait::async_trait;
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_otlp::LogExporter;
use opentelemetry_sdk::logs::SdkLoggerProvider;
use std::sync::Mutex;
use std::time::SystemTime;
use tracing::debug;
use tracing::error;
use tracing::info;
use tracing::warn;
use tracing::Level;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::Layer;

pub struct DatadogLogger {
    logger_provider: Mutex<Option<SdkLoggerProvider>>,
}

impl DatadogLogger {
    pub fn new() -> Self {
        Self {
            logger_provider: Mutex::new(None),
        }
    }

    // Convert LogLevel to tracing::Level
    fn to_tracing_level(level: LogLevel) -> Level {
        match level {
            LogLevel::Debug => Level::DEBUG,
            LogLevel::Info => Level::INFO,
            LogLevel::Warn => Level::WARN,
            LogLevel::Error => Level::ERROR,
            LogLevel::Critical => Level::ERROR,
            LogLevel::Trace => Level::TRACE,
        }
    }

    
}

#[async_trait]
impl LoggerPort for DatadogLogger {
    async fn init(&self) -> Result<(), TelemetryError> {
        let resource = get_resource();

        let exporter = LogExporter::builder()
            .with_tonic()
            .build()
            .map_err(|e| TelemetryError::LoggerInitError(e.to_string()))?;

        let logger_provider = SdkLoggerProvider::builder()
            .with_resource(resource)
            .with_batch_exporter(exporter)
            .build();

        // Create OpenTelemetry layer
        let otel_layer = OpenTelemetryTracingBridge::new(&logger_provider);

        // Create filter for OpenTelemetry layer
        let filter_otel = EnvFilter::new("info")
            .add_directive("hyper=off".parse().unwrap())
            .add_directive("opentelemetry=off".parse().unwrap())
            .add_directive("tonic=off".parse().unwrap())
            .add_directive("h2=off".parse().unwrap())
            .add_directive("reqwest=off".parse().unwrap());

        let otel_layer = otel_layer.with_filter(filter_otel);

        // Create a standard formatting layer
        let filter_fmt =
            EnvFilter::new("info").add_directive("opentelemetry=debug".parse().unwrap());

        let fmt_layer = tracing_subscriber::fmt::layer()
            .with_thread_names(true)
            .with_filter(filter_fmt);

        // Initialize the tracing subscriber
        tracing_subscriber::registry()
            .with(otel_layer)
            .with(fmt_layer)
            .init();

        // Store provider for shutdown
        let mut provider = self.logger_provider.lock().unwrap();
        *provider = Some(logger_provider);

        Ok(())
    }

    

    fn log(&self, context: LogContext) {
        let target = context.target.as_deref().unwrap_or("app");
        let level = Self::to_tracing_level(context.level);

        // Extract timestamp if available
        let timestamp = context
            .timestamp
            .map(|ts| {
                // Convert nanoseconds to seconds and fractional part for readability
                let seconds = ts / 1_000_000_000;
                let nanos = ts % 1_000_000_000;
                format!("{}.{:09}", seconds, nanos)
            })
            .unwrap_or(
                SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_secs_f64()
                    .to_string(),
            );

        // Create a tracing event with the appropriate level
        match level {
            Level::ERROR => {
                if context.attributes.is_empty() {
                    error!(target, "{}", context.message);
                } else {
                    let fields = context
                        .attributes
                        .iter()
                        .map(|(k, v)| {
                            format!(
                                "{}={}",
                                k,
                                match v {
                                    AttributeValue::String(s) => s.clone(),
                                    AttributeValue::Int(i) => i.to_string(),
                                    AttributeValue::Float(f) => f.to_string(),
                                    AttributeValue::Bool(b) => b.to_string(),
                                }
                            )
                        })
                        .collect::<Vec<_>>()
                        .join(", ");

                    error!(target, timestamp, "{} [{}]", context.message, fields);
                }
            }
            Level::WARN => {
                if context.attributes.is_empty() {
                    warn!(target, "{}", context.message);
                } else {
                    let fields = context
                        .attributes
                        .iter()
                        .map(|(k, v)| {
                            format!(
                                "{}={}",
                                k,
                                match v {
                                    AttributeValue::String(s) => s.clone(),
                                    AttributeValue::Int(i) => i.to_string(),
                                    AttributeValue::Float(f) => f.to_string(),
                                    AttributeValue::Bool(b) => b.to_string(),
                                }
                            )
                        })
                        .collect::<Vec<_>>()
                        .join(", ");

                    warn!(target, timestamp, "{} [{}]", context.message, fields);
                }
            }
            Level::INFO => {
                if context.attributes.is_empty() {
                    info!(target, "{}", context.message);
                } else {
                    let fields = context
                        .attributes
                        .iter()
                        .map(|(k, v)| {
                            format!(
                                "{}={}",
                                k,
                                match v {
                                    AttributeValue::String(s) => s.clone(),
                                    AttributeValue::Int(i) => i.to_string(),
                                    AttributeValue::Float(f) => f.to_string(),
                                    AttributeValue::Bool(b) => b.to_string(),
                                }
                            )
                        })
                        .collect::<Vec<_>>()
                        .join(", ");

                    info!(target, timestamp, "{} [{}]", context.message, fields);
                }
            }
            Level::DEBUG => {
                if context.attributes.is_empty() {
                    debug!(target, "{}", context.message);
                } else {
                    let fields = context
                        .attributes
                        .iter()
                        .map(|(k, v)| {
                            format!(
                                "{}={}",
                                k,
                                match v {
                                    AttributeValue::String(s) => s.clone(),
                                    AttributeValue::Int(i) => i.to_string(),
                                    AttributeValue::Float(f) => f.to_string(),
                                    AttributeValue::Bool(b) => b.to_string(),
                                }
                            )
                        })
                        .collect::<Vec<_>>()
                        .join(", ");

                    debug!(target, timestamp, "{} [{}]", context.message, fields);
                }
            }
            _ => {
                // Use info for any other level
                info!(target, timestamp, "{}", context.message);
            }
        }
    }

    async fn shutdown(&self) -> Result<(), TelemetryError> {
        let mut provider = self.logger_provider.lock().unwrap();
        if let Some(provider) = provider.take() {
            provider
                .shutdown()
                .map_err(|e| TelemetryError::ShutdownError(e.to_string()))?;
        }

        Ok(())
    }
}
