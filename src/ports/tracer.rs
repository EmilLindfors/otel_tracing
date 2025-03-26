use async_trait::async_trait;

use crate::domain::telemetry::{SpanContext, AttributeValue, TelemetryError};

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
}
