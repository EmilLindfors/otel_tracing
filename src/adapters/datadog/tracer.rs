use std::sync::Arc;
use std::sync::Mutex;
use async_trait::async_trait;
use opentelemetry::global;
use opentelemetry::trace::SpanKind;
use opentelemetry::trace::{Tracer as OtelTracer, Span as OtelSpan, TraceContextExt};
use opentelemetry::Context;
use opentelemetry::KeyValue;
use opentelemetry_sdk::trace::SdkTracerProvider;
use opentelemetry_otlp::SpanExporter;
use tracing::debug;
use tracing::info;

use crate::domain::telemetry::{SpanContext, AttributeValue, TelemetryError, get_resource, to_key_value};
use crate::ports::tracer::{TracerPort, Span};

pub struct DatadogTracer {
    tracer_provider: Mutex<Option<SdkTracerProvider>>,
}

impl DatadogTracer {
    pub fn new() -> Self {
        Self {
            tracer_provider: Mutex::new(None),
        }
    }
}

#[async_trait]
impl TracerPort for DatadogTracer {
    async fn init(&self) -> Result<(), TelemetryError> {
        info!("Initializing DatadogTracer");
        let resource = get_resource();
            
        let exporter = SpanExporter::builder()
            .with_tonic()
            .build()
            .map_err(|e| TelemetryError::TracerInitError(e.to_string()))?;
            
        let tracer_provider = SdkTracerProvider::builder()
            .with_resource(resource)
            .with_batch_exporter(exporter)
            .build();
            
        // Set global tracer provider
        global::set_tracer_provider(tracer_provider.clone());
        
        // Store provider for shutdown
        let mut provider = self.tracer_provider.lock().unwrap();
        *provider = Some(tracer_provider);

        info!("DatadogTracer initialized successfully");
        
        Ok(())
    }
    
    fn create_span(&self, context: SpanContext) -> Box<dyn Span> {
        let tracer = global::tracer("datadog-tracer");
        
        let attributes: Vec<KeyValue> = context.attributes.iter()
            .map(|(k, v)| to_key_value(k.clone(), v))
            .collect();

        // Get the current context - will contain parent span if one exists
        let current_ctx = Context::current();
        
        if current_ctx.span().span_context().is_valid() {
            debug!("Creating child span with parent: {:?}", current_ctx.span().span_context().trace_id());
        } else {
            debug!("Creating root span (no parent)");
        }
        
        // Create a span builder
        let span_builder = tracer.span_builder(context.name.clone())
            .with_kind(SpanKind::Internal)
            .with_attributes(attributes);
            
        // Start the span within the current context (preserving parent relationship)
        let span = tracer.build_with_context(span_builder, &current_ctx);
        
        // Create a new context with this span
        let cx = current_ctx.with_span(span);
        
        Box::new(DatadogSpan { 
            ctx: cx,
            name: context.name,
        })
    }
    
    async fn shutdown(&self) -> Result<(), TelemetryError> {
        info!("Shutting down DatadogTracer");
        let mut provider = self.tracer_provider.lock().unwrap();
        if let Some(provider) = provider.take() {
            provider.shutdown()
                .map_err(|e| TelemetryError::ShutdownError(e.to_string()))?;
        }
        
        Ok(())
    }
}

struct DatadogSpan {
    // The context containing the span
    ctx: Context,
    // Store span name for debugging
    name: String,
}

impl Span for DatadogSpan {
    fn set_attribute(&self, key: String, value: AttributeValue) {
        let keyvalue = match &value {
            AttributeValue::String(s) => KeyValue::new(key, s.clone()),
            AttributeValue::Int(i) => KeyValue::new(key, *i),
            AttributeValue::Float(f) => KeyValue::new(key, *f),
            AttributeValue::Bool(b) => KeyValue::new(key, *b),
            AttributeValue::Uint(u) => KeyValue::new(key, u.to_string()),
        };
        
        // Use the TraceContextExt to get the span from the context
        self.ctx.span().set_attribute(keyvalue);
    }
    
    fn add_event(&self, name: &str, attributes: Vec<(String, AttributeValue)>) {
        let otel_attributes = attributes.iter()
            .map(|(k, v)| to_key_value(k.clone(), v))
            .collect();
            
        self.ctx.span().add_event(name.to_string(), otel_attributes);
    }
    
    fn end(&self) {
        self.ctx.span().end();
    }

    fn get_context(&self) -> Context {
        self.ctx.clone()
    }
}