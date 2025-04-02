use opentelemetry::{
    global, logs::LoggerProvider as OtelLoggerProvider, InstrumentationScope, KeyValue,
};
use opentelemetry_sdk::logs::log_processor_with_async_runtime::BatchLogProcessor;
use opentelemetry_sdk::{
    logs::{LoggerProvider as SdkLoggerProvider, LoggerProviderBuilder, SdkLogger},
    runtime, Resource,
};
use std::borrow::Cow;
use std::sync::Arc;

use super::log_processor::DatadogLogProcessor;

/// Datadog Logger Provider that wraps the OpenTelemetry SDK Logger Provider
#[derive(Debug, Clone)]
pub struct DatadogLoggerProvider {
    provider: Arc<SdkLoggerProvider>,
}

impl DatadogLoggerProvider {
    /// Create a new Datadog logger provider
    pub fn new(
        service_name: &str,
        version: &str,
        environment: &str,
        endpoint: &str,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        // Create resource information to identify the service
        let resource = Resource::builder().with_attributes(vec![
            KeyValue::new("service.name", service_name.to_string()),
            KeyValue::new("service.version", version.to_string()),
            KeyValue::new("deployment.environment", environment.to_string()),
        ]);

        // Create the Datadog log processor
        let processor = DatadogLogProcessor::new(endpoint)?;

        // Create batch processor with Tokio runtime
        let runtime = runtime::Tokio;
        let batch_processor = BatchLogProcessor::builder(processor, runtime).build();

        // Create the SDK logger provider with our processor
        let provider = SdkLoggerProvider::builder()
            .with_resource(resource)
            .with_log_processor(batch_processor)
            .build();

        // Register the provider as global
        global::set_logger_provider(provider.clone());

        Ok(Self {
            provider: Arc::new(provider),
        })
    }

    /// Shutdown the provider
    pub fn shutdown(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.provider.shutdown()?;
        Ok(())
    }
}

impl OtelLoggerProvider for DatadogLoggerProvider {
    type Logger = SdkLogger;

    fn logger_with_scope(&self, scope: InstrumentationScope) -> Self::Logger {
        self.provider.logger_with_scope(scope)
    }
}
