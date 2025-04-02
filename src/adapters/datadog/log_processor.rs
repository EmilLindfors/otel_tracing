use crate::adapters::datadog::formatter::DatadogId;
use opentelemetry::InstrumentationScope;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::logs::SdkLogRecord;
use opentelemetry_sdk::logs::log_processor_with_async_runtime::LogProcessor;

/// A log processor that exports logs to Datadog via OTLP
#[derive(Debug)]
pub struct DatadogLogProcessor {
    exporter: opentelemetry_otlp::LogExporter,
}

impl DatadogLogProcessor {
    /// Create a new DatadogLogProcessor with the given OTLP endpoint
    pub fn new(endpoint: &str) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let exporter = opentelemetry_otlp::LogExporter::builder()
            .with_tonic()
            .with_endpoint(endpoint)
            .with_timeout(std::time::Duration::from_secs(5))
            .build()?;
            
        Ok(Self { exporter })
    }
    
    /// Enhance the log record with Datadog-specific attributes for trace correlation
    fn add_datadog_attributes(&self, record: &mut SdkLogRecord) {
        // Get the trace context if available
        if let Some(trace_ctx) = record.trace_context().cloned() {
            // Convert trace ID to Datadog format
            let dd_trace_id = DatadogId::from(trace_ctx.trace_id);
            record.add_attribute("dd.trace_id", format!("{}", dd_trace_id.inner()));
            
            // Convert span ID to Datadog format
            let dd_span_id = DatadogId::from(trace_ctx.span_id);
            record.add_attribute("dd.span_id", format!("{}", dd_span_id.inner()));
        }
    }
}

impl LogProcessor for DatadogLogProcessor {
    fn emit(&self, data: &mut SdkLogRecord, instrumentation: &InstrumentationScope) {
        // Add Datadog-specific attributes for trace correlation
        self.add_datadog_attributes(data);
        
        // Use the OTLP exporter to send the log
        let result = self.exporter.export(&[data.clone()]);
        if let Err(err) = result {
            eprintln!("Failed to export log to Datadog: {}", err);
        }
    }
    
    fn force_flush(&self) -> opentelemetry_sdk::error::OTelSdkResult {
        self.exporter.force_flush()
    }
    
    fn shutdown(&self) -> opentelemetry_sdk::error::OTelSdkResult {
        self.exporter.shutdown()
    }
}