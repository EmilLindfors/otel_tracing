use async_trait::async_trait;
use opentelemetry::global;
use opentelemetry::metrics::{
    Counter as OtelCounter, Gauge as OtelGauge, Histogram as OtelHistogram, Meter,
};
use opentelemetry::KeyValue;
use opentelemetry_otlp::MetricExporter;
use opentelemetry_sdk::metrics::{SdkMeterProvider, Temporality};
use std::sync::Mutex;

use crate::domain::telemetry::{
    get_resource, to_key_value, AttributeValue, MetricContext, TelemetryError,
};
use crate::ports::metrics::{Counter, Gauge, Histogram, MetricsPort};

pub struct DatadogMetrics {
    meter_provider: Mutex<Option<SdkMeterProvider>>,
}

impl DatadogMetrics {
    pub fn new() -> Self {
        Self {
            meter_provider: Mutex::new(None),
        }
    }

    fn convert_attributes(attributes: &[(String, AttributeValue)]) -> Vec<KeyValue> {
        attributes
            .iter()
            .map(|(key, value)| to_key_value(key.to_string(), value))
            .collect()
    }

    // Apply Datadog conventions to metric names
    fn format_metric_name(&self, name: &str) -> String {
        // Datadog prefers lowercase names with dots as separators
        // First convert spaces and underscores to dots
        let normalized = name.replace(' ', ".").replace('_', ".");

        // If name doesn't start with a namespace, add service name as namespace
        if !normalized.contains('.') {
            format!(
                "{}.{}",
                "emil-test",
                normalized.to_lowercase()
            )
        } else {
            normalized.to_lowercase()
        }
    }
}

#[async_trait]
impl MetricsPort for DatadogMetrics {
    async fn init(&self) -> Result<(), TelemetryError> {
        let resource = get_resource();

        let exporter = MetricExporter::builder()
            .with_tonic()
            .with_temporality(Temporality::Delta)
            .build()
            .map_err(|e| TelemetryError::MetricsInitError(e.to_string()))?;

        let meter_provider = SdkMeterProvider::builder()
            .with_resource(resource)
            .with_periodic_exporter(exporter)
            
            .build();

        // Set global meter provider
        global::set_meter_provider(meter_provider.clone());

        // Store provider for shutdown
        let mut provider = self.meter_provider.lock().unwrap();
        *provider = Some(meter_provider);

        Ok(())
    }
   
    fn create_counter(&self, context: MetricContext) -> Box<dyn Counter> {
        // Apply Datadog naming conventions
        let metric_name = self.format_metric_name(&context.name);
        println!("Creating counter: {} (from {})", metric_name, context.name);
        
        let meter = global::meter("datadog-metrics");
        
        let counter_builder = meter.u64_counter(metric_name);
        
        let counter_builder = if let Some(desc) = context.description {
            counter_builder.with_description(desc)
        } else {
            counter_builder
        };
        
        let counter_builder = if let Some(unit) = context.unit {
            counter_builder.with_unit(unit)
        } else {
            counter_builder
        };
        
        let counter = counter_builder.build();
        
        Box::new(DatadogCounter { 
            counter,
        })
    }
    
    fn create_gauge(&self, context: MetricContext) -> Box<dyn Gauge> {
        // Apply Datadog naming conventions
        let metric_name = self.format_metric_name(&context.name);
        println!("Creating gauge: {} (from {})", metric_name, context.name);
        
        let meter = global::meter("datadog-metrics");
        
        let gauge_builder = meter.f64_gauge(metric_name);
        
        let gauge_builder = if let Some(desc) = context.description {
            gauge_builder.with_description(desc)
        } else {
            gauge_builder
        };
        
        let gauge_builder = if let Some(unit) = context.unit {
            gauge_builder.with_unit(unit)
        } else {
            gauge_builder
        };
        
        let gauge = gauge_builder.build();
        
        Box::new(DatadogGauge { 
            gauge,
        })
    }
    
    fn create_histogram(&self, context: MetricContext) -> Box<dyn Histogram> {
        // Apply Datadog naming conventions
        let metric_name = self.format_metric_name(&context.name);
        println!("Creating histogram: {} (from {})", metric_name, context.name);
        
        let meter = global::meter("datadog-metrics");
        
        let histogram_builder = meter.f64_histogram(metric_name);
        
        let histogram_builder = if let Some(desc) = context.description {
            histogram_builder.with_description(desc)
        } else {
            histogram_builder
        };
        
        let histogram_builder = if let Some(unit) = context.unit {
            histogram_builder.with_unit(unit)
        } else {
            histogram_builder
        };
        
        let histogram = histogram_builder.build();
        
        Box::new(DatadogHistogram { 
            histogram,
        })
    }
    

    async fn shutdown(&self) -> Result<(), TelemetryError> {
        let mut provider = self.meter_provider.lock().unwrap();
        if let Some(provider) = provider.take() {
            provider
                .shutdown()
                .map_err(|e| TelemetryError::ShutdownError(e.to_string()))?;
        }

        Ok(())
    }
}

struct DatadogCounter {
    counter: OtelCounter<u64>,
}

impl Counter for DatadogCounter {
    fn add(&self, value: u64, attributes: Vec<(String, AttributeValue)>) {
        let otel_attributes = DatadogMetrics::convert_attributes(&attributes);
        self.counter.add(value, &otel_attributes);
    }
}

struct DatadogGauge {
    gauge: OtelGauge<f64>,
}

impl Gauge for DatadogGauge {
    fn set(&self, value: f64, attributes: Vec<(String, AttributeValue)>) {
        let otel_attributes = DatadogMetrics::convert_attributes(&attributes);
        self.gauge.record(value, &otel_attributes);
    }
}

struct DatadogHistogram {
    histogram: OtelHistogram<f64>,
}

impl Histogram for DatadogHistogram {
    fn record(&self, value: f64, attributes: Vec<(String, AttributeValue)>) {
        let otel_attributes = DatadogMetrics::convert_attributes(&attributes);
        self.histogram.record(value, &otel_attributes);
    }
}
