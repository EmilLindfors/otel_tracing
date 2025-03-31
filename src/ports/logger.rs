use async_trait::async_trait;
use tracing_subscriber::EnvFilter;

use crate::domain::telemetry::{LogContext, TelemetryError};

#[async_trait]
pub trait LoggerPort: Send + Sync {
    async fn init(&self, filter: Option<EnvFilter>) -> Result<(), TelemetryError>;
    
    fn log(&self, context: LogContext);
    
    async fn shutdown(&self) -> Result<(), TelemetryError>;
}
