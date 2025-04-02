use otel_datadog::{
    TelemetryConfig, init_telemetry, shutdown_telemetry, get_logger, get_tracer, TelemetryHandle
};
use std::error::Error;
use std::thread;
use std::time::Duration;
use tracing::{info, info_span, warn, Instrument};

// Example using the tracing crate
fn tracing_database_query() {
    // Create a span for the database query
    let span = info_span!("database_query", span.type = "sql");
    
    // Execute the span
    async {
        // Log within the span
        info!(sql.query = "SELECT * FROM users WHERE id = 1", "Executing database query");
        
        // Simulate work
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    .instrument(span)
    .now_or_never();
}

// Example using direct API
fn database_query() {
    // Get the global tracer
    let tracer = get_tracer();
    let logger = get_logger();
    
    // Create a span for the database query
    let mut span = tracer.start_span("database_query");
    
    // Set Datadog-specific attributes
    span.set_attribute("span.type", "sql");
    span.set_attribute("sql.query", "SELECT * FROM users WHERE id = 1");
    
    // Log within the span
    logger.info("Executing database query").unwrap();
    
    // Simulate work
    thread::sleep(Duration::from_millis(50));
    
    // End the span
    span.end();
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    // Configure telemetry
    let config = TelemetryConfig::new(
        "example-service",
        "0.1.0",
        "development",
    )
    .with_tracing(true); // Enable tracing integration
    
    // Alternative: Load config from environment variables
    // let config = TelemetryConfig::from_env();
    
    // Initialize telemetry
    let _handle: TelemetryHandle = init_telemetry(&config)?;
    
    // Method 1: Using the tracing crate
    info!("Starting application with tracing");
    
    // Create a trace for an HTTP request using tracing macros
    {
        let span = info_span!(
            "http_request", 
            span.type = "web",
            http.url = "http://localhost:8080/api/users",
            http.method = "GET",
            http.status_code = 200
        );
        
        async {
            // Log within the span
            info!("Processing HTTP request with tracing");
            
            // Simulate some work
            tokio::time::sleep(Duration::from_millis(100)).await;
            
            // Call the database query function (which will create a child span)
            tracing_database_query();
            
            // More processing
            tokio::time::sleep(Duration::from_millis(50)).await;
            
            // Span ends automatically when dropped
        }
        .instrument(span)
        .await;
    }
    
    // Method 2: Using the direct API
    // Get logger and tracer
    let logger = get_logger();
    let tracer = get_tracer();
    
    // Log application start
    logger.info("Starting application with direct API")?;
    
    // Create a trace for an HTTP request
    tracer.with_span("http_request_direct", || {
        // Get the current span and set attributes
        let mut span = tracer.start_span("http_request_direct");
        span.set_attribute("span.type", "web");
        span.set_attribute("http.url", "http://localhost:8080/api/products");
        span.set_attribute("http.method", "POST");
        span.set_attribute("http.status_code", 201);
        
        // Log within the span
        logger.info("Processing HTTP request with direct API").unwrap();
        
        // Simulate some work
        thread::sleep(Duration::from_millis(100));
        
        // Call the database query function (which will create a child span)
        database_query();
        
        // More processing
        thread::sleep(Duration::from_millis(50));
        
        // Warn about something
        warn!("This is a warning from the tracing API within a direct API span");
        logger.warn("This is a warning from the direct API").unwrap();
        
        // End the span
        span.end();
    });
    
    // Log application completion
    info!("Application completed (tracing)");
    logger.info("Application completed (direct API)")?;
    
    // Shutdown telemetry - now requires await
    shutdown_telemetry().await?;
    
    Ok(())
}