//! Result container for aggregating and deduplicating search results

use super::types::*;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};

/// Container for aggregating search results from multiple engines
#[derive(Debug, Clone)]
pub struct ResultContainer {
    /// Main results map (URL hash -> Result) for deduplication
    results_map: Arc<RwLock<HashMap<String, Result>>>,
    /// Direct answers
    answers: Arc<RwLock<Vec<Answer>>>,
    /// Search suggestions
    suggestions: Arc<RwLock<HashSet<Suggestion>>>,
    /// Spelling corrections
    corrections: Arc<RwLock<HashSet<Correction>>>,
    /// Information boxes
    infoboxes: Arc<RwLock<Vec<InfoBox>>>,
    /// Unresponsive engines
    unresponsive_engines: Arc<RwLock<Vec<UnresponsiveEngine>>>,
    /// Engine timings
    timings: Arc<RwLock<Vec<Timing>>>,
    /// Redirect URL (for external bangs)
    redirect_url: Arc<RwLock<Option<String>>>,
    /// Engine weights for scoring
    engine_weights: HashMap<String, f64>,
}

impl Default for ResultContainer {
    fn default() -> Self {
        Self::new()
    }
}

impl ResultContainer {
    /// Create a new empty result container
    pub fn new() -> Self {
        Self {
            results_map: Arc::new(RwLock::new(HashMap::new())),
            answers: Arc::new(RwLock::new(Vec::new())),
            suggestions: Arc::new(RwLock::new(HashSet::new())),
            corrections: Arc::new(RwLock::new(HashSet::new())),
            infoboxes: Arc::new(RwLock::new(Vec::new())),
            unresponsive_engines: Arc::new(RwLock::new(Vec::new())),
            timings: Arc::new(RwLock::new(Vec::new())),
            redirect_url: Arc::new(RwLock::new(None)),
            engine_weights: HashMap::new(),
        }
    }

    /// Create with engine weights
    pub fn with_weights(weights: HashMap<String, f64>) -> Self {
        let mut container = Self::new();
        container.engine_weights = weights;
        container
    }

    /// Add a result, merging with existing if URL matches
    pub fn add_result(&self, result: Result) {
        let url_hash = Self::url_hash(&result.url);

        let mut map = self.results_map.write().unwrap();
        if let Some(existing) = map.get_mut(&url_hash) {
            existing.merge(&result);
        } else {
            map.insert(url_hash, result);
        }
    }

    /// Add multiple results
    pub fn extend_results(&self, results: Vec<Result>) {
        for result in results {
            self.add_result(result);
        }
    }

    /// Add an answer
    pub fn add_answer(&self, answer: Answer) {
        let mut answers = self.answers.write().unwrap();
        // Avoid duplicates
        if !answers.iter().any(|a| a.answer == answer.answer) {
            answers.push(answer);
        }
    }

    /// Add a suggestion
    pub fn add_suggestion(&self, suggestion: Suggestion) {
        self.suggestions.write().unwrap().insert(suggestion);
    }

    /// Add a correction
    pub fn add_correction(&self, correction: Correction) {
        self.corrections.write().unwrap().insert(correction);
    }

    /// Add an infobox
    pub fn add_infobox(&self, infobox: InfoBox) {
        let mut boxes = self.infoboxes.write().unwrap();
        // Merge if same ID exists
        if let Some(existing) = boxes.iter_mut().find(|b| b.id == infobox.id) {
            // Keep the one with more content
            if infobox.content.as_ref().map(|c| c.len()).unwrap_or(0)
                > existing.content.as_ref().map(|c| c.len()).unwrap_or(0)
            {
                *existing = infobox;
            }
        } else {
            boxes.push(infobox);
        }
    }

    /// Record an unresponsive engine
    pub fn add_unresponsive(&self, name: String, error: EngineError) {
        self.unresponsive_engines
            .write()
            .unwrap()
            .push(UnresponsiveEngine { name, error });
    }

    /// Record engine timing
    pub fn add_timing(&self, timing: Timing) {
        self.timings.write().unwrap().push(timing);
    }

    /// Set redirect URL (for external bangs)
    pub fn set_redirect(&self, url: String) {
        *self.redirect_url.write().unwrap() = Some(url);
    }

    /// Get redirect URL if set
    pub fn get_redirect(&self) -> Option<String> {
        self.redirect_url.read().unwrap().clone()
    }

    /// Get all results sorted by score
    pub fn get_ordered_results(&self) -> Vec<Result> {
        let map = self.results_map.read().unwrap();
        let mut results: Vec<Result> = map.values().cloned().collect();

        // Calculate scores
        for result in &mut results {
            result.calculate_score(&self.engine_weights);
        }

        // Sort by score descending
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

        results
    }

    /// Get paginated results
    pub fn get_results_page(&self, page: usize, per_page: usize) -> Vec<Result> {
        let results = self.get_ordered_results();
        let start = page.saturating_sub(1) * per_page;
        results.into_iter().skip(start).take(per_page).collect()
    }

    /// Get answers
    pub fn get_answers(&self) -> Vec<Answer> {
        self.answers.read().unwrap().clone()
    }

    /// Get suggestions
    pub fn get_suggestions(&self) -> Vec<Suggestion> {
        self.suggestions.read().unwrap().iter().cloned().collect()
    }

    /// Get corrections
    pub fn get_corrections(&self) -> Vec<Correction> {
        self.corrections.read().unwrap().iter().cloned().collect()
    }

    /// Get infoboxes
    pub fn get_infoboxes(&self) -> Vec<InfoBox> {
        self.infoboxes.read().unwrap().clone()
    }

    /// Get unresponsive engines
    pub fn get_unresponsive(&self) -> Vec<UnresponsiveEngine> {
        self.unresponsive_engines.read().unwrap().clone()
    }

    /// Get timings
    pub fn get_timings(&self) -> Vec<Timing> {
        self.timings.read().unwrap().clone()
    }

    /// Get total result count
    pub fn result_count(&self) -> usize {
        self.results_map.read().unwrap().len()
    }

    /// Get number of engines that returned results
    pub fn engine_count(&self) -> usize {
        let map = self.results_map.read().unwrap();
        map.values()
            .flat_map(|r| r.engines.iter())
            .collect::<HashSet<_>>()
            .len()
    }

    /// Create a normalized URL hash for deduplication
    fn url_hash(url: &str) -> String {
        // Normalize URL for better deduplication
        let url = url
            .trim_end_matches('/')
            .replace("https://", "")
            .replace("http://", "")
            .replace("www.", "");

        // Use the normalized URL as the hash key
        url.to_lowercase()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_result_deduplication() {
        let container = ResultContainer::new();

        let r1 = Result::new(
            "https://example.com".to_string(),
            "Example".to_string(),
            "google".to_string(),
        )
        .with_position(1);

        let r2 = Result::new(
            "https://example.com/".to_string(),
            "Example Site".to_string(),
            "bing".to_string(),
        )
        .with_position(2);

        container.add_result(r1);
        container.add_result(r2);

        assert_eq!(container.result_count(), 1);

        let results = container.get_ordered_results();
        assert_eq!(results[0].engines.len(), 2);
    }

    #[test]
    fn test_result_ordering() {
        let container = ResultContainer::new();

        let r1 = Result::new(
            "https://first.com".to_string(),
            "First".to_string(),
            "google".to_string(),
        )
        .with_position(5);

        let r2 = Result::new(
            "https://second.com".to_string(),
            "Second".to_string(),
            "google".to_string(),
        )
        .with_position(1);

        container.add_result(r1);
        container.add_result(r2);

        let results = container.get_ordered_results();
        assert_eq!(results[0].url, "https://second.com");
    }
}
