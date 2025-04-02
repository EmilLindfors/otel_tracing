use otel_datadog::{
    TelemetryConfig, init_telemetry, shutdown_telemetry, get_logger, get_tracer, TelemetryHandle
};
use std::error::Error;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{error, info, info_span, Instrument};

/// Function that performs some work and generates logs within a trace
async fn perform_work(iteration: usize) {
    // Create a parent span for this operation
    let work_span = info_span!(
        "perform_work",
        iteration = iteration,
        span.type = "custom", 
        operation.name = "work_batch"
    );

    // Instrument the async block with the span
    async {
        // Log within the span - this will have trace correlation
        info!(
            status = "started",
            "Starting work iteration {}", 
            iteration
        );
        
        // Simulate some processing time
        sleep(Duration::from_millis(50)).await;
        
        // Create a child span for a nested operation
        let db_span = info_span!(
            "database_query",
            span.type = "sql",
            sql.query = "SELECT * FROM work_items WHERE batch_id = ?",
            sql.params = format!("batch_{}", iteration)
        );

        // Instrument a nested async block
        async {
            info!("Executing database query for batch {}", iteration);
            
            // Simulate database query
            sleep(Duration::from_millis(20)).await;
            
            // Sometimes generate an error
            if iteration % 3 == 0 {
                error!(
                    error.type = "timeout",
                    error.message = "Database query timed out",
                    "Query timeout occurred for batch {}", 
                    iteration
                );
            } else {
                info!(
                    rows_affected = iteration * 10,
                    "Query completed successfully"
                );
            }
        }
        .instrument(db_span)
        .await;

        // Do some more work after the database query
        sleep(Duration::from_millis(30)).await;
        
        info!(
            duration_ms = 100,
            status = "completed",
            "Completed work iteration {}", 
            iteration
        );
    }
    .instrument(work_span)
    .await;
}

/// Function that demonstrates using the direct logger API
async fn direct_api_example() {
    let logger = get_logger();
    let tracer = get_tracer();
    
    logger.info("Starting direct API example").unwrap();
    
    // Create a span
    tracer.with_span("direct_api", || {
        let mut span = tracer.start_span("direct_api");
        span.set_attribute("span.type", "custom");
        span.set_attribute("operation.name", "direct_api_operation");
        
        // Log within the span - logs will be correlated
        logger.info("Processing with direct API").unwrap();
        
        // Simulate work
        std::thread::sleep(Duration::from_millis(100));
        
        // Example of structured logging with the direct API
        let mut db_span = tracer.start_span("db_operation");
        db_span.set_attribute("span.type", "sql");
        db_span.set_attribute("sql.query", "INSERT INTO logs VALUES (?, ?)");
        
        logger.info("Executing database write operation").unwrap();
        std::thread::sleep(Duration::from_millis(50));
        
        logger.info("Database operation completed").unwrap();
        db_span.end();
        
        logger.info("Direct API example completed").unwrap();
        span.end();
    });
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    dotenvy::dotenv().ok(); // Load environment variables from .env file
    println!("Starting async logging example");
    
    // Configure telemetry with async batch processing enabled
    let config = TelemetryConfig::new(
        "async-logging-example",
        "0.1.0",
        "development",
    )
    .with_dd_trace_agent_url("http://localhost:8126")
    .with_otlp_logs_endpoint("http://localhost:4318/v1/logs")
    .with_tracing(true);
    
    // Initialize telemetry
    let _handle: TelemetryHandle = init_telemetry(&config)?;
    println!("Telemetry initialized with async batch processing");

    // Generate a series of operations with logs
    for i in 1..=5 {
        perform_work(i).await;
    }
    
    // Demonstrate using the direct API as well
    direct_api_example().await;
    
    // Allow time for batch processor to flush logs
    println!("Waiting for batch processor to flush logs...");
    sleep(Duration::from_secs(1)).await;
    
    // Proper shutdown of telemetry (will flush any pending logs)
    println!("Shutting down telemetry...");
    shutdown_telemetry().await?;
    println!("Telemetry shutdown completed");
    
    Ok(())
}