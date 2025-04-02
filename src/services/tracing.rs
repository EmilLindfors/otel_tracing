use crate::adapters::datadog::formatter::DatadogFormatter;
use crate::config::TelemetryConfig;
use std::env;
use std::sync::Arc;
use tracing_appender::non_blocking::{NonBlocking, WorkerGuard};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Layer, Registry};

use crate::adapters::datadog::tracing::TracingBuilder;

/// Result of initializing tracing - contains the worker guard and shutdown handle
pub struct TracingHandle {
    /// Worker guard that keeps the non-blocking writer alive
    pub guard: WorkerGuard,
}

/// Create a log filter layer based on environment variables
fn create_log_filter(config: &TelemetryConfig) -> EnvFilter {
    // Use RUST_LOG environment variable or fallback to info level
    let log_level = env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());
    
    // Configure OpenTelemetry logging level (useful for debugging)
    let otel_log_level = env::var("OTEL_LOG_LEVEL").unwrap_or_else(|_| "debug".to_string());
    
    // Set the combined filter
    env::set_var(
        "RUST_LOG",
        format!("{log_level},otel={otel_log_level}"),
    );
    
    EnvFilter::from_default_env()
}

/// Create a logging layer that outputs to the specified writer
fn create_log_layer<S>(
    config: &TelemetryConfig,
    non_blocking: NonBlocking,
) -> Box<dyn Layer<S> + Send + Sync + 'static>
where
    S: tracing::Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
{
    // Format logs with Datadog correlation if telemetry is enabled
    Box::new(
        tracing_subscriber::fmt::layer()
            .json()
            .event_format(DatadogFormatter)
            .with_writer(non_blocking)
    )
}

/// Initialize tracing with Datadog integration
pub fn init_tracing(config: &TelemetryConfig) -> Result<TracingHandle, Box<dyn std::error::Error + Send + Sync>> {
    // Set up a non-blocking writer for logs
    let (non_blocking, guard) = tracing_appender::non_blocking(std::io::stdout());
    
    // Build the telemetry layer if Datadog is enabled
    let telemetry_layer = TracingBuilder::new(&config.service_name, &config.version)
        .with_agent_endpoint(&config.dd_trace_agent_url)
        .build_layer()?;
    
    // Initialize the tracing subscriber with all layers
    Registry::default()
        .with(create_log_filter(config))
        .with(create_log_layer(config, non_blocking))
        .with(telemetry_layer)
        .init();
    
    Ok(TracingHandle { guard })
}