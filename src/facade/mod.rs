//! Public facade layer for the telemetry service.
//!
//! This module provides a simple interface to the telemetry service
//! with initialization, shutdown, and global service management.

mod log;
mod metrics;
mod trace;

use std::sync::{Arc, OnceLock};

use crate::domain::telemetry::TelemetryError;
use crate::services::telemetry::TelemetryService;
pub use crate::domain::metrics::*;

// Re-export all public functions from sub-modules
pub use log::*;
pub use metrics::*;
pub use trace::*;
use tracing_subscriber::EnvFilter;

// Global instance of TelemetryService
static TELEMETRY_SERVICE: OnceLock<Arc<TelemetryService>> = OnceLock::new();

/// Initialize the global telemetry service.
/// This must be called before any other telemetry functions.
pub async fn init(service: TelemetryService, filter: Option<EnvFilter>) -> Result<(), TelemetryError> {
    let service_arc = Arc::new(service);

    if TELEMETRY_SERVICE.set(service_arc.clone()).is_err() {
        return Err(TelemetryError::TracerInitError(
            "Telemetry service already initialized".to_string(),
        ));
    }

    service_arc.init(filter).await
}

/// Initialize a DataDog-based telemetry service.
/// This is a convenience function for common DataDog setup.
pub async fn init_datadog(filter: Option<EnvFilter>) -> Result<(), TelemetryError> {
    let service = crate::services::telemetry::TelemetryServiceBuilder::build_datadog()?;

    init(service, filter).await
}

/// Shutdown the global telemetry service.
pub async fn shutdown() -> Result<(), TelemetryError> {
    if let Some(service) = TELEMETRY_SERVICE.get() {
        service.shutdown().await
    } else {
        Ok(())
    }
}

/// Get a reference to the global telemetry service.
pub(crate) fn service() -> &'static TelemetryService {
    TELEMETRY_SERVICE
        .get()
        .expect("Telemetry service not initialized")
}
