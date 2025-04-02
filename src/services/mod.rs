mod tracing;
mod tracing;

pub use tracing::{init_tracing, TracingHandle};

use crate::config::TelemetryConfig;
use crate::adapters::datadog::{DatadogTracer, DatadogLogger};
use crate::ports::{Logger, Tracer};
use std::sync::{Arc, OnceLock};
use std::error::Error;

// Global instances of tracer and logger services
static TRACER: OnceLock<Arc<dyn Tracer>> = OnceLock::new();
static LOGGER: OnceLock<Arc<dyn Logger>> = OnceLock::new();

/// Result of initializing telemetry with all necessary handles
pub struct TelemetryHandle {
    /// Optional tracing handle for non-blocking logging
    pub tracing_handle: Option<TracingHandle>,
}

/// Initialize telemetry with the given configuration
pub fn init_telemetry(config: &TelemetryConfig) -> Result<TelemetryHandle, Box<dyn Error + Send + Sync>> {
    // Create Datadog tracer
    let tracer = DatadogTracer::new(
        &config.service_name,
        &config.version,
        &config.dd_trace_agent_url,
    )?;
    
    // Create Datadog logger
    let logger = DatadogLogger::new(
        &config.service_name,
        &config.version,
        &config.environment,
        &config.otlp_logs_endpoint,
    )?;
    
    // Store the services globally
    let tracer: Arc<dyn Tracer> = Arc::new(tracer);
    let logger: Arc<dyn Logger> = Arc::new(logger);
    
    if TRACER.set(tracer).is_err() {
        return Err("Tracer already initialized".into());
    }
    
    if LOGGER.set(logger).is_err() {
        return Err("Logger already initialized".into());
    }
    
    // Initialize tracing if enabled in the config
    let tracing_handle = if config.enable_tracing {
        Some(init_tracing(config)?)
    } else {
        None
    };
    
    Ok(TelemetryHandle { tracing_handle })
}

/// Get the global tracer instance
pub fn get_tracer() -> Arc<dyn Tracer> {
    TRACER.get()
        .expect("Telemetry not initialized - call init_telemetry first")
        .clone()
}

/// Get the global logger instance
pub fn get_logger() -> Arc<dyn Logger> {
    LOGGER.get()
        .expect("Telemetry not initialized - call init_telemetry first")
        .clone()
}

/// Shutdown telemetry
pub async fn shutdown_telemetry() -> Result<(), Box<dyn Error + Send + Sync>> {
    // Shutdown the logger if it exists
    if let Some(logger) = LOGGER.get() {
        logger.shutdown()?;
    }
    
    // Shutdown the tracer if it exists
    if let Some(tracer) = TRACER.get() {
        tracer.shutdown()?;
    }
    
    // Perform global providers shutdown
    opentelemetry::global::shutdown_tracer_provider();
    opentelemetry::global::shutdown_logger_provider();
    
    Ok(())
}
