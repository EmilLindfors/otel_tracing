use opentelemetry::Value;
use std::error::Error;

/// Tracer interface - primary port for tracing operations
pub trait Tracer: Send + Sync {
    /// Start a new span
    fn start_span(&self, name: &str) -> Box<dyn Span>;
    
    /// Create a span and execute the given closure within it
    fn with_span<F, R>(&self, name: &str, f: F) -> R
    where
        F: FnOnce() -> R;
        
    /// Shut down the tracer
    fn shutdown(&self) -> Result<(), Box<dyn Error + Send + Sync>>;
}

/// Span interface
pub trait Span: Send + Sync {
    /// Set an attribute on the span
    fn set_attribute(&mut self, key: &str, value: impl Into<Value>);
    
    /// End the span
    fn end(&mut self);
}
