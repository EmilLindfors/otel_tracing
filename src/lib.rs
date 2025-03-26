mod domain;
mod ports;
mod adapters;
mod services;
pub mod facade;

pub use facade as telemetry;
pub use domain::telemetry::{SpanContext, MetricContext, LogContext, AttributeValue, TelemetryError};

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
    ($name:expr, $code:expr) => {
        $crate::telemetry::with_async_span($name, vec![], $code)
    };
    ($name:expr, $($key:expr => $value:expr),+, $code:expr) => {
        $crate::telemetry::with_async_span(
            $name,
            vec![$(($key.to_string(), $value.into())),+],
            $code
        )
    };
}

#[macro_export]
macro_rules! counter {
    ($name:expr) => {
        $crate::telemetry::create_counter($crate::MetricContext {
            name: $name.to_string(),
            description: None,
            unit: None,
            attributes: vec![],
        })
    };
    ($name:expr, $description:expr, $unit:expr) => {
        $crate::telemetry::create_counter($crate::MetricContext {
            name: $name.to_string(),
            description: Some($description.to_string()),
            unit: Some($unit.to_string()),
            attributes: vec![],
        })
    };
    ($name:expr, $description:expr, $unit:expr, $($key:expr => $value:expr),+ $(,)?) => {
        $crate::telemetry::create_counter($crate::MetricContext {
            name: $name.to_string(),
            description: Some($description.to_string()),
            unit: Some($unit.to_string()),
            attributes: vec![
                $(($key.to_string(), $value.into())),+
            ],
        })
    };
}

#[macro_export]
macro_rules! gauge {
    ($name:expr) => {
        $crate::telemetry::create_gauge($crate::MetricContext {
            name: $name.to_string(),
            description: None,
            unit: None,
            attributes: vec![],
        })
    };
    ($name:expr, $description:expr, $unit:expr) => {
        $crate::telemetry::create_gauge($crate::MetricContext {
            name: $name.to_string(),
            description: Some($description.to_string()),
            unit: Some($unit.to_string()),
            attributes: vec![],
        })
    };
    ($name:expr, $description:expr, $unit:expr, $($key:expr => $value:expr),+ $(,)?) => {
        $crate::telemetry::create_gauge($crate::MetricContext {
            name: $name.to_string(),
            description: Some($description.to_string()),
            unit: Some($unit.to_string()),
            attributes: vec![
                $(($key.to_string(), $value.into())),+
            ],
        })
    };
}

#[macro_export]
macro_rules! histogram {
    ($name:expr) => {
        $crate::telemetry::create_histogram($crate::MetricContext {
            name: $name.to_string(),
            description: None,
            unit: None,
            attributes: vec![],
        })
    };
    ($name:expr, $description:expr, $unit:expr) => {
        $crate::telemetry::create_histogram($crate::MetricContext {
            name: $name.to_string(),
            description: Some($description.to_string()),
            unit: Some($unit.to_string()),
            attributes: vec![],
        })
    };
    ($name:expr, $description:expr, $unit:expr, $($key:expr => $value:expr),+ $(,)?) => {
        $crate::telemetry::create_histogram($crate::MetricContext {
            name: $name.to_string(),
            description: Some($description.to_string()),
            unit: Some($unit.to_string()),
            attributes: vec![
                $(($key.to_string(), $value.into())),+
            ],
        })
    };
}

#[macro_export]
macro_rules! log {
    ($message:expr) => {
        $crate::telemetry::log($crate::LogContext {
            message: $message.to_string(),
            target: None,
            attributes: vec![],
        })
    };
    ($message:expr, target: $target:expr) => {
        $crate::telemetry::log($crate::LogContext {
            message: $message.to_string(),
            target: Some($target.to_string()),
            attributes: vec![],
        })
    };
    ($message:expr, $($key:expr => $value:expr),+ $(,)?) => {
        $crate::telemetry::log($crate::LogContext {
            message: $message.to_string(),
            target: None,
            attributes: vec![
                $(($key.to_string(), $value.into())),+
            ],
        })
    };
    ($message:expr, target: $target:expr, $($key:expr => $value:expr),+ $(,)?) => {
        $crate::telemetry::log($crate::LogContext {
            message: $message.to_string(),
            target: Some($target.to_string()),
            attributes: vec![
                $(($key.to_string(), $value.into())),+
            ],
        })
    };
}
