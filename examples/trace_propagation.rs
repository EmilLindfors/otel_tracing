// Making metrics shareable across tasks using Arc

use opentelemetry::context::FutureExt;
use opentelemetry::Context;
use otel_tracing::{
    counter, debug_log, error_log,
    facade::MetricUnit,
    gauge, histogram, info_log, span,
    telemetry::{init_datadog, shutdown},
    warn_log, with_async_span, with_span,
};
use std::{
    sync::Arc,
    time::{Duration, Instant},
};

/// Helper function to spawn a task that preserves trace context
pub fn spawn_with_context<F, R>(future: F) -> tokio::task::JoinHandle<R>
where
    F: std::future::Future<Output = R> + Send + 'static,
    R: Send + 'static,
{
    // Capture the current context before spawning
    let parent_context = Context::current();

    // Use opentelemetry_futures to propagate context without a guard
    tokio::spawn(future.with_context(parent_context))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    // Initialize telemetry
    init_datadog().await?;

    // Create metrics that we'll use throughout the application
    // Wrap them in Arc to make them shareable across tasks
    println!("Creating test metrics...");

    // Create metrics with Arc for sharing
    let test_counter = Arc::new(counter!("rust_test_service.request_count",
        "Test request counter",
        "count",
        "type" => "http"
    ));

    let test_gauge = Arc::new(gauge!("rust_test_service.active_users",
        "Test active users gauge",
        "count",
        "type" => "users"
    ));

    let test_histogram = Arc::new(histogram!(
        "rust_test_service.response_time",
        "Test response time histogram",
        "ms"
    ));

    // Main process with logs and metrics
    let result = with_async_span!("main_process", async {
        let start_time = Instant::now();

        // Clone Arc for use in this async block
        let request_counter = test_counter.clone();
        let processing_gauge = test_gauge.clone();
        let duration_histogram = test_histogram.clone();

        // Increment the counter
        request_counter.add(
            1,
            vec![
                ("operation".to_string(), "main_process".into()),
                ("priority".to_string(), "high".into()),
            ],
        );

        // First operation - prepare data
        let data = with_async_span!("prepare_data", async {
            // Clone Arc for use in this async block
            let processing_gauge = processing_gauge.clone();
            let duration_histogram = duration_histogram.clone();

            let prepare_start = Instant::now();

            // Record metric - set gauge to indicate active task
            processing_gauge.set(1.0, vec![("step".to_string(), "prepare_data".into())]);

            // Simulate work
            tokio::time::sleep(Duration::from_secs(1)).await;

            // Record duration in histogram
            let duration_ms = prepare_start.elapsed().as_millis() as f64;
            duration_histogram.record(
                duration_ms,
                vec![("step".to_string(), "prepare_data".into())],
            );

            // Update gauge - no longer active
            processing_gauge.set(0.0, vec![("step".to_string(), "prepare_data".into())]);

            vec![1, 2, 3, 4, 5]
        })
        .await;

        // Spawn multiple tasks with proper context propagation
        let mut handles = Vec::new();

        // Update gauge to reflect total tasks
        processing_gauge.set(3.0, vec![("step".to_string(), "background_tasks".into())]);

        for i in 0..3 {
            let task_data = data.clone();

            // Clone the Arc references for use in the spawned task
            let request_counter = request_counter.clone();
            let duration_histogram = duration_histogram.clone();

            // Increment counter for each task
            request_counter.add(1, vec![("operation".to_string(), "background_task".into())]);

            // Use our helper to spawn a task with context
            let handle = spawn_with_context(async move {
                // This will be properly connected to the parent trace
                with_async_span!(
                    "background_task",
                    [("task_id", i), ("data_size", task_data.len())],
                    async {
                        let task_start = Instant::now();

                        // Simulate processing
                        tokio::time::sleep(Duration::from_secs(2)).await;

                        // Process data in nested span
                        let result = with_async_span!("process_data", async {
                            // Clone the Arc for this inner span
                            let duration_histogram = duration_histogram.clone();

                            let process_start = Instant::now();
                            tokio::time::sleep(Duration::from_secs(1)).await;

                            let sum = task_data.iter().sum::<i32>();

                            // Record duration in histogram
                            let duration_ms = process_start.elapsed().as_millis() as f64;
                            duration_histogram.record(
                                duration_ms,
                                vec![
                                    ("step".to_string(), "process_data".into()),
                                    ("task_id".to_string(), i.to_string().into()),
                                ],
                            );

                            sum
                        })
                        .await;

                        // Record overall task duration
                        let task_duration_ms = task_start.elapsed().as_millis() as f64;
                        duration_histogram.record(
                            task_duration_ms,
                            vec![
                                ("step".to_string(), "background_task".into()),
                                ("task_id".to_string(), i.to_string().into()),
                            ],
                        );

                        result
                    }
                )
                .await
            });

            handles.push(handle);
        }

        // Wait for all tasks and collect results
        let mut results = Vec::new();
        for (i, handle) in handles.into_iter().enumerate() {
            info_log!("Waiting for task completion", "task_id" => i);
            let task_result = handle.await.unwrap();
            results.push(task_result);
        }

        // Update gauge - all tasks complete
        processing_gauge.set(0.0, vec![("step".to_string(), "background_tasks".into())]);

        // Final aggregation with metrics
        let final_result = with_span!("aggregate_results", {
            let agg_start = Instant::now();

            // Record metric for this operation
            processing_gauge.set(1.0, vec![("step".to_string(), "aggregate_results".into())]);

            let sum = results.iter().sum::<i32>();

            // Record completion
            let duration_ms = agg_start.elapsed().as_millis() as f64;
            duration_histogram.record(
                duration_ms,
                vec![("step".to_string(), "aggregate_results".into())],
            );
            processing_gauge.set(0.0, vec![("step".to_string(), "aggregate_results".into())]);

            sum
        });

        // End-to-end duration metric
        let total_duration_ms = start_time.elapsed().as_millis() as f64;
        duration_histogram.record(
            total_duration_ms,
            vec![("step".to_string(), "total".into())],
        );

        final_result
    })
    .await;

    println!("Final result: {}", result);

    // Shutdown telemetry
    info_log!("Shutting down telemetry");
    shutdown().await?;

    // Allow time for traces to be sent
    tokio::time::sleep(Duration::from_secs(2)).await;

    Ok(())
}
