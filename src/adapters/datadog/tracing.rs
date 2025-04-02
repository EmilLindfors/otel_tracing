//! Trace and layer builders to export traces to the Datadog agent through tracing.
//!
//! This module contains functions for building a tracer with an exporter
//! to send traces to the Datadog agent and for creating tracing layers.

use opentelemetry::global;
use opentelemetry_datadog::{ApiVersion, DatadogPropagator};
use opentelemetry_sdk::trace::{self, RandomIdGenerator, Sampler, Tracer};
use std::time::Duration;
use tracing::Subscriber;
use tracing_opentelemetry::{OpenTelemetryLayer, PreSampledTracer};
use tracing_subscriber::registry::LookupSpan;

use super::trace::AgentBasedSampler;

/// Builder for configuring a Datadog tracer using tracing
pub struct TracingBuilder {
    service_name: String,
    version: String,
    agent_endpoint: String,
}

impl TracingBuilder {
    /// Create a new tracing builder with default values
    pub fn new(service_name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            service_name: service_name.into(),
            version: version.into(),
            agent_endpoint: "http://localhost:8126".to_string(),
        }
    }

    /// Set the Datadog agent endpoint
    pub fn with_agent_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.agent_endpoint = endpoint.into();
        self
    }

    /// Build the tracer
    pub fn build_tracer(&self) -> Result<Tracer, opentelemetry::trace::TraceError> {
        // Disable connection reuse with dd-agent to avoid "connection closed from server" errors
        let dd_http_client = reqwest::ClientBuilder::new()
            .pool_idle_timeout(Duration::from_millis(1))
            .build()
            .map_err(|e| opentelemetry::trace::TraceError::Other(e.to_string().into()))?;

        // Create a tracer with Datadog export
        let tracer = opentelemetry_datadog::new_pipeline()
            .with_http_client(dd_http_client)
            .with_service_name(&self.service_name)
            .with_api_version(ApiVersion::Version05)
            .with_agent_endpoint(&self.agent_endpoint)
            .with_trace_config(
                trace::Config::default()
                    .with_sampler(Sampler::ParentBased(Box::new(AgentBasedSampler)))
                    .with_id_generator(RandomIdGenerator::default()),
            )
            .install_batch()?;

        // Set up the Datadog propagator for distributed tracing
        global::set_text_map_propagator(DatadogPropagator::default());

        Ok(tracer)
    }

    /// Build a tracing layer with the tracer
    pub fn build_layer<S>(&self) -> Result<OpenTelemetryLayer<S, Tracer>, opentelemetry::trace::TraceError>
    where
        S: Subscriber + for<'span> LookupSpan<'span>,
        Tracer: PreSampledTracer + 'static,
    {
        let tracer = self.build_tracer()?;
        Ok(tracing_opentelemetry::layer().with_tracer(tracer))
    }
}