use async_trait::async_trait;
use opentelemetry::Context;

use crate::domain::telemetry::{AttributeValue, SpanContext, TelemetryError};

#[async_trait]
pub trait TracerPort: Send + Sync {
    async fn init(&self) -> Result<(), TelemetryError>;

    fn create_span(&self, context: SpanContext) -> Box<dyn Span>;

    async fn shutdown(&self) -> Result<(), TelemetryError>;
}

pub trait Span: Send + Sync {
    fn set_attribute(&self, key: String, value: AttributeValue);

    fn add_event(&self, name: &str, attributes: Vec<(String, AttributeValue)>);

    fn end(&self);

    /// Get the OpenTelemetry context containing this span
    /// This is used for context propagation across async boundaries
    fn get_context(&self) -> Context;
}
