mod domain;
mod ports;
mod adapters;
mod services;
pub mod facade;

pub use facade as telemetry;
pub use domain::telemetry::{SpanContext, MetricContext, LogContext, AttributeValue, TelemetryError, LogLevel};

/// Create a span for tracing operations.
/// 
/// # Examples
/// 
/// ```
/// // Simple span with just a name
/// let span = span!("process_request");
/// 
/// // Span with attributes
/// let span = span!("process_request", 
///     "user_id" => "12345",
///     "request_id" => "abc-123"
/// );
/// 
/// // Remember to end the span when the operation is complete
/// span.end();
/// ```
#[macro_export]
macro_rules! span {
    ($name:expr) => {
        $crate::telemetry::create_span($crate::SpanContext {
            name: $name.to_string(),
            attributes: vec![],
        })
    };
    ($name:expr, $($key:expr => $value:expr),+ $(,)?) => {
        $crate::telemetry::create_span($crate::SpanContext {
            name: $name.to_string(),
            attributes: vec![
                $(($key.to_string(), $value.into())),+
            ],
        })
    };
}

/// Execute code within a span scope, automatically ending the span when done.
/// 
/// # Examples
/// 
/// ```
/// // Simple span with just a name
/// let result = with_span!("calculate_result", {
///     // Code to execute within the span
///     calculate_something(42)
/// });
/// 
/// // Span with attributes
/// let result = with_span!("process_request", 
///     "user_id" => "12345",
///     "request_id" => "abc-123",
///     {
///         // Code to execute within the span
///         process_request()
///     }
/// );
/// ```
#[macro_export]
macro_rules! with_span {
    ($name:expr, $code:expr) => {
        $crate::telemetry::with_span($name, vec![], || $code)
    };
    ($name:expr, $($key:expr => $value:expr),+, $code:expr) => {
        $crate::telemetry::with_span(
            $name,
            vec![$(($key.to_string(), $value.into())),+],
            || $code
        )
    };
}

/// Execute async code within a span scope, automatically ending the span when done.
/// 
/// # Examples
/// 
/// ```
/// // Simple span with just a name
/// let result = with_async_span!("fetch_data", async {
///     // Async code to execute within the span
///     fetch_data().await
/// }).await;
/// 
/// // Span with attributes
/// let result = with_async_span!("process_request", 
///     "user_id" => "12345",
///     "request_id" => "abc-123",
///     async {
///         // Async code to execute within the span
///         process_request().await
///     }
/// ).await;
/// ```
#[macro_export]
macro_rules! with_async_span {
    // Case 1: No attributes, just name and code (unchanged)
    ($name:expr, $code:expr) => {
        $crate::telemetry::with_async_span($name, vec![], $code)
    };
    
    // Case 2: With attributes in tuple array syntax
    ($name:expr, [$(($key:expr, $value:expr)),* $(,)*], $code:expr) => {
        $crate::telemetry::with_async_span(
            $name,
            vec![$(($key.to_string(), $value.into())),*],
            $code
        )
    };
}

/// Create a counter metric.
/// 
/// # Examples
/// 
/// ```
/// // Simple counter with just a name
/// let counter = counter!("requests_processed");
/// 
/// // Counter with description and unit
/// let counter = counter!("requests_processed", "Number of requests processed", "requests");
/// 
/// // Counter with attributes
/// let counter = counter!("requests_processed",
///     "Number of requests processed",
///     "requests",
///     "service" => "api",
///     "status" => "200"
/// );
/// 
/// // Increment the counter
/// counter.inc();
/// ```
#[macro_export]
macro_rules! counter {
    ($name:expr, $unit:expr) => {
        $crate::telemetry::create_counter($crate::MetricContext {
            name: $name.to_string(),
            description: None,
            unit: $unit,
            attributes: vec![],
        })
    };
    ($name:expr, $description:expr, $unit:expr) => {
        $crate::telemetry::create_counter($crate::MetricContext {
            name: $name.to_string(),
            description: Some($description.to_string()),
            unit: $unit,
            attributes: vec![],
        })
    };
    ($name:expr, $description:expr, $unit:expr, $($key:expr => $value:expr),+ $(,)?) => {
        $crate::telemetry::create_counter($crate::MetricContext {
            name: $name.to_string(),
            description: Some($description.to_string()),
            unit: $unit,
            attributes: vec![
                $(($key.to_string(), $value.into())),+
            ],
        })
    };
}

/// Create a gauge metric.
/// 
/// # Examples
/// 
/// ```
/// // Simple gauge with just a name
/// let gauge = gauge!("cpu_usage");
/// 
/// // Gauge with description and unit
/// let gauge = gauge!("cpu_usage", "CPU usage percentage", "percent");
/// 
/// // Gauge with attributes
/// let gauge = gauge!("cpu_usage",
///    "CPU usage percentage",
///   "percent",
///   "service" => "api",
///  "status" => "200"
/// );
/// 
/// // Set the gauge value
/// gauge.set(42.0);
/// ```
#[macro_export]
macro_rules! gauge {
    ($name:expr, $unit:expr) => {
        $crate::telemetry::create_gauge($crate::MetricContext {
            name: $name.to_string(),
            description: None,
            unit: $unit,
            attributes: vec![],
        })
    };
    ($name:expr, $description:expr, $unit:expr) => {
        $crate::telemetry::create_gauge($crate::MetricContext {
            name: $name.to_string(),
            description: Some($description.to_string()),
            unit: $unit,
            attributes: vec![],
        })
    };
    ($name:expr, $description:expr, $unit:expr, $($key:expr => $value:expr),+ $(,)?) => {
        $crate::telemetry::create_gauge($crate::MetricContext {
            name: $name.to_string(),
            description: Some($description.to_string()),
            unit: $unit,
            attributes: vec![
                $(($key.to_string(), $value.into())),+
            ],
        })
    };
}


/// Create a histogram metric.
/// 
/// # Examples
/// 
/// ```
/// // Simple histogram with just a name
/// let histogram = histogram!("request_duration");
/// 
/// // Histogram with description and unit
/// let histogram = histogram!("request_duration", "Duration of requests", "milliseconds");
/// 
/// // Histogram with attributes
/// let histogram = histogram!("request_duration",
///    "Duration of requests",
///  "milliseconds",
/// "service" => "api",
/// "status" => "200"
/// );
/// 
/// // Record a value in the histogram
/// histogram.record(42.0);
/// ```
#[macro_export]
macro_rules! histogram {
    ($name:expr, $unit:expr) => {
        $crate::telemetry::create_histogram($crate::MetricContext {
            name: $name.to_string(),
            description: None,
            unit: $unit,
            attributes: vec![],
        })
    };
    ($name:expr, $description:expr, $unit:expr) => {
        $crate::telemetry::create_histogram($crate::MetricContext {
            name: $name.to_string(),
            description: Some($description.to_string()),
            unit: $unit,
            attributes: vec![],
        })
    };
    ($name:expr, $description:expr, $unit:expr, $($key:expr => $value:expr),+ $(,)?) => {
        $crate::telemetry::create_histogram($crate::MetricContext {
            name: $name.to_string(),
            description: Some($description.to_string()),
            unit: $unit,
            attributes: vec![
                $(($key.to_string(), $value.into())),+
            ],
        })
    };
}

/// Log a message with a specific log level.
/// 
/// # Examples
/// 
/// ```
/// // Simple log message with INFO level (default)
/// log!("Processing request");
/// 
/// // Log message with specific level
/// log!(ERROR, "Failed to process request");
/// 
/// // Log message with level and target
/// log!(WARN, "Processing request", target: "app::process_request");
/// 
/// // Log message with level and attributes
/// log!(INFO, "Processing request", "user_id" => "12345", "request_id" => "abc-123");
/// 
/// // Log message with level, target and attributes
/// log!(DEBUG, "Processing request", target: "app::process_request", "user_id" => "12345", "request_id" => "abc-123");
/// ```
macro_rules! log {
    // Without level (default to Info)
    ($message:expr) => {
        $crate::telemetry::log($crate::LogContext {
            level: $crate::LogLevel::Info,
            message: $message.to_string(),
            target: None,
            attributes: vec![],
        })
    };
    ($message:expr, target: $target:expr) => {
        $crate::telemetry::log($crate::LogContext {
            level: $crate::LogLevel::Info,
            message: $message.to_string(),
            target: Some($target.to_string()),
            attributes: vec![],
        })
    };
    ($message:expr, $($key:expr => $value:expr),+ $(,)?) => {
        $crate::telemetry::log($crate::LogContext {
            level: $crate::LogLevel::Info,
            message: $message.to_string(),
            target: None,
            attributes: vec![
                $(($key.to_string(), $value.into())),+
            ],
        })
    };
    ($message:expr, target: $target:expr, $($key:expr => $value:expr),+ $(,)?) => {
        $crate::telemetry::log($crate::LogContext {
            level: $crate::LogLevel::Info,
            message: $message.to_string(),
            target: Some($target.to_string()),
            attributes: vec![
                $(($key.to_string(), $value.into())),+
            ],
        })
    };
}

/// Log a message at DEBUG level.
/// 
/// # Examples
/// 
/// ```
/// // Simple debug message
/// debug!("Processing request details");
/// 
/// // With target
/// debug!("Processing request details", target: "app::process_request");
/// 
/// // With attributes
/// debug!("Processing request details", "user_id" => "12345", "request_id" => "abc-123");
/// ```
#[macro_export]
macro_rules! debug_log {
    ($message:expr) => {
        $crate::telemetry::log($crate::LogContext {
            level: $crate::LogLevel::Debug,
            message: $message.to_string(),
            target: None,
            attributes: vec![],
        })
    };
    ($message:expr, target: $target:expr) => {
        $crate::telemetry::log($crate::LogContext {
            level: $crate::LogLevel::Debug,
            message: $message.to_string(),
            target: Some($target.to_string()),
            attributes: vec![],
        })
    };
    ($message:expr, $($key:expr => $value:expr),+ $(,)?) => {
        $crate::telemetry::log($crate::LogContext {
            level: $crate::LogLevel::Debug,
            message: $message.to_string(),
            target: None,
            attributes: vec![
                $(($key.to_string(), $value.into())),+
            ],
        })
    };
    ($message:expr, target: $target:expr, $($key:expr => $value:expr),+ $(,)?) => {
        $crate::telemetry::log($crate::LogContext {
            level: $crate::LogLevel::Debug,
            message: $message.to_string(),
            target: Some($target.to_string()),
            attributes: vec![
                $(($key.to_string(), $value.into())),+
            ],
        })
    };
}
/// Log a message at INFO level.
/// 
/// # Examples
/// 
/// ```
/// // Simple info message
/// info!("Request processed successfully");
/// 
/// // With target
/// info!("Request processed successfully", target: "app::process_request");
/// 
/// // With attributes
/// info!("Request processed successfully", "user_id" => "12345", "request_id" => "abc-123");
/// ```
#[macro_export]
macro_rules! info_log {
    ($message:expr) => {
        $crate::telemetry::log($crate::LogContext {
            level: $crate::LogLevel::Info,
            message: $message.to_string(),
            target: None,
            attributes: vec![],
        })
    };
    ($message:expr, target: $target:expr) => {
        $crate::telemetry::log($crate::LogContext {
            level: $crate::LogLevel::Info,
            message: $message.to_string(),
            target: Some($target.to_string()),
            attributes: vec![],
        })
    };
    ($message:expr, $($key:expr => $value:expr),+ $(,)?) => {
        $crate::telemetry::log($crate::LogContext {
            level: $crate::LogLevel::Info,
            message: $message.to_string(),
            target: None,
            attributes: vec![
                $(($key.to_string(), $value.into())),+
            ],
        })
    };
    ($message:expr, target: $target:expr, $($key:expr => $value:expr),+ $(,)?) => {
        $crate::telemetry::log($crate::LogContext {
            level: $crate::LogLevel::Info,
            message: $message.to_string(),
            target: Some($target.to_string()),
            attributes: vec![
                $(($key.to_string(), $value.into())),+
            ],
        })
    };
}

/// Log a message at WARN level.
/// 
/// # Examples
/// 
/// ```
/// // Simple warning message
/// warn!("Resource usage high");
/// 
/// // With target
/// warn!("Resource usage high", target: "app::resource_monitor");
/// 
/// // With attributes
/// warn!("Resource usage high", "cpu" => "85%", "memory" => "90%");
/// ```
#[macro_export]
macro_rules! warn_log {
    ($message:expr) => {
        $crate::telemetry::log($crate::LogContext {
            level: $crate::LogLevel::Warn,
            message: $message.to_string(),
            target: None,
            attributes: vec![],
        })
    };
    ($message:expr, target: $target:expr) => {
        $crate::telemetry::log($crate::LogContext {
            level: $crate::LogLevel::Warn,
            message: $message.to_string(),
            target: Some($target.to_string()),
            attributes: vec![],
        })
    };
    ($message:expr, $($key:expr => $value:expr),+ $(,)?) => {
        $crate::telemetry::log($crate::LogContext {
            level: $crate::LogLevel::Warn,
            message: $message.to_string(),
            target: None,
            attributes: vec![
                $(($key.to_string(), $value.into())),+
            ],
        })
    };
    ($message:expr, target: $target:expr, $($key:expr => $value:expr),+ $(,)?) => {
        $crate::telemetry::log($crate::LogContext {
            level: $crate::LogLevel::Warn,
            message: $message.to_string(),
            target: Some($target.to_string()),
            attributes: vec![
                $(($key.to_string(), $value.into())),+
            ],
        })
    };
}


/// Log a message at ERROR level.
/// 
/// # Examples
/// 
/// ```
/// // Simple error message
/// error!("Failed to process request");
/// 
/// // With target
/// error!("Failed to process request", target: "app::process_request");
/// 
/// // With attributes
/// error!("Failed to process request", "user_id" => "12345", "error_code" => "500");
/// ```
#[macro_export]
macro_rules! error_log {
    ($message:expr) => {
        $crate::telemetry::log($crate::LogContext {
            level: $crate::LogLevel::Error,
            message: $message.to_string(),
            target: None,
            attributes: vec![],
        })
    };
    ($message:expr, target: $target:expr) => {
        $crate::telemetry::log($crate::LogContext {
            level: $crate::LogLevel::Error,
            message: $message.to_string(),
            target: Some($target.to_string()),
            attributes: vec![],
        })
    };
    ($message:expr, $($key:expr => $value:expr),+ $(,)?) => {
        $crate::telemetry::log($crate::LogContext {
            level: $crate::LogLevel::Error,
            message: $message.to_string(),
            target: None,
            attributes: vec![
                $(($key.to_string(), $value.into())),+
            ],
        })
    };
    ($message:expr, target: $target:expr, $($key:expr => $value:expr),+ $(,)?) => {
        $crate::telemetry::log($crate::LogContext {
            level: $crate::LogLevel::Error,
            message: $message.to_string(),
            target: Some($target.to_string()),
            attributes: vec![
                $(($key.to_string(), $value.into())),+
            ],
        })
    };
}