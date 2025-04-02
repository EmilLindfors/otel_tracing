pub mod domain;
pub mod ports;
pub mod adapters;
pub mod services;
pub mod config;

pub use config::TelemetryConfig;
pub use services::{
    init_telemetry, shutdown_telemetry, get_logger, get_tracer,
    init_tracing, TracingHandle, TelemetryHandle
};
