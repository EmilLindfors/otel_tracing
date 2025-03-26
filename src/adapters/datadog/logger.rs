use async_trait::async_trait;
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_otlp::LogExporter;
use opentelemetry_sdk::logs::SdkLoggerProvider;
use std::sync::Mutex;
use tracing::info;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::Layer;
use crate::domain::telemetry::{get_resource, AttributeValue, LogContext, TelemetryError};
use crate::ports::logger::LoggerPort;

pub struct DatadogLogger {
    logger_provider: Mutex<Option<SdkLoggerProvider>>,
}

impl DatadogLogger {
    pub fn new() -> Self {
        Self {
            logger_provider: Mutex::new(None),
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
        let target = context
            .target
            .unwrap_or_else(|| "default-target".to_string());

        // Convert attributes to a format usable with structured logging
        let attributes: Vec<(&str, String)> = context
            .attributes
            .iter()
            .map(|(k, v)| {
                let value = match v {
                    AttributeValue::String(s) => s.clone(),
                    AttributeValue::Int(i) => i.to_string(),
                    AttributeValue::Float(f) => f.to_string(),
                    AttributeValue::Bool(b) => b.to_string(),
                };
                (k.as_str(), value)
            })
            .collect();

        // Use tracing to log the message
        info!(target, message = context.message.as_str(), ?attributes);
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
