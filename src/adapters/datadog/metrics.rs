use async_trait::async_trait;
use opentelemetry::global;
use opentelemetry::metrics::MeterProvider;
use opentelemetry::metrics::{
    Counter as OtelCounter, Gauge as OtelGauge, Histogram as OtelHistogram, Meter,
};
use opentelemetry::KeyValue;
use opentelemetry_otlp::MetricExporter;
use opentelemetry_sdk::metrics::{SdkMeterProvider, Temporality};
use opentelemetry_sdk::Resource;
use std::sync::Mutex;
use tracing::{debug, info};

use crate::domain::telemetry::{
    get_resource, to_key_value, AttributeValue, MetricContext, TelemetryError,
};
use crate::ports::metrics::{Counter, Gauge, Histogram, MetricsPort};

pub fn merge_with_system_tags(
    mut attributes: Vec<(String, AttributeValue)>,
) -> Vec<(String, AttributeValue)> {
    // Check if env tag is already present
    if !attributes.iter().any(|(k, _)| k == "env") {
        if let Ok(env) = std::env::var("DD_ENV") {
            attributes.push(("env".to_string(), AttributeValue::String(env)));
        }
    }

    // Same for service and version
    if !attributes.iter().any(|(k, _)| k == "service") {
        if let Ok(service) = std::env::var("DD_SERVICE") {
            attributes.push(("service".to_string(), AttributeValue::String(service)));
        }
    }

    if !attributes.iter().any(|(k, _)| k == "version") {
        if let Ok(version) = std::env::var("DD_VERSION") {
            attributes.push(("version".to_string(), AttributeValue::String(version)));
        }
    }

    attributes
}

#[derive(bon::Builder)]
pub struct DatadogMetrics {
    counter_meter_provider: Mutex<Option<SdkMeterProvider>>,
    gauge_meter_provider: Mutex<Option<SdkMeterProvider>>,
    histogram_meter_provider: Mutex<Option<SdkMeterProvider>>,
    resource: Option<Resource>,
}

impl DatadogMetrics {
    pub fn new() -> Self {
        Self {
            counter_meter_provider: Mutex::new(None),
            gauge_meter_provider: Mutex::new(None),
            histogram_meter_provider: Mutex::new(None),
            resource: None,
        }
    }

    fn convert_attributes(attributes: &[(String, AttributeValue)]) -> Vec<KeyValue> {
        attributes
            .iter()
            .map(|(key, value)| to_key_value(key.to_string(), value))
            .collect()
    }

    // Format metric name according to DataDog conventions
    fn format_metric_name(&self, name: &str) -> String {
        // DataDog prefers lowercase names with dots as separators
        let name = name.to_lowercase().replace('_', ".");

        // Prefix with namespace if not already prefixed
        if !name.contains('.') {
            format!("custom.{}", name)
        } else {
            name
        }
    }
}

#[async_trait]
impl MetricsPort for DatadogMetrics {
    async fn init(&self) -> Result<(), TelemetryError> {
        // Add DataDog-specific resource attributes
        let resource = get_resource();

        // Counter provider with Delta temporality
        let counter_exporter = MetricExporter::builder()
            .with_tonic()
            .with_temporality(Temporality::Delta)
            .build()
            .map_err(|e| TelemetryError::MetricsInitError(e.to_string()))?;

        let counter_provider = SdkMeterProvider::builder()
            .with_resource(resource.clone())
            .with_periodic_exporter(counter_exporter)
            .build();

        // Gauge provider with Cumulative temporality
        let gauge_exporter = MetricExporter::builder()
            .with_tonic()
            .with_temporality(Temporality::Cumulative)
            .build()
            .map_err(|e| TelemetryError::MetricsInitError(e.to_string()))?;

        let gauge_provider = SdkMeterProvider::builder()
            .with_resource(resource.clone())
            .with_periodic_exporter(gauge_exporter)
            .build();

        // Histogram provider with Delta temporality
        let histogram_exporter = MetricExporter::builder()
            .with_tonic()
            .with_temporality(Temporality::Delta)
            .build()
            .map_err(|e| TelemetryError::MetricsInitError(e.to_string()))?;

        let histogram_provider = SdkMeterProvider::builder()
            .with_resource(resource)
            .with_periodic_exporter(histogram_exporter)
            .build();

        // Store providers for shutdown
        *self.counter_meter_provider.lock().unwrap() = Some(counter_provider.clone());
        *self.gauge_meter_provider.lock().unwrap() = Some(gauge_provider.clone());
        *self.histogram_meter_provider.lock().unwrap() = Some(histogram_provider.clone());

        // Set global meter provider (optional - we'll use specific ones in methods)
        global::set_meter_provider(counter_provider);

        Ok(())
    }

    fn create_counter(&self, context: MetricContext) -> Box<dyn Counter> {
        // Apply Datadog naming conventions
        let metric_name = self.format_metric_name(&context.name);

        // Get meter from counter provider
        let meter = match self.counter_meter_provider.lock().unwrap().as_ref() {
            Some(provider) => provider.meter("datadog-metrics"),
            None => global::meter("datadog-metrics"), // Fallback
        };

        // Merge system tags with passed attributes
        let attributes = merge_with_system_tags(context.attributes);

        let counter_builder = meter.u64_counter(metric_name);

        let counter_builder = if let Some(desc) = context.description {
            counter_builder.with_description(desc)
        } else {
            counter_builder
        };

        let counter_builder = if let Some(unit) = context.unit {
            counter_builder.with_unit(unit.as_str().to_string())
        } else {
            counter_builder
        };

        let counter = counter_builder.build();

        Box::new(DatadogCounter {
            counter,
            name: context.name,
            default_attributes: attributes
                .iter()
                .map(|(k, v)| to_key_value(k.to_string(), v))
                .collect(),
        })
    }

    fn create_gauge(&self, context: MetricContext) -> Box<dyn Gauge> {
        // Apply DataDog naming conventions
        let metric_name = self.format_metric_name(&context.name);

        // Get meter from gauge provider
        let meter = match self.gauge_meter_provider.lock().unwrap().as_ref() {
            Some(provider) => provider.meter("datadog-metrics"),
            None => global::meter("datadog-metrics"), // Fallback
        };

        // Merge system tags with passed attributes
        let attributes = merge_with_system_tags(context.attributes);

        let gauge_builder = meter.f64_gauge(metric_name);

        let gauge_builder = if let Some(desc) = context.description {
            gauge_builder.with_description(desc)
        } else {
            gauge_builder
        };

        let gauge_builder = if let Some(unit) = context.unit {
            gauge_builder.with_unit(unit.as_str().to_string())
        } else {
            gauge_builder
        };

        let gauge = gauge_builder.build();

        Box::new(DatadogGauge {
            gauge,
            name: context.name.clone(),
            default_attributes: attributes
                .iter()
                .map(|(k, v)| to_key_value(k.to_string(), v))
                .collect(),
        })
    }

    fn create_histogram(&self, context: MetricContext) -> Box<dyn Histogram> {
        // Apply DataDog naming conventions
        let metric_name = self.format_metric_name(&context.name);

        // Get meter from histogram provider
        let meter = match self.histogram_meter_provider.lock().unwrap().as_ref() {
            Some(provider) => provider.meter("datadog-metrics"),
            None => global::meter("datadog-metrics"), // Fallback
        };

        // Merge system tags with passed attributes
        let attributes = merge_with_system_tags(context.attributes);



        let histogram_builder = meter.f64_histogram(metric_name);

        let histogram_builder = if let Some(desc) = context.description {
            histogram_builder.with_description(desc)
        } else {
            histogram_builder
        };

        let histogram_builder = if let Some(unit) = context.unit {
            histogram_builder.with_unit(unit.as_str().to_string())
        } else {
            histogram_builder
        };

        let histogram = histogram_builder.build();

        Box::new(DatadogHistogram {
            histogram,
            name: context.name,
            default_attributes: attributes
                .iter()
                .map(|(k, v)| to_key_value(k.to_string(), v))
                .collect(),
        })
    }

    async fn shutdown(&self) -> Result<(), TelemetryError> {
        info!("Shutting down DatadogMetrics");

        let mut provider = self.counter_meter_provider.lock().unwrap();
        if let Some(provider) = provider.take() {
            provider
                .shutdown()
                .map_err(|e| TelemetryError::ShutdownError(e.to_string()))?;
        }

        let mut provider = self.gauge_meter_provider.lock().unwrap();
        if let Some(provider) = provider.take() {
            provider
                .shutdown()
                .map_err(|e| TelemetryError::ShutdownError(e.to_string()))?;
        }

        let mut provider = self.histogram_meter_provider.lock().unwrap();
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
    name: String,
    default_attributes: Vec<KeyValue>,
}

impl Counter for DatadogCounter {
    fn add(&self, value: u64, attributes: Vec<(String, AttributeValue)>) {
        // Create a combined set of attributes - defaults plus provided ones
        let mut combined_attributes = self.default_attributes.clone();

        // Add the specific attributes for this call
        let call_attributes = DatadogMetrics::convert_attributes(&attributes);
        combined_attributes.extend(call_attributes);

        self.counter.add(value, &combined_attributes);
    }
}

struct DatadogGauge {
    gauge: OtelGauge<f64>,
    name: String,
    default_attributes: Vec<KeyValue>,
}

impl Gauge for DatadogGauge {
    fn set(&self, value: f64, attributes: Vec<(String, AttributeValue)>) {
        // Create a combined set of attributes - defaults plus provided ones
        let mut combined_attributes = self.default_attributes.clone();

        // Add the specific attributes for this call
        let call_attributes = DatadogMetrics::convert_attributes(&attributes);
        combined_attributes.extend(call_attributes);

        self.gauge.record(value, &combined_attributes);
    }
}

struct DatadogHistogram {
    histogram: OtelHistogram<f64>,
    name: String,
    default_attributes: Vec<KeyValue>,
}

impl Histogram for DatadogHistogram {
    fn record(&self, value: f64, attributes: Vec<(String, AttributeValue)>) {
        // Create a combined set of attributes - defaults plus provided ones
        let mut combined_attributes = self.default_attributes.clone();

        // Add the specific attributes for this call
        let call_attributes = DatadogMetrics::convert_attributes(&attributes);
        combined_attributes.extend(call_attributes);

        self.histogram.record(value, &combined_attributes);
    }
}
