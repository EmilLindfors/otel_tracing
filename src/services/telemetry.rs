use std::sync::Arc;

use crate::domain::telemetry::{LogContext, MetricContext, SpanContext, TelemetryError};
use crate::ports::logger::LoggerPort;
use crate::ports::metrics::{Counter, Gauge, Histogram, MetricsPort};
use crate::ports::tracer::{Span, TracerPort};

/// TelemetryService provides a unified interface for tracing, metrics, and logging
pub struct TelemetryService {
    tracer: Arc<dyn TracerPort>,
    metrics: Arc<dyn MetricsPort>,
    logger: Arc<dyn LoggerPort>,
}

impl TelemetryService {
    /// Create a new TelemetryService with the given tracer, metrics, and logger implementations
    pub fn new(
        tracer: Arc<dyn TracerPort>,
        metrics: Arc<dyn MetricsPort>,
        logger: Arc<dyn LoggerPort>,
    ) -> Self {
        Self {
            tracer,
            metrics,
            logger,
        }
    }

    /// Initialize all telemetry components
    pub async fn init(&self) -> Result<(), TelemetryError> {
        // Initialize logger first, so we can capture logs from other initializations
        self.logger.init().await?;
        self.tracer.init().await?;
        self.metrics.init().await?;

        Ok(())
    }

    /// Shutdown all telemetry components
    pub async fn shutdown(&self) -> Result<(), TelemetryError> {
        let mut errors = Vec::new();

        if let Err(e) = self.tracer.shutdown().await {
            errors.push(format!("Tracer shutdown error: {}", e));
        }

        if let Err(e) = self.metrics.shutdown().await {
            errors.push(format!("Metrics shutdown error: {}", e));
        }

        if let Err(e) = self.logger.shutdown().await {
            errors.push(format!("Logger shutdown error: {}", e));
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(TelemetryError::ShutdownError(errors.join("; ")))
        }
    }

    /// Create a new span
    pub fn create_span(&self, context: SpanContext) -> Box<dyn Span> {
        self.tracer.create_span(context)
    }

    /// Create a new counter
    pub fn create_counter(&self, context: MetricContext) -> Box<dyn Counter> {
        self.metrics.create_counter(context)
    }

    /// Create a new gauge
    pub fn create_gauge(&self, context: MetricContext) -> Box<dyn Gauge> {
        self.metrics.create_gauge(context)
    }

    /// Create a new histogram
    pub fn create_histogram(&self, context: MetricContext) -> Box<dyn Histogram> {
        self.metrics.create_histogram(context)
    }

    /// Log a message
    pub fn log(&self, context: LogContext) {
        self.logger.log(context)
    }
}

/// A builder for configuring and creating a TelemetryService
pub struct TelemetryServiceBuilder {
    tracer: Option<Arc<dyn TracerPort>>,
    metrics: Option<Arc<dyn MetricsPort>>,
    logger: Option<Arc<dyn LoggerPort>>,
}

impl TelemetryServiceBuilder {
    /// Create a new TelemetryServiceBuilder
    pub fn new() -> Self {
        Self {
            tracer: None,
            metrics: None,
            logger: None,
        }
    }

    /// Set the tracer implementation
    pub fn with_tracer(mut self, tracer: impl TracerPort + 'static) -> Self {
        self.tracer = Some(Arc::new(tracer));
        self
    }

    /// Set the metrics implementation
    pub fn with_metrics(mut self, metrics: impl MetricsPort + 'static) -> Self {
        self.metrics = Some(Arc::new(metrics));
        self
    }

    /// Set the logger implementation
    pub fn with_logger(mut self, logger: impl LoggerPort + 'static) -> Self {
        self.logger = Some(Arc::new(logger));
        self
    }

    /// Build the TelemetryService
    pub fn build(self) -> Result<TelemetryService, TelemetryError> {
        let tracer = self
            .tracer
            .ok_or_else(|| TelemetryError::TracerInitError("No tracer provided".to_string()))?;

        let metrics = self.metrics.ok_or_else(|| {
            TelemetryError::MetricsInitError("No metrics provider provided".to_string())
        })?;

        let logger = self
            .logger
            .ok_or_else(|| TelemetryError::LoggerInitError("No logger provided".to_string()))?;

        Ok(TelemetryService::new(tracer, metrics, logger))
    }

    /// Build a DataDog-based TelemetryService with default configuration
    pub fn build_datadog() -> Result<TelemetryService, TelemetryError> {
        use crate::adapters::datadog::{DatadogLogger, DatadogMetrics, DatadogTracer};

        let tracer = Arc::new(DatadogTracer::new());
        let metrics = Arc::new(DatadogMetrics::new());
        let logger = Arc::new(DatadogLogger::new());

        Ok(TelemetryService::new(tracer, metrics, logger))
    }
}
