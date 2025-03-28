use crate::domain::telemetry::{SpanContext, AttributeValue};
pub use crate::ports::tracer::Span;
use super::service;
use opentelemetry::context::FutureExt;

/// Create a new span.
pub fn create_span(context: SpanContext) -> Box<dyn Span> {
    service().create_span(context)
}

/// Execute a function within a span scope, automatically ending the span when done.
/// Properly maintains trace context for nested spans.
pub fn with_span<F, R>(name: &str, attributes: Vec<(String, AttributeValue)>, f: F) -> R
where
    F: FnOnce() -> R,
{
    // Create a span in the current context
    let span = create_span(SpanContext {
        name: name.to_string(),
        attributes,
    });
    
    // Get the context containing this span
    let cx = span.get_context();
    
    // Attach the context - this makes the span active for the duration
    let guard = cx.attach();
    
    // Execute the function
    let result = f();
    
    // End the span before dropping the guard
    span.end();
    
    // Detach the context (guard dropped)
    drop(guard);
    
    result
}


pub async fn with_async_span<F, R>(name: &str, attributes: Vec<(String, AttributeValue)>, fut: F) -> R
where
    F: std::future::Future<Output = R>,
{
    // Create a span in the current context
    let span = create_span(SpanContext {
        name: name.to_string(),
        attributes,
    });
    
    // Get the context containing this span
    let cx = span.get_context();
    
    // Use FutureExt::with_context which properly handles context propagation
    // without creating a guard that needs to be Send
    let result = fut.with_context(cx).await;
    
    // End the span when done
    span.end();
    
    result
}