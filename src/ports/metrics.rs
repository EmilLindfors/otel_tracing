use async_trait::async_trait;

use crate::domain::telemetry::{MetricContext, AttributeValue, TelemetryError};

#[async_trait]
pub trait MetricsPort: Send + Sync {
    async fn init(&self) -> Result<(), TelemetryError>;
    
    fn create_counter(&self, context: MetricContext) -> Box<dyn Counter>;
    
    fn create_gauge(&self, context: MetricContext) -> Box<dyn Gauge>;
    
    fn create_histogram(&self, context: MetricContext) -> Box<dyn Histogram>;
    
    async fn shutdown(&self) -> Result<(), TelemetryError>;
}

pub trait Counter: Send + Sync {
    fn add(&self, value: u64, attributes: Vec<(String, AttributeValue)>);
}

pub trait Gauge: Send + Sync {
    fn set(&self, value: f64, attributes: Vec<(String, AttributeValue)>);
}

pub trait Histogram: Send + Sync {
    fn record(&self, value: f64, attributes: Vec<(String, AttributeValue)>);
}
