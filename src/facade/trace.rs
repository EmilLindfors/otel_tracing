use crate::domain::telemetry::{SpanContext, AttributeValue};
use crate::ports::tracer::Span;
use super::service;

/// Create a new span.
pub fn create_span(context: SpanContext) -> Box<dyn Span> {
    service().create_span(context)
}

/// Execute a function within a span and automatically end the span when done.
pub fn with_span<F, R>(name: &str, attributes: Vec<(String, AttributeValue)>, f: F) -> R
where
    F: FnOnce() -> R,
{
    let span = create_span(SpanContext {
        name: name.to_string(),
        attributes,
    });
    
    let result = f();
    span.end();
    result
}

/// Execute an async function within a span and automatically end the span when done.
pub async fn with_async_span<F, R>(name: &str, attributes: Vec<(String, AttributeValue)>, f: F) -> R
where
    F: std::future::Future<Output = R>,
{
    let span = create_span(SpanContext {
        name: name.to_string(),
        attributes,
    });
    
    let result = f.await;
    span.end();
    result
}
