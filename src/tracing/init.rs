use crate::tracing::formatter::DatadogFormatter;
use std::env;
use opentelemetry_sdk::trace::TraceError;
use tracing::Subscriber;
use tracing_appender::non_blocking::{NonBlocking, WorkerGuard};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Layer, Registry};

use super::shutdown::TracerShutdown;
use super::tracer::build_tracer;

fn loglevel_filter_layer(dd_enabled: bool) -> EnvFilter {
    let log_level = env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());


    // `otel::setup` set to debug to log detected resources, configuration read and infered
    let otel_log_level = env::var("OTEL_LOG_LEVEL").unwrap_or_else(|_| "debug".to_string());

    env::set_var(
        "RUST_LOG",
        format!("{log_level},otel={otel_log_level}"),
    );

    EnvFilter::from_default_env()
}

fn log_layer<S>(
    dd_enabled: bool,
    non_blocking: NonBlocking,
) -> Box<dyn Layer<S> + Send + Sync + 'static>
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    if dd_enabled {
        Box::new(
            tracing_subscriber::fmt::layer()
                .json()
                .event_format(DatadogFormatter)
                .with_writer(non_blocking),
        )
    } else {
        Box::new(tracing_subscriber::fmt::layer().with_writer(non_blocking))
    }
}

pub fn init() -> Result<(WorkerGuard, TracerShutdown), TraceError> {
    let (non_blocking, guard) = tracing_appender::non_blocking(std::io::stdout());

    let dd_enabled = env::var("DD_ENABLED").map(|s| s == "true").unwrap_or(false);

    let tracer = if dd_enabled {
        Some(build_tracer()?)
    } else {
        None
    };
    let telemetry_layer = tracer.map(|tracer| tracing_opentelemetry::layer().with_tracer(tracer));

    Registry::default()
        .with(loglevel_filter_layer(dd_enabled))
        .with(log_layer(dd_enabled, non_blocking))
        .with(telemetry_layer)
        .init();

    Ok((guard, TracerShutdown {}))
}