use std::borrow::Cow;
use std::sync::Arc;
use opentelemetry::InstrumentationScope;
use crate::domain::telemetry::Level;
use crate::ports::logger::{Logger, LoggerProvider};
use super::log::DatadogLoggerProvider;
use opentelemetry_sdk::logs::SdkLogger;

/// Adapter that implements our Logger port using the SDK Logger
pub struct DatadogLoggerAdapter {
    provider: Arc<DatadogLoggerProvider>,
    logger: SdkLogger,
}

impl DatadogLoggerAdapter {
    /// Create a new DatadogLoggerAdapter
    pub fn new(
        service_name: &str,
        version: &str,
        environment: &str,
        endpoint: &str,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let provider = Arc::new(DatadogLoggerProvider::new(
            service_name,
            version,
            environment,
            endpoint,
        )?);
        
        let logger = provider.logger("default");
        
        Ok(Self {
            provider,
            logger,
        })
    }
}

impl Logger for DatadogLoggerAdapter {
    fn log_with_level(&self, level: Level, message: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut record = self.logger.create_log_record();
        
        // Convert our Level to OpenTelemetry Severity
        record.severity = Some(match level {
            Level::Trace => opentelemetry::logs::Severity::Trace,
            Level::Debug => opentelemetry::logs::Severity::Debug,
            Level::Info => opentelemetry::logs::Severity::Info,
            Level::Warn => opentelemetry::logs::Severity::Warn,
            Level::Error => opentelemetry::logs::Severity::Error,
        });
        
        // Set the message and timestamp
        record.body = Some(message.into());
        record.timestamp = Some(std::time::SystemTime::now());
        
        // Emit the log
        self.logger.emit(record);
        
        Ok(())
    }

    fn log(&self, event: crate::domain::TelemetryEvent) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut record = self.logger.create_log_record();
        
        // Convert our Level to OpenTelemetry Severity
        record.severity = Some(match event.level {
            Level::Trace => opentelemetry::logs::Severity::Trace,
            Level::Debug => opentelemetry::logs::Severity::Debug,
            Level::Info => opentelemetry::logs::Severity::Info,
            Level::Warn => opentelemetry::logs::Severity::Warn,
            Level::Error => opentelemetry::logs::Severity::Error,
        });
        
        // Set the message and timestamp
        record.body = Some(event.name.into());
        record.timestamp = event.timestamp;
        
        // Add attributes
        for attr in event.attributes {
            record.attributes.push(attr);
        }
        
        // Emit the log
        self.logger.emit(record);
        
        Ok(())
    }
    
    fn shutdown(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.provider.shutdown()
    }
}

/// Provider adapter that implements our LoggerProvider port using the DatadogLoggerProvider
pub struct DatadogLoggerProviderAdapter {
    provider: Arc<DatadogLoggerProvider>,
}

impl DatadogLoggerProviderAdapter {
    /// Create a new DatadogLoggerProviderAdapter
    pub fn new(
        service_name: &str,
        version: &str,
        environment: &str,
        endpoint: &str,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let provider = Arc::new(DatadogLoggerProvider::new(
            service_name,
            version,
            environment,
            endpoint,
        )?);
        
        Ok(Self { provider })
    }
}

impl LoggerProvider for DatadogLoggerProviderAdapter {
    type LoggerImpl = DatadogLoggerAdapter;
    
    fn logger(&self, name: impl Into<Cow<'static, str>>) -> Self::LoggerImpl {
        let logger = self.provider.logger(name);
        
        DatadogLoggerAdapter {
            provider: self.provider.clone(),
            logger,
        }
    }
    
    fn logger_with_scope(&self, scope: InstrumentationScope) -> Self::LoggerImpl {
        let logger = self.provider.logger_with_scope(scope);
        
        DatadogLoggerAdapter {
            provider: self.provider.clone(),
            logger,
        }
    }
    
    fn shutdown(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.provider.shutdown()
    }
}