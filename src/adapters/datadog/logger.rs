use crate::domain::telemetry::{get_resource, AttributeValue, LogContext, TelemetryError};
use crate::ports::logger::LoggerPort;
use crate::LogLevel;
use async_trait::async_trait;
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_otlp::LogExporter;
use opentelemetry_sdk::logs::SdkLoggerProvider;
use std::collections::HashMap;
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
    use_high_precision_timestamps: bool,
    service_name: String,
}

impl DatadogLogger {
    pub fn new(service_name: impl AsRef<str>) -> Self {
        Self {
            logger_provider: Mutex::new(None),
            use_high_precision_timestamps: true,
            service_name: service_name.as_ref().to_string(),
        }
    }

    pub fn with_high_precision_timestamps(mut self, use_high_precision: bool) -> Self {
        self.use_high_precision_timestamps = use_high_precision;
        self
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

    // Convert LogLevel to Datadog status string
    fn to_datadog_status(level: LogLevel) -> &'static str {
        match level {
            LogLevel::Debug => "debug",
            LogLevel::Info => "info",
            LogLevel::Warn => "warn",
            LogLevel::Error => "error",
            LogLevel::Critical => "critical",
            LogLevel::Trace => "trace",
        }
    }

    // Extract stack trace from an error
    fn extract_stack_trace(error: &dyn std::error::Error) -> String {
        // First, try to extract from sources if available
        let mut error_chain = Vec::new();
        let mut current_error: Option<&dyn std::error::Error> = Some(error);

        while let Some(err) = current_error {
            error_chain.push(err.to_string());
            current_error = err.source();
        }

        // Join the error chain into a stack-like format
        error_chain.join("\n    caused by: ")
    }

    // Extract error kind from a standard error
    fn extract_error_kind(error: &dyn std::error::Error) -> String {
        // Try to get the type name using std::any downcast
        if let Some(error_type) = (|| {
            let type_id = std::any::TypeId::of::<dyn std::error::Error>();
            let type_name = std::any::type_name::<dyn std::error::Error>();
            Some(type_name.split("::").last()?.to_string())
        })() {
            error_type
        } else {
            // Fallback: use a generic error type
            "Error".to_string()
        }
    }

    // Transform flat attributes to a Datadog-compatible nested structure
    fn transform_attributes_to_datadog_format(
        attributes: HashMap<String, AttributeValue>,
    ) -> HashMap<String, AttributeValue> {
        let mut datadog_attributes = HashMap::new();

        // Process each attribute to apply Datadog conventions
        for (key, value) in attributes {
            // Handle special Datadog reserved attributes
            match key.as_str() {
                "service" | "host" | "source" | "trace_id" => {
                    datadog_attributes.insert(key, value);
                }
                // Datadog standard attribute domains
                _ if key.starts_with("network.")
                    || key.starts_with("http.")
                    || key.starts_with("logger.")
                    || key.starts_with("error.")
                    || key.starts_with("usr.")
                    || key.starts_with("db.")
                    || key.starts_with("syslog.")
                    || key.starts_with("dns.")
                    || key.starts_with("evt.") =>
                {
                    // These are already in Datadog standard format, keep as is
                    datadog_attributes.insert(key, value);
                }
                // Common attributes to remap to Datadog standard attributes
                "user" | "user_id" => {
                    datadog_attributes.insert("usr.id".to_string(), value);
                }
                "duration" | "latency" | "exec_time" | "time_elapsed" => {
                    datadog_attributes.insert("duration".to_string(), value);
                }
                "ip" | "client_ip" | "remote_addr" | "remote_ip" => {
                    datadog_attributes.insert("network.client.ip".to_string(), value);
                }
                // For all other attributes, keep them as custom attributes
                _ => {
                    datadog_attributes.insert(key, value);
                }
            }
        }

        datadog_attributes
    }
}

#[async_trait]
impl LoggerPort for DatadogLogger {
    async fn init(&self, filter: Option<EnvFilter>) -> Result<(), TelemetryError> {
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
        let filter_otel = filter.unwrap_or_else(|| {
            EnvFilter::new("info")
                .add_directive("opentelemetry=info".parse().unwrap())
                .add_directive("hyper=off".parse().unwrap())
                .add_directive("tonic=off".parse().unwrap())
                .add_directive("h2=off".parse().unwrap())
                .add_directive("reqwest=off".parse().unwrap())
        });

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

    fn log_error(
        &self,
        error: Box<dyn std::error::Error>,
        target: Option<&str>,
        mut attributes: Vec<(String, AttributeValue)>,
    ) {
        // Extract stack trace information
        let stack_trace = Self::extract_stack_trace(&*error);
        let error_message = error.to_string();
        let error_kind = Self::extract_error_kind(&*error);
        //let thread_name = thread::current().name().unwrap_or("unknown").to_string();

        // Add Datadog specific stack trace attributes
        attributes.push((
            "logger.name".to_string(),
            AttributeValue::String(target.unwrap_or("app").to_string()),
        ));
        //attributes.push(("logger.thread_name".to_string(), AttributeValue::String(thread_name)));
        attributes.push((
            "error.stack".to_string(),
            AttributeValue::String(stack_trace),
        ));
        attributes.push((
            "error.message".to_string(),
            AttributeValue::String(error_message.clone()),
        ));
        attributes.push(("error.kind".to_string(), AttributeValue::String(error_kind)));

        // Get current high-precision timestamp
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap();
        let timestamp = if self.use_high_precision_timestamps {
            now.as_nanos() as u128
        } else {
            now.as_millis() as u128
        };

        let context = LogContext {
            timestamp: Some(timestamp),
            message: error_message,
            level: LogLevel::Error,
            target: target.map(|s| s.to_string()),
            attributes: attributes.into_iter().collect(),
        };

        self.log(context);
    }

    fn log(&self, context: LogContext) {
        let target = context.target.as_deref().unwrap_or("app");
        let level = Self::to_tracing_level(context.level);
        let status = Self::to_datadog_status(context.level);

        

        let timestamp = context.timestamp.unwrap_or_else(|| {
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_millis()
        });

        // Add standard Datadog attributes if not present
        let mut attributes = context.attributes.clone();
        if !attributes.contains_key("status") {
            attributes.insert(
                "status".to_string(),
                AttributeValue::String(status.to_string()),
            );
        }
        if !attributes.contains_key("service") {
            attributes.insert(
                "service".to_string(),
                AttributeValue::String(self.service_name.clone()),
            );
        }

        let datadog_attributes = Self::transform_attributes_to_datadog_format(attributes);
        
        //create a string from the attributes
        let dd_attrs = datadog_attributes
            .iter()
            .map(|(k, v)| format!("{} = {}", k, v))
            .collect::<Vec<_>>()
            .join(", ");

        let message = context.message.clone();

        println!("message and level: {} - {}", message, dd_attrs);
       
        

        // Create a tracing event with the appropriate level
        match level {
            Level::ERROR => {
                 // Create a span with all attributes properly attached
                 let span_builder = tracing::span!(
                    parent: None, 
                    Level::ERROR, 
                    "error_log",
                    message = %context.message,
                    timestamp = %timestamp
                );
                
                // Add all attributes to the span
                for (key, value) in &datadog_attributes {
                    match value {
                        AttributeValue::String(s) => { span_builder.record(key.as_str(), &s.as_str()); },
                        AttributeValue::Int(i) => { span_builder.record(key.as_str(), &i); },
                        AttributeValue::Float(f) => { span_builder.record(key.as_str(), &f); },
                        AttributeValue::Bool(b) => { span_builder.record(key.as_str(), &b); },
                    }
                }
                
                // Enter the span to make it the current span
                let _guard = span_builder.enter();
                
                // Log the event within the span context
                error!(target = target, "{}", context.message);
            }
            Level::WARN => {
              // Create a span with all attributes properly attached
              let span_builder = tracing::span!(
                parent: None, 
                Level::WARN, 
                "warn_log",
                message = %context.message,
                timestamp = %timestamp
            );
            
            // Add all attributes to the span
            for (key, value) in &datadog_attributes {
                match value {
                    AttributeValue::String(s) => { span_builder.record(key.as_str(), &s.as_str()); },
                    AttributeValue::Int(i) => { span_builder.record(key.as_str(), &i); },
                    AttributeValue::Float(f) => { span_builder.record(key.as_str(), &f); },
                    AttributeValue::Bool(b) => { span_builder.record(key.as_str(), &b); },
                }
            }
            
            // Enter the span to make it the current span
            let _guard = span_builder.enter();
            
            // Log the event within the span context
            warn!(target = target, "{}", context.message);
            }
            Level::INFO => {
                // Create a span with all attributes properly attached
                let span_builder = tracing::span!(
                    parent: None, 
                    Level::INFO, 
                    "info_log",
                    message = %context.message,
                    timestamp = %timestamp
                );
                
                // Add all attributes to the span
                for (key, value) in &datadog_attributes {
                    match value {
                        AttributeValue::String(s) => { span_builder.record(key.as_str(), &s.as_str()); },
                        AttributeValue::Int(i) => { span_builder.record(key.as_str(), &i); },
                        AttributeValue::Float(f) => { span_builder.record(key.as_str(), &f); },
                        AttributeValue::Bool(b) => { span_builder.record(key.as_str(), &b); },
                    }
                }
                
                // Enter the span to make it the current span
                let _guard = span_builder.enter();
                
                // Log the event within the span context
                info!(target = target, "{}", context.message);
            }
            Level::DEBUG => {
                // Create a span with all attributes properly attached
                let mut span_builder = tracing::span!(
                    parent: None, 
                    Level::DEBUG, 
                    "debug_log",
                    message = %context.message,
                    timestamp = %timestamp
                );
                
                // Add all attributes to the span
                for (key, value) in &datadog_attributes {
                    match value {
                        AttributeValue::String(s) => { span_builder.record(key.as_str(), &s.as_str()); },
                        AttributeValue::Int(i) => { span_builder.record(key.as_str(), &i); },
                        AttributeValue::Float(f) => { span_builder.record(key.as_str(), &f); },
                        AttributeValue::Bool(b) => { span_builder.record(key.as_str(), &b); },
                    }
                }
                
                
                // Enter the span to make it the current span
                let _guard = span_builder.enter();
                
                // Log the event within the span context
                debug!(target = target, "{}", context.message);
            }
            _ => {
                // Use info for any other level
                println!("{:?}: {}", context.level, context.message);
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
