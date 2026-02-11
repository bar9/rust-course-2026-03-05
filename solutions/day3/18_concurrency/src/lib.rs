use std::collections::HashMap;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

/// A thread-safe word frequency counter.
///
/// Words are normalised to lowercase and split on whitespace.
/// Punctuation attached to words is stripped.
pub struct WordCounter {
    counts: Arc<Mutex<HashMap<String, usize>>>,
}

impl WordCounter {
    pub fn new() -> Self {
        WordCounter {
            counts: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Count words in a single string (single-threaded).
    pub fn count(&self, text: &str) {
        let mut map = self.counts.lock().unwrap();
        for word in Self::words(text) {
            *map.entry(word).or_insert(0) += 1;
        }
    }

    /// Count words across multiple texts in parallel (one thread per text).
    pub fn count_parallel(&self, texts: &[&str]) {
        let mut handles = Vec::with_capacity(texts.len());

        for text in texts {
            let text = text.to_string();
            let counts = Arc::clone(&self.counts);
            let handle = thread::spawn(move || {
                let local: HashMap<String, usize> =
                    Self::words(&text).fold(HashMap::new(), |mut acc, w| {
                        *acc.entry(w).or_insert(0) += 1;
                        acc
                    });
                let mut map = counts.lock().unwrap();
                for (word, n) in local {
                    *map.entry(word).or_insert(0) += n;
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }
    }

    /// Get the frequency of a specific word (case-insensitive).
    pub fn get(&self, word: &str) -> usize {
        let map = self.counts.lock().unwrap();
        map.get(&word.to_lowercase()).copied().unwrap_or(0)
    }

    /// Get the top N most frequent words, ordered by count descending,
    /// then alphabetically for ties.
    pub fn top_n(&self, n: usize) -> Vec<(String, usize)> {
        let map = self.counts.lock().unwrap();
        let mut entries: Vec<(String, usize)> =
            map.iter().map(|(k, &v)| (k.clone(), v)).collect();
        entries.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        entries.truncate(n);
        entries
    }

    /// Merge another `WordCounter`'s counts into this one.
    pub fn merge(&self, other: &WordCounter) {
        let other_map = other.counts.lock().unwrap();
        let mut self_map = self.counts.lock().unwrap();
        for (word, &n) in other_map.iter() {
            *self_map.entry(word.clone()).or_insert(0) += n;
        }
    }

    /// Reset all counts to zero.
    pub fn reset(&self) {
        let mut map = self.counts.lock().unwrap();
        map.clear();
    }

    /// Iterate over normalised words: lowercase, punctuation-stripped.
    fn words(text: &str) -> impl Iterator<Item = String> + '_ {
        text.split_whitespace().map(|w| {
            w.trim_matches(|c: char| c.is_ascii_punctuation())
                .to_lowercase()
        }).filter(|w| !w.is_empty())
    }
}

/// Channel-based variant: count words across multiple texts using `mpsc` channels.
///
/// Each text is processed in its own thread. Per-text counts are sent through a
/// channel and merged in the calling thread.
pub fn count_with_channel(texts: &[&str]) -> HashMap<String, usize> {
    let (tx, rx) = mpsc::channel();

    for text in texts {
        let text = text.to_string();
        let tx = tx.clone();
        thread::spawn(move || {
            let mut local = HashMap::new();
            for w in text.split_whitespace() {
                let word = w
                    .trim_matches(|c: char| c.is_ascii_punctuation())
                    .to_lowercase();
                if !word.is_empty() {
                    *local.entry(word).or_insert(0usize) += 1;
                }
            }
            tx.send(local).unwrap();
        });
    }

    drop(tx); // close the original sender so rx iterator terminates

    let mut combined = HashMap::new();
    for partial in rx {
        for (word, n) in partial {
            *combined.entry(word).or_insert(0) += n;
        }
    }
    combined
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Helper ──────────────────────────────────────────────

    fn make_counter(texts: &[&str]) -> WordCounter {
        let wc = WordCounter::new();
        for text in texts {
            wc.count(text);
        }
        wc
    }

    // ── Single-threaded counting ────────────────────────────

    #[test]
    fn count_single_text() {
        let wc = make_counter(&["hello world hello"]);
        assert_eq!(wc.get("hello"), 2);
        assert_eq!(wc.get("world"), 1);
    }

    #[test]
    fn count_ignores_case_and_punctuation() {
        let wc = make_counter(&["Hello, HELLO! hello."]);
        assert_eq!(wc.get("hello"), 3);
    }

    #[test]
    fn count_empty_input() {
        let wc = make_counter(&[""]);
        assert_eq!(wc.get("anything"), 0);
    }

    // ── Parallel counting ───────────────────────────────────

    #[test]
    fn parallel_matches_sequential() {
        let texts = &[
            "the quick brown fox",
            "the slow brown dog",
            "the quick red cat",
        ];

        let seq = WordCounter::new();
        for t in texts {
            seq.count(t);
        }

        let par = WordCounter::new();
        par.count_parallel(texts);

        // Both must agree on every word
        for word in &["the", "quick", "brown", "fox", "slow", "dog", "red", "cat"] {
            assert_eq!(
                seq.get(word),
                par.get(word),
                "mismatch for word '{word}'"
            );
        }
    }

    // ── top_n ───────────────────────────────────────────────

    #[test]
    fn top_n_returns_most_frequent() {
        let wc = make_counter(&["a b a c a b"]);
        let top = wc.top_n(2);
        assert_eq!(top[0], ("a".to_string(), 3));
        assert_eq!(top[1], ("b".to_string(), 2));
    }

    #[test]
    fn top_n_alphabetical_on_tie() {
        let wc = make_counter(&["x y z"]);
        let top = wc.top_n(3);
        // All have count 1, so sorted alphabetically
        assert_eq!(top[0].0, "x");
        assert_eq!(top[1].0, "y");
        assert_eq!(top[2].0, "z");
    }

    // ── merge ───────────────────────────────────────────────

    #[test]
    fn merge_combines_counts() {
        let a = make_counter(&["hello world"]);
        let b = make_counter(&["hello rust"]);
        a.merge(&b);
        assert_eq!(a.get("hello"), 2);
        assert_eq!(a.get("world"), 1);
        assert_eq!(a.get("rust"), 1);
    }

    // ── reset ───────────────────────────────────────────────

    #[test]
    fn reset_clears_all_counts() {
        let wc = make_counter(&["some words here"]);
        assert!(wc.get("some") > 0);
        wc.reset();
        assert_eq!(wc.get("some"), 0);
    }

    // ── Channel-based counting ──────────────────────────────

    #[test]
    fn channel_counting_matches_sequential() {
        let texts = &["one two three", "two three three"];
        let result = count_with_channel(texts);
        assert_eq!(result.get("one"), Some(&1));
        assert_eq!(result.get("two"), Some(&2));
        assert_eq!(result.get("three"), Some(&3));
    }

    // ── Concurrent access safety ────────────────────────────

    #[test]
    fn concurrent_access_is_safe() {
        let wc = Arc::new(WordCounter::new());
        let mut handles = Vec::new();

        for _ in 0..10 {
            let wc = Arc::clone(&wc);
            handles.push(thread::spawn(move || {
                wc.count("word");
            }));
        }

        for h in handles {
            h.join().unwrap();
        }

        assert_eq!(wc.get("word"), 10);
    }
}
