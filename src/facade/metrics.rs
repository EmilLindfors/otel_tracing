//! Facade for metrics functionality.
//! 
//! This module provides functions for creating and updating metrics.

use crate::domain::metrics::MetricUnit;
use crate::domain::telemetry::{MetricContext, AttributeValue};
use crate::ports::metrics::{Counter, Gauge, Histogram};
use super::service;

/// Create a new counter.
pub fn create_counter(context: MetricContext) -> Box<dyn Counter> {
    service().create_counter(context)
}

/// Create a new gauge.
pub fn create_gauge(context: MetricContext) -> Box<dyn Gauge> {
    service().create_gauge(context)
}

/// Create a new histogram.
pub fn create_histogram(context: MetricContext) -> Box<dyn Histogram> {
    service().create_histogram(context)
}

/// Create a counter metric with fixed initial attributes.
/// The counter can be incremented with additional attributes.
pub fn create_counter_with_attributes(
    name: &str, 
    description: Option<&str>, 
    unit: Option<MetricUnit>,
    attributes: Vec<(String, AttributeValue)>
) -> Box<dyn Counter> {
    create_counter(MetricContext {
        name: name.to_string(),
        description: description.map(|s| s.to_string()),
        unit,
        attributes,
    })
}

/// Create a gauge metric with fixed initial attributes.
/// The gauge can be updated with additional attributes.
pub fn create_gauge_with_attributes(
    name: &str, 
    description: Option<&str>, 
    unit: Option<MetricUnit>,
    attributes: Vec<(String, AttributeValue)>
) -> Box<dyn Gauge> {
    create_gauge(MetricContext {
        name: name.to_string(),
        description: description.map(|s| s.to_string()),
        unit,
        attributes,
    })
}

/// Create a histogram metric with fixed initial attributes.
/// The histogram can record values with additional attributes.
pub fn create_histogram_with_attributes(
    name: &str, 
    description: Option<&str>, 
    unit: Option<MetricUnit>, 
    attributes: Vec<(String, AttributeValue)>
) -> Box<dyn Histogram> {
    create_histogram(MetricContext {
        name: name.to_string(),
        description: description.map(|s| s.to_string()),
        unit,
        attributes,
    })
}
