//! Metrics collection module
//!
//! Tracks engine performance, error rates, and usage statistics.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::RwLock;

/// Global metrics collector
pub struct Metrics {
    /// Total search count
    pub total_searches: AtomicU64,
    /// Searches per engine
    engine_searches: RwLock<HashMap<String, u64>>,
    /// Engine response times (rolling average in ms)
    engine_response_times: RwLock<HashMap<String, Vec<u64>>>,
    /// Engine error counts
    engine_errors: RwLock<HashMap<String, u64>>,
    /// Engine success counts
    engine_successes: RwLock<HashMap<String, u64>>,
}

impl Metrics {
    /// Create a new metrics instance
    pub fn new() -> Self {
        Self {
            total_searches: AtomicU64::new(0),
            engine_searches: RwLock::new(HashMap::new()),
            engine_response_times: RwLock::new(HashMap::new()),
            engine_errors: RwLock::new(HashMap::new()),
            engine_successes: RwLock::new(HashMap::new()),
        }
    }

    /// Increment total search count
    pub fn inc_search(&self) {
        self.total_searches.fetch_add(1, Ordering::Relaxed);
    }

    /// Record an engine search
    pub fn record_engine_search(&self, engine: &str) {
        let mut searches = self.engine_searches.write().unwrap();
        *searches.entry(engine.to_string()).or_insert(0) += 1;
    }

    /// Record engine response time
    pub fn record_response_time(&self, engine: &str, time_ms: u64) {
        let mut times = self.engine_response_times.write().unwrap();
        let entry = times.entry(engine.to_string()).or_insert_with(Vec::new);

        // Keep last 100 response times
        if entry.len() >= 100 {
            entry.remove(0);
        }
        entry.push(time_ms);
    }

    /// Record engine error
    pub fn record_error(&self, engine: &str) {
        let mut errors = self.engine_errors.write().unwrap();
        *errors.entry(engine.to_string()).or_insert(0) += 1;
    }

    /// Record engine success
    pub fn record_success(&self, engine: &str) {
        let mut successes = self.engine_successes.write().unwrap();
        *successes.entry(engine.to_string()).or_insert(0) += 1;
    }

    /// Get total searches
    pub fn get_total_searches(&self) -> u64 {
        self.total_searches.load(Ordering::Relaxed)
    }

    /// Get average response time for an engine
    pub fn get_avg_response_time(&self, engine: &str) -> Option<u64> {
        let times = self.engine_response_times.read().unwrap();
        times.get(engine).and_then(|t| {
            if t.is_empty() {
                None
            } else {
                Some(t.iter().sum::<u64>() / t.len() as u64)
            }
        })
    }

    /// Get reliability percentage for an engine
    pub fn get_reliability(&self, engine: &str) -> f64 {
        let errors = self.engine_errors.read().unwrap();
        let successes = self.engine_successes.read().unwrap();

        let error_count = *errors.get(engine).unwrap_or(&0);
        let success_count = *successes.get(engine).unwrap_or(&0);

        let total = error_count + success_count;
        if total == 0 {
            100.0
        } else {
            (success_count as f64 / total as f64) * 100.0
        }
    }

    /// Get all engine statistics
    pub fn get_engine_stats(&self) -> HashMap<String, EngineStats> {
        let searches = self.engine_searches.read().unwrap();
        let mut stats = HashMap::new();

        for engine in searches.keys() {
            stats.insert(
                engine.clone(),
                EngineStats {
                    searches: *searches.get(engine).unwrap_or(&0),
                    avg_response_time: self.get_avg_response_time(engine),
                    reliability: self.get_reliability(engine),
                },
            );
        }

        stats
    }
}

impl Default for Metrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics for a single engine
#[derive(Debug, Clone)]
pub struct EngineStats {
    pub searches: u64,
    pub avg_response_time: Option<u64>,
    pub reliability: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics() {
        let metrics = Metrics::new();

        metrics.inc_search();
        metrics.record_engine_search("google");
        metrics.record_response_time("google", 100);
        metrics.record_success("google");

        assert_eq!(metrics.get_total_searches(), 1);
        assert_eq!(metrics.get_avg_response_time("google"), Some(100));
        assert_eq!(metrics.get_reliability("google"), 100.0);
    }
}
