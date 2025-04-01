use async_trait::async_trait;
use tracing_subscriber::EnvFilter;

use crate::{domain::telemetry::{LogContext, TelemetryError}, AttributeValue};

#[async_trait]
pub trait LoggerPort: Send + Sync {
    async fn init(&self, filter: Option<EnvFilter>) -> Result<(), TelemetryError>;
    
    fn log(&self, context: LogContext);

    fn log_error(
        &self,
        error: Box<dyn std::error::Error>,
        target: Option<&str>,
        attributes: Vec<(String, AttributeValue)>,
    );
    
    async fn shutdown(&self) -> Result<(), TelemetryError>;
}
