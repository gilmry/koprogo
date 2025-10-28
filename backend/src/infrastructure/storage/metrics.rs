use once_cell::sync::Lazy;
use prometheus::{register_histogram_vec, register_int_counter_vec, HistogramVec, IntCounterVec};
use std::time::Duration;

static STORAGE_OPERATION_TOTAL: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "storage_operation_total",
        "Total number of storage operations",
        &["provider", "operation", "result"]
    )
    .expect("failed to register storage_operation_total counter")
});

static STORAGE_OPERATION_DURATION_SECONDS: Lazy<HistogramVec> = Lazy::new(|| {
    register_histogram_vec!(
        "storage_operation_duration_seconds",
        "Storage operation duration in seconds",
        &["provider", "operation"]
    )
    .expect("failed to register storage_operation_duration_seconds histogram")
});

pub fn record_storage_operation(
    provider: &'static str,
    operation: &'static str,
    duration: Duration,
    result: Result<(), &str>,
) {
    let status = match result {
        Ok(_) => "success",
        Err(_) => "error",
    };

    STORAGE_OPERATION_TOTAL
        .with_label_values(&[provider, operation, status])
        .inc();

    STORAGE_OPERATION_DURATION_SECONDS
        .with_label_values(&[provider, operation])
        .observe(duration.as_secs_f64());
}
