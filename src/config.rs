/// Configuration for the telemetry services
#[derive(Debug, Clone)]
pub struct TelemetryConfig {
    /// Service name to be reported to Datadog
    pub service_name: String,
    /// Service version
    pub version: String,
    /// Environment (e.g., "production", "staging")
    pub environment: String,
    /// Datadog agent endpoint for traces (default: http://localhost:8126)
    pub dd_trace_agent_url: String,
    /// OTLP collector endpoint for logs (default: http://localhost:4318)
    pub otlp_logs_endpoint: String,
    /// Enable tracing integration (default: false)
    pub enable_tracing: bool,
}

impl TelemetryConfig {
    /// Create a new telemetry configuration with the given parameters
    pub fn new(
        service_name: impl Into<String>,
        version: impl Into<String>,
        environment: impl Into<String>,
    ) -> Self {
        Self {
            service_name: service_name.into(),
            version: version.into(),
            environment: environment.into(),
            dd_trace_agent_url: "http://localhost:8126".to_string(),
            otlp_logs_endpoint: "http://localhost:4318/v1/logs".to_string(),
            enable_tracing: false,
        }
    }

    /// Set the Datadog trace agent URL
    pub fn with_dd_trace_agent_url(mut self, url: impl Into<String>) -> Self {
        self.dd_trace_agent_url = url.into();
        self
    }

    /// Set the OTLP logs endpoint
    pub fn with_otlp_logs_endpoint(mut self, url: impl Into<String>) -> Self {
        self.otlp_logs_endpoint = url.into();
        self
    }
    
    /// Enable tracing integration
    pub fn with_tracing(mut self, enable: bool) -> Self {
        self.enable_tracing = enable;
        self
    }
    
    /// Create a configuration from environment variables
    pub fn from_env() -> Self {
        let service_name = std::env::var("DD_SERVICE").unwrap_or_else(|_| "unknown-service".to_string());
        let version = std::env::var("DD_VERSION").unwrap_or_else(|_| "0.1.0".to_string());
        let environment = std::env::var("DD_ENV").unwrap_or_else(|_| "development".to_string());
        
        let dd_host = std::env::var("DD_AGENT_HOST").unwrap_or_else(|_| "localhost".to_string());
        let dd_port = std::env::var("DD_AGENT_PORT")
            .ok()
            .and_then(|it| it.parse::<i32>().ok())
            .unwrap_or(8126);
        let dd_trace_agent_url = format!("http://{}:{}", dd_host, dd_port);
        
        let otlp_host = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
            .unwrap_or_else(|_| "http://localhost:4318".to_string());
        let otlp_logs_endpoint = format!("{}/v1/logs", otlp_host);
        
        let enable_tracing = std::env::var("DD_ENABLED")
            .map(|s| s == "true")
            .unwrap_or(false);
            
        Self {
            service_name,
            version,
            environment,
            dd_trace_agent_url,
            otlp_logs_endpoint,
            enable_tracing,
        }
    }
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            service_name: "unknown-service".to_string(),
            version: "0.1.0".to_string(),
            environment: "development".to_string(),
            dd_trace_agent_url: "http://localhost:8126".to_string(),
            otlp_logs_endpoint: "http://localhost:4318/v1/logs".to_string(),
            enable_tracing: false,
        }
    }
}
