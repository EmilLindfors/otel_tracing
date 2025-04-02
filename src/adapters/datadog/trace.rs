use crate::ports::tracer::{Span, Tracer};
use opentelemetry::{
    global,
    trace::{SamplingResult, TraceContextExt, TracerProvider},
    InstrumentationScope, Key, KeyValue, Value,
};
use opentelemetry_datadog::{new_pipeline, ApiVersion, DatadogTraceStateBuilder};
use opentelemetry_sdk::trace::{self, RandomIdGenerator, ShouldSample};
use opentelemetry_semantic_conventions as semcov;
use std::sync::Arc;

/// DatadogTracer implements the Tracer port for Datadog
#[derive(Debug, Clone)]
pub struct DatadogTracer {
    tracer: opentelemetry::trace::Tracer,
    provider: trace::TracerProvider,
}

impl DatadogTracer {
    /// Create a new DatadogTracer
    pub fn new(service_name: &str, version: &str, agent_url: &str) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let mut config = trace::Config::default();
        config.sampler = Box::new(AgentBasedSampler);
        config.id_generator = Box::new(RandomIdGenerator::default());
        
        // Configure the trace pipeline to send to Datadog
        let pipeline = new_pipeline()
            .with_service_name(service_name)
            .with_api_version(ApiVersion::Version05)
            .with_agent_endpoint(agent_url)
            .with_trace_config(config);
            
        // Install the pipeline
        let provider = pipeline.install_simple()?;
            
        global::set_tracer_provider(provider.clone());
        
        // Create a tracer with appropriate instrumentation scope
        let scope = InstrumentationScope::builder(service_name)
            .with_version(version)
            .with_schema_url(semcov::SCHEMA_URL)
            .with_attributes(None)
            .build();
            
        let tracer = provider.tracer_with_scope(scope);
        
        Ok(Self { tracer, provider })
    }
}

/// DatadogSpan implements the Span port for Datadog
#[derive(Debug)]
pub struct DatadogSpan {
    span: opentelemetry::trace::Span,
}

impl Span for DatadogSpan {
    fn set_attribute(&mut self, key: &str, value: impl Into<Value>) {
        self.span.set_attribute(KeyValue::new(key.to_string(), value.into()));
    }
    
    fn end(&mut self) {
        self.span.end();
    }
}

impl Tracer for DatadogTracer {
    fn start_span(&self, name: &str) -> Box<dyn Span> {
        let span = self.tracer.start(name);
        Box::new(DatadogSpan { span })
    }
    
    fn with_span<F, R>(&self, name: &str, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        self.tracer.in_span(name, |_| f())
    }
    
    fn shutdown(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Note: The global provider shutdown will be handled centrally
        Ok(())
    }
}

/// Sampler implementation for Datadog agent-based sampling
#[derive(Debug, Clone)]
struct AgentBasedSampler;

impl ShouldSample for AgentBasedSampler {
    fn should_sample(
        &self,
        parent_context: Option<&opentelemetry::Context>,
        _trace_id: opentelemetry::trace::TraceId,
        _name: &str,
        _span_kind: &opentelemetry::trace::SpanKind,
        _attributes: &[opentelemetry::KeyValue],
        _links: &[opentelemetry::trace::Link],
    ) -> opentelemetry::trace::SamplingResult {
        let trace_state = parent_context
            .map(
                |parent_context| parent_context.span().span_context().trace_state().clone(), // inherit sample decision from parent span
            )
            .unwrap_or_else(|| {
                DatadogTraceStateBuilder::default()
                    .with_priority_sampling(true) // always sample root span(span without remote or local parent)
                    .with_measuring(true) // datadog-agent will create metric for this span for APM
                    .build()
            });
        SamplingResult {
            decision: opentelemetry::trace::SamplingDecision::RecordAndSample, // send all spans to datadog-agent
            attributes: vec![],
            trace_state,
        }
    }
}