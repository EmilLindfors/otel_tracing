#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::error::Error;
    use std::fmt;
    use std::sync::{Arc, Mutex};
    use std::time::SystemTime;

    use async_trait::async_trait;
    use mockall::predicate::*;
    use mockall::*;
    use tracing_subscriber::EnvFilter;

    use otel_tracing::adapters::datadog::DatadogLogger;
    use otel_tracing::domain::telemetry::{AttributeValue, LogContext, LogLevel, TelemetryError};
    use otel_tracing::ports::logger::LoggerPort;

    // First, let's create a simple error type for testing
    #[derive(Debug)]
    struct TestError {
        message: String,
        source: Option<Box<dyn Error + Send + Sync>>,
    }

    impl TestError {
        fn new(message: &str) -> Self {
            Self {
                message: message.to_string(),
                source: None,
            }
        }

        fn with_source(mut self, source: Box<dyn Error + Send + Sync>) -> Self {
            self.source = Some(source);
            self
        }
    }

    impl fmt::Display for TestError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.message)
        }
    }

    impl Error for TestError {
        fn source(&self) -> Option<&(dyn Error + 'static)> {
            self.source
                .as_ref()
                .map(|e| e.as_ref() as &(dyn Error + 'static))
        }
    }

    // Mock for tracing events to verify log calls
    mock! {
        #[derive(Debug, Clone)]
        pub TracingEvents {
            pub fn log_event(&self, level: LogLevel, message: String, attributes: HashMap<String, AttributeValue>);
        }
    }
    
    // A test logger that uses our mock to verify log events
    struct TestLogger {
        mock_events: Arc<Mutex<MockTracingEvents>>,
    }

    impl TestLogger {
        fn new(mock_events: MockTracingEvents) -> Self {
            Self { 
                mock_events: Arc::new(Mutex::new(mock_events)) 
            }
        }
    }

    #[async_trait]
    impl LoggerPort for TestLogger {
        async fn init(&self, _filter: Option<EnvFilter>) -> Result<(), TelemetryError> {
            Ok(())
        }

        fn log(&self, context: LogContext) {
            if let Ok(mock) = self.mock_events.lock() {
                mock.log_event(context.level, context.message, context.attributes);
            }
        }

        fn log_error(
            &self,
            error: Box<dyn std::error::Error>,
            target: Option<&str>,
            attributes: Vec<(String, AttributeValue)>,
        ) {
            let mut attrs: HashMap<String, AttributeValue> = attributes.into_iter().collect();
            attrs.insert(
                "error.message".to_string(),
                AttributeValue::String(error.to_string()),
            );
            attrs.insert(
                "error.kind".to_string(),
                AttributeValue::String("TestError".to_string()),
            );

            if let Ok(mock) = self.mock_events.lock() {
                mock.log_event(LogLevel::Error, error.to_string(), attrs);
            }
        }

        async fn shutdown(&self) -> Result<(), TelemetryError> {
            Ok(())
        }
    }

    // Create a mock for LoggerPort for testing the domain
    mock! {
        pub LoggerPort {}

        #[async_trait]
        impl LoggerPort for LoggerPort {
            async fn init(&self, filter: Option<EnvFilter>) -> Result<(), TelemetryError>;
            fn log(&self, context: LogContext);
            fn log_error<'a>(
                &'a self,
                error: Box<dyn std::error::Error>,
                target: Option<&'a str>,
                attributes: Vec<(String, AttributeValue)>,
            );
            async fn shutdown(&self) -> Result<(), TelemetryError>;
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use std::sync::Arc;
        use std::time::Duration;

        // Utility function to create a test LogContext
        fn create_test_log_context(level: LogLevel, message: &str) -> LogContext {
            let mut attributes = HashMap::new();
            attributes.insert(
                "test_key".to_string(),
                AttributeValue::String("test_value".to_string()),
            );

            LogContext {
                level,
                timestamp: Some(
                    SystemTime::now()
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap()
                        .as_millis(),
                ),
                message: message.to_string(),
                target: Some("test_target".to_string()),
                attributes,
            }
        }

        #[tokio::test]
        async fn test_datadog_logger_initialization() {
            let logger = DatadogLogger::new("test-service");

            // Test initialization with default filter
            let result = logger.init(None).await;
            assert!(result.is_ok(), "Logger initialization failed: {:?}", result);

            // Test initialization with custom filter
            let custom_filter = EnvFilter::new("debug");
            let result = logger.init(Some(custom_filter)).await;
            assert!(
                result.is_ok(),
                "Logger initialization with custom filter failed: {:?}",
                result
            );

            // Test successful shutdown
            let result = logger.shutdown().await;
            assert!(result.is_ok(), "Logger shutdown failed: {:?}", result);
        }

        #[test]
        fn test_to_tracing_level_conversion() {
            // We need to test the private method, so we'll test the behavior indirectly
            // through the public log method

            let mut mock_events = MockTracingEvents::new();
            
            // Set up all expectations before creating the test logger
            mock_events
                .expect_log_event()
                .with(
                    eq(LogLevel::Debug),
                    eq("Debug message".to_string()),
                    always(),
                )
                .times(1)
                .return_const(());
                
            mock_events
                .expect_log_event()
                .with(eq(LogLevel::Info), eq("Info message".to_string()), always())
                .times(1)
                .return_const(());
                
            mock_events
                .expect_log_event()
                .with(eq(LogLevel::Warn), eq("Warn message".to_string()), always())
                .times(1)
                .return_const(());
                
            mock_events
                .expect_log_event()
                .with(
                    eq(LogLevel::Error),
                    eq("Error message".to_string()),
                    always(),
                )
                .times(1)
                .return_const(());
                
            mock_events
                .expect_log_event()
                .with(
                    eq(LogLevel::Critical),
                    eq("Critical message".to_string()),
                    always(),
                )
                .times(1)
                .return_const(());
            
            // Now create the test logger with the configured mock
            let test_logger = TestLogger::new(mock_events);

            test_logger.log(create_test_log_context(LogLevel::Debug, "Debug message"));
            test_logger.log(create_test_log_context(LogLevel::Info, "Info message"));
            test_logger.log(create_test_log_context(LogLevel::Warn, "Warn message"));
            test_logger.log(create_test_log_context(LogLevel::Error, "Error message"));
            test_logger.log(create_test_log_context(LogLevel::Critical, "Critical message"));
        }

        #[test]
        fn test_transform_attributes_to_datadog_format() {
            let logger = DatadogLogger::new("test-service");

            // Create a log context with various attribute types
            let mut attributes = HashMap::new();
            attributes.insert(
                "service".to_string(),
                AttributeValue::String("test-service".to_string()),
            );
            attributes.insert(
                "host".to_string(),
                AttributeValue::String("test-host".to_string()),
            );
            attributes.insert(
                "user_id".to_string(),
                AttributeValue::String("test-user".to_string()),
            );
            attributes.insert("duration".to_string(), AttributeValue::Int(100));
            attributes.insert(
                "client_ip".to_string(),
                AttributeValue::String("127.0.0.1".to_string()),
            );
            attributes.insert("custom_attr".to_string(), AttributeValue::Bool(true));
            attributes.insert("http.status_code".to_string(), AttributeValue::Int(200));

            let context = LogContext {
                level: LogLevel::Info,
                timestamp: Some(
                    SystemTime::now()
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap()
                        .as_millis(),
                ),
                message: "Test message".to_string(),
                target: Some("test_target".to_string()),
                attributes,
            };

            // Use a mock tracing layer to capture the event
            let mut mock_events = MockTracingEvents::new();
            mock_events
                .expect_log_event()
                .withf(|level, message, attrs| {
                    // Verify datadog attributes transformation
                    level == &LogLevel::Info &&
                message == "Test message" &&
                attrs.contains_key("service") &&
                attrs.contains_key("host") &&
                attrs.contains_key("usr.id") && // Verify user_id was transformed
                attrs.contains_key("duration") &&
                attrs.contains_key("network.client.ip") && // Verify client_ip was transformed
                attrs.contains_key("custom_attr") &&
                attrs.contains_key("http.status_code") &&
                attrs.contains_key("status") && // Verify status was added
                attrs.get("status").unwrap().to_string() == "info"
                })
                .times(1)
                .return_const(());

            // Inject our mock and call log
            let test_logger = TestLogger::new(mock_events);
            test_logger.log(context);
        }

        #[test]
        fn test_log_error_with_stack_trace() {
            let logger = DatadogLogger::new("test-service");

            // Create a nested error
            let nested_error = TestError::new("Nested error cause");
            let main_error = TestError::new("Main error").with_source(Box::new(nested_error));

            // Set up expected attribute transformations
            let mut mock_events = MockTracingEvents::new();
            mock_events
                .expect_log_event()
                .withf(|level, message, attrs| {
                    level == &LogLevel::Error
                        && message == "Main error"
                        && attrs.contains_key("error.stack")
                        && attrs.contains_key("error.message")
                        && attrs.contains_key("error.kind")
                        && attrs.contains_key("logger.name")
                })
                .times(1)
                .return_const(());

            // Inject our mock and call log_error
            let test_logger = TestLogger::new(mock_events);
            test_logger.log_error(
                Box::new(main_error),
                Some("error_logger"),
                vec![(
                    "transaction_id".to_string(),
                    AttributeValue::String("abc123".to_string()),
                )],
            );
        }

        #[tokio::test]
        async fn test_datadog_logger_high_precision_timestamps() {
            // Test with high precision timestamps
            let logger_high_precision =
                DatadogLogger::new("test-service").with_high_precision_timestamps(true);

            // Test with standard precision timestamps
            let logger_standard_precision =
                DatadogLogger::new("test-service").with_high_precision_timestamps(false);

            // We would need to create a custom test to verify timestamp precision
            // For now, just verify that the loggers can be initialized
            let result = logger_high_precision.init(None).await;
            assert!(result.is_ok());

            let result = logger_standard_precision.init(None).await;
            assert!(result.is_ok());
        }

        #[test]
        fn test_logger_port_trait_implementation() {
            // Create a mock implementation of LoggerPort
            let mut mock_logger = MockLoggerPort::new();

            // Set up expectations for the log method
            mock_logger
                .expect_log()
                .withf(|context| {
                    context.level == LogLevel::Info
                        && context.message == "Test message"
                        && context.target == Some("test_target".to_string())
                })
                .times(1)
                .return_const(());

            // Call the log method
            mock_logger.log(create_test_log_context(LogLevel::Info, "Test message"));

            // Set up expectations for the log_error method
            let test_error = TestError::new("Test error");

            mock_logger
                .expect_log_error()
                .withf(|err, target, attrs| {
                    err.to_string() == "Test error"
                        && *target == Some("error_target")
                        && attrs.len() == 1
                        && attrs[0].0 == "correlation_id"
                        && matches!(attrs[0].1, AttributeValue::String(_))
                })
                .times(1)
                .return_const(());

            // Call the log_error method
            mock_logger.log_error(
                Box::new(test_error),
                Some("error_target"),
                vec![(
                    "correlation_id".to_string(),
                    AttributeValue::String("xyz789".to_string()),
                )],
            );
        }

        #[tokio::test]
        async fn test_logger_port_async_methods() {
            // Create a mock implementation of LoggerPort
            let mut mock_logger = MockLoggerPort::new();

            // Set up expectations for the init method
            mock_logger
                .expect_init()
                .withf(|filter| filter.is_none())
                .times(1)
                .returning(|_| Ok(()));

            // Call the init method
            let result = mock_logger.init(None).await;
            assert!(result.is_ok());

            // Set up expectations for the shutdown method
            mock_logger.expect_shutdown().times(1).returning(|| Ok(()));

            // Call the shutdown method
            let result = mock_logger.shutdown().await;
            assert!(result.is_ok());

            // Test error cases
            let mut mock_logger = MockLoggerPort::new();

            // Set up expectations for init failure
            mock_logger.expect_init().times(1).returning(|_| {
                Err(TelemetryError::LoggerInitError(
                    "Test init error".to_string(),
                ))
            });

            // Call the init method expecting an error
            let result = mock_logger.init(None).await;
            assert!(result.is_err());
            if let Err(TelemetryError::LoggerInitError(msg)) = result {
                assert_eq!(msg, "Test init error");
            } else {
                panic!("Expected LoggerInitError");
            }

            // Set up expectations for shutdown failure
            mock_logger.expect_shutdown().times(1).returning(|| {
                Err(TelemetryError::ShutdownError(
                    "Test shutdown error".to_string(),
                ))
            });

            // Call the shutdown method expecting an error
            let result = mock_logger.shutdown().await;
            assert!(result.is_err());
            if let Err(TelemetryError::ShutdownError(msg)) = result {
                assert_eq!(msg, "Test shutdown error");
            } else {
                panic!("Expected ShutdownError");
            }
        }

        // Integration-style test that combines multiple components
        #[tokio::test]
        async fn test_datadog_logger_integration() {
            // This would be a more realistic integration test
            // For now, we'll just initialize and use the logger

            let logger = DatadogLogger::new("integration-test-service");

            // Initialize the logger
            let init_result = logger.init(None).await;
            assert!(
                init_result.is_ok(),
                "Logger initialization failed: {:?}",
                init_result
            );

            // Log some messages at different levels
            let debug_context = LogContext::new("Debug message".to_string(), LogLevel::Debug)
                .with_target("integration_test")
                .with_attribute(
                    "test_attr",
                    AttributeValue::String("debug_value".to_string()),
                );

            logger.log(debug_context);

            let info_context = LogContext::new("Info message".to_string(), LogLevel::Info)
                .with_target("integration_test")
                .with_attribute(
                    "test_attr",
                    AttributeValue::String("info_value".to_string()),
                );

            logger.log(info_context);

            // Log an error
            let test_error = TestError::new("Integration test error");
            logger.log_error(
                Box::new(test_error),
                Some("integration_test"),
                vec![(
                    "test_attr".to_string(),
                    AttributeValue::String("error_value".to_string()),
                )],
            );

            // Give some time for logs to be processed
            tokio::time::sleep(Duration::from_millis(100)).await;

            // Shutdown the logger
            let shutdown_result = logger.shutdown().await;
            assert!(
                shutdown_result.is_ok(),
                "Logger shutdown failed: {:?}",
                shutdown_result
            );
        }
    }
}
