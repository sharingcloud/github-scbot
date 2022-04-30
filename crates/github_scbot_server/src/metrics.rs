use actix_web_prom::{PrometheusMetrics, PrometheusMetricsBuilder};
use lazy_static::lazy_static;
use prometheus::IntCounter;

lazy_static! {
    pub static ref GITHUB_API_CALLS: IntCounter =
        IntCounter::new("github_api_calls", "GitHub API calls").unwrap();
    pub static ref TENOR_API_CALLS: IntCounter =
        IntCounter::new("tenor_api_calls", "Tenor API calls").unwrap();
    pub static ref REDIS_CALLS: IntCounter = IntCounter::new("redis_calls", "Redis calls").unwrap();
}

pub(crate) fn build_metrics_handler() -> PrometheusMetrics {
    let prometheus = PrometheusMetricsBuilder::new("api")
        .endpoint("/metrics")
        .build()
        .unwrap();

    setup_process_metrics(&prometheus);

    prometheus
        .registry
        .register(Box::new(GITHUB_API_CALLS.clone()))
        .unwrap();
    prometheus
        .registry
        .register(Box::new(TENOR_API_CALLS.clone()))
        .unwrap();
    prometheus
        .registry
        .register(Box::new(REDIS_CALLS.clone()))
        .unwrap();
    prometheus
}

#[cfg(unix)]
fn setup_process_metrics(metrics: &PrometheusMetrics) {
    use prometheus::process_collector::ProcessCollector;

    prometheus
        .registry
        .register(Box::new(ProcessCollector::for_self()))
        .unwrap();
}

#[cfg(windows)]
fn setup_process_metrics(_metrics: &PrometheusMetrics) {
    println!("Process metrics are not supported on Windows.");
}
