// File: lib.rs - This file is part of AURIA
// Copyright (c) 2026 AURIA Developers and Contributors
// Description:
//     Logging, metrics, and tracing for AURIA Runtime Core.
//     Provides observability primitives including structured logging,
//     metrics collection, and distributed tracing support.
//
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct MetricsCollector {
    counters: Arc<RwLock<HashMap<String, Counter>>>,
    gauges: Arc<RwLock<HashMap<String, f64>>>,
    histograms: Arc<RwLock<HashMap<String, Histogram>>>,
}

#[derive(Clone)]
pub struct Counter {
    pub name: String,
    pub value: u64,
    pub labels: HashMap<String, String>,
}

#[derive(Clone)]
pub struct Histogram {
    pub name: String,
    pub values: Vec<f64>,
    pub count: u64,
    pub sum: f64,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            counters: Arc::new(RwLock::new(HashMap::new())),
            gauges: Arc::new(RwLock::new(HashMap::new())),
            histograms: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn increment_counter(&self, name: &str, value: u64, labels: HashMap<String, String>) {
        let mut counters = self.counters.write().await;
        if let Some(counter) = counters.get_mut(name) {
            counter.value += value;
        } else {
            counters.insert(name.to_string(), Counter {
                name: name.to_string(),
                value,
                labels,
            });
        }
    }

    pub async fn set_gauge(&self, name: &str, value: f64) {
        let mut gauges = self.gauges.write().await;
        gauges.insert(name.to_string(), value);
    }

    pub async fn record_histogram(&self, name: &str, value: f64) {
        let mut histograms = self.histograms.write().await;
        if let Some(hist) = histograms.get_mut(name) {
            hist.values.push(value);
            hist.count += 1;
            hist.sum += value;
        } else {
            histograms.insert(name.to_string(), Histogram {
                name: name.to_string(),
                values: vec![value],
                count: 1,
                sum: value,
            });
        }
    }

    pub async fn get_counter(&self, name: &str) -> Option<u64> {
        let counters = self.counters.read().await;
        counters.get(name).map(|c| c.value)
    }

    pub async fn get_gauge(&self, name: &str) -> Option<f64> {
        let gauges = self.gauges.read().await;
        gauges.get(name).copied()
    }

    pub async fn get_all_metrics(&self) -> String {
        let mut output = String::new();
        
        let counters = self.counters.read().await;
        for (_, counter) in counters.iter() {
            output.push_str(&format!("# HELP {} counter\n", counter.name));
            output.push_str(&format!("# TYPE {} counter\n", counter.name));
            output.push_str(&format!("{} {{}} {}\n\n", counter.name, counter.value));
        }

        let gauges = self.gauges.read().await;
        for (name, value) in gauges.iter() {
            output.push_str(&format!("# HELP {} gauge\n", name));
            output.push_str(&format!("# TYPE {} gauge\n", name));
            output.push_str(&format!("{} {}\n\n", name, value));
        }

        let histograms = self.histograms.read().await;
        for (name, hist) in histograms.iter() {
            output.push_str(&format!("# HELP {} histogram\n", name));
            output.push_str(&format!("# TYPE {} histogram\n", name));
            output.push_str(&format!("{}_count {}\n", name, hist.count));
            output.push_str(&format!("{}sum {}\n\n", name, hist.sum));
        }

        output
    }

    pub async fn reset(&self) {
        self.counters.write().await.clear();
        self.gauges.write().await.clear();
        self.histograms.write().await.clear();
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

pub struct TracingConfig {
    pub service_name: String,
    pub log_level: String,
    pub enable_jaeger: bool,
    pub jaeger_endpoint: Option<String>,
}

impl Default for TracingConfig {
    fn default() -> Self {
        Self {
            service_name: "auria".to_string(),
            log_level: "info".to_string(),
            enable_jaeger: false,
            jaeger_endpoint: None,
        }
    }
}

pub fn init_tracing(config: TracingConfig) {
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(config.log_level.parse::<tracing::Level>().unwrap_or(tracing::Level::INFO))
        .with_target(true)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("Failed to set tracing subscriber");
}

pub struct Telemetry {
    metrics: Arc<MetricsCollector>,
    start_time: u64,
}

impl Telemetry {
    pub fn new() -> Self {
        let start_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        Self {
            metrics: Arc::new(MetricsCollector::new()),
            start_time,
        }
    }

    pub fn metrics(&self) -> Arc<MetricsCollector> {
        self.metrics.clone()
    }

    pub fn uptime_seconds(&self) -> u64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        now - self.start_time
    }

    pub async fn record_request(&self, tier: &str, latency_ms: f64, tokens: u32) {
        let mut labels = HashMap::new();
        labels.insert("tier".to_string(), tier.to_string());
        
        self.metrics.increment_counter("auria_requests_total", 1, labels.clone()).await;
        
        let latency_name = format!("{}_latency_ms", tier);
        self.metrics.record_histogram(&latency_name, latency_ms).await;
        
        let tokens_name = format!("{}_tokens_total", tier);
        self.metrics.increment_counter(&tokens_name, tokens as u64, labels.clone()).await;
    }

    pub async fn record_error(&self, error_type: &str) {
        let mut labels = HashMap::new();
        labels.insert("type".to_string(), error_type.to_string());
        self.metrics.increment_counter("auria_errors_total", 1, labels).await;
    }
}

impl Default for Telemetry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_metrics_collector() {
        let collector = MetricsCollector::new();
        
        collector.increment_counter("test_counter", 1, HashMap::new()).await;
        let value = collector.get_counter("test_counter").await;
        
        assert_eq!(value, Some(1));
    }
}
