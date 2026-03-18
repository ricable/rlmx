//! Context Window Management for RLM agents.
//!
//! Manages what fits within the model's context window using approximate
//! token counting and priority-based segment selection. Ensures the most
//! relevant context segments are included first.

use serde::{Deserialize, Serialize};

use crate::vllm::ChatMessage;

/// A segment of context that can be included in the model's context window.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextSegment {
    /// Unique identifier for this segment.
    pub id: String,
    /// The content of this context segment.
    pub content: String,
    /// Relevance score (higher = more relevant). Used for priority ordering.
    pub relevance: f64,
    /// Source of this segment (e.g., "vec_search", "user_input", "sub_agent").
    pub source: String,
    /// Approximate token count for this segment.
    pub token_estimate: usize,
}

impl ContextSegment {
    /// Create a new context segment with automatic token estimation.
    pub fn new(id: impl Into<String>, content: impl Into<String>, relevance: f64, source: impl Into<String>) -> Self {
        let content = content.into();
        let token_estimate = estimate_tokens(&content);
        Self {
            id: id.into(),
            content,
            relevance,
            source: source.into(),
            token_estimate,
        }
    }
}

/// Approximate token count using chars/4 heuristic.
pub fn estimate_tokens(text: &str) -> usize {
    // Rough approximation: 1 token ~= 4 characters for English text
    (text.len() + 3) / 4
}

/// Manages the model's context window, ensuring content fits within
/// the token budget while prioritizing the most relevant segments.
#[derive(Debug, Clone)]
pub struct ContextWindow {
    /// Maximum tokens allowed in the context window.
    max_tokens: usize,
    /// Tokens currently consumed by added segments.
    used_tokens: usize,
    /// Tokens reserved for the system prompt and query overhead.
    reserved_tokens: usize,
    /// Context segments added to the window, sorted by relevance.
    segments: Vec<ContextSegment>,
}

impl ContextWindow {
    /// Create a new context window with the given maximum token budget.
    ///
    /// Reserves 1024 tokens by default for system prompt and query overhead.
    pub fn new(max_tokens: usize) -> Self {
        Self {
            max_tokens,
            used_tokens: 0,
            reserved_tokens: 1024,
            segments: Vec::new(),
        }
    }

    /// Set the number of tokens reserved for system prompt and query.
    pub fn with_reserved_tokens(mut self, reserved: usize) -> Self {
        self.reserved_tokens = reserved;
        self
    }

    /// Returns the available token budget for context segments.
    pub fn available_tokens(&self) -> usize {
        self.max_tokens
            .saturating_sub(self.reserved_tokens)
            .saturating_sub(self.used_tokens)
    }

    /// Returns the total number of tokens currently used by segments.
    pub fn used_tokens(&self) -> usize {
        self.used_tokens
    }

    /// Returns the number of segments in the window.
    pub fn segment_count(&self) -> usize {
        self.segments.len()
    }

    /// Returns true if the window has no segments.
    pub fn is_empty(&self) -> bool {
        self.segments.is_empty()
    }

    /// Try to add a context segment to the window.
    ///
    /// Returns `true` if the segment was added, `false` if there is not
    /// enough room in the context window. Segments are maintained in
    /// descending relevance order.
    pub fn add_segment(&mut self, segment: ContextSegment) -> bool {
        if segment.token_estimate > self.available_tokens() {
            return false;
        }

        self.used_tokens += segment.token_estimate;
        self.segments.push(segment);

        // Re-sort by relevance (highest first)
        self.segments
            .sort_by(|a, b| b.relevance.partial_cmp(&a.relevance).unwrap_or(std::cmp::Ordering::Equal));

        true
    }

    /// Add multiple segments, selecting the most relevant ones that fit.
    ///
    /// Segments are sorted by relevance and added greedily until the
    /// context window is full. Returns the number of segments added.
    pub fn add_segments_by_priority(&mut self, mut segments: Vec<ContextSegment>) -> usize {
        // Sort by relevance descending
        segments.sort_by(|a, b| b.relevance.partial_cmp(&a.relevance).unwrap_or(std::cmp::Ordering::Equal));

        let mut added = 0;
        for segment in segments {
            if self.add_segment(segment) {
                added += 1;
            }
        }
        added
    }

    /// Build the full prompt messages from the context window.
    ///
    /// Combines the system prompt, context segments, and user query into
    /// a list of chat messages suitable for the vLLM API.
    pub fn build_prompt(&self, system_prompt: &str, query: &str) -> Vec<ChatMessage> {
        let mut messages = vec![ChatMessage::system(system_prompt)];

        if !self.segments.is_empty() {
            let context_text = self
                .segments
                .iter()
                .enumerate()
                .map(|(i, seg)| {
                    format!(
                        "[Context {}] (source: {}, relevance: {:.2})\n{}",
                        i + 1,
                        seg.source,
                        seg.relevance,
                        seg.content
                    )
                })
                .collect::<Vec<_>>()
                .join("\n\n");

            messages.push(ChatMessage::user(format!(
                "Available context:\n{}\n\nQuery: {}",
                context_text, query
            )));
        } else {
            messages.push(ChatMessage::user(query.to_string()));
        }

        messages
    }

    /// Get a reference to all segments in the window.
    pub fn segments(&self) -> &[ContextSegment] {
        &self.segments
    }

    /// Clear all segments from the window.
    pub fn clear(&mut self) {
        self.segments.clear();
        self.used_tokens = 0;
    }
}

impl Default for ContextWindow {
    fn default() -> Self {
        Self::new(8192)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_segments_and_ordering() {
        let mut window = ContextWindow::new(8192);

        let seg1 = ContextSegment::new("s1", "First segment content", 0.5, "test");
        let seg2 = ContextSegment::new("s2", "Second segment more relevant", 0.9, "test");
        let seg3 = ContextSegment::new("s3", "Third segment least relevant", 0.2, "test");

        assert!(window.add_segment(seg1));
        assert!(window.add_segment(seg2));
        assert!(window.add_segment(seg3));

        assert_eq!(window.segment_count(), 3);

        // Segments should be ordered by relevance (highest first)
        let segments = window.segments();
        assert_eq!(segments[0].id, "s2");
        assert_eq!(segments[1].id, "s1");
        assert_eq!(segments[2].id, "s3");
    }

    #[test]
    fn test_context_window_overflow() {
        // Create a very small window (256 tokens, minus 1024 reserved = no space)
        // Actually, let's make it have just enough for one small segment
        let mut window = ContextWindow::new(1100).with_reserved_tokens(1024);

        // Available: 1100 - 1024 = 76 tokens
        assert_eq!(window.available_tokens(), 76);

        // This segment (~7 tokens) should fit
        let small = ContextSegment::new("small", "short text here", 0.8, "test");
        assert!(window.add_segment(small));

        // After adding, available should decrease
        assert!(window.available_tokens() < 76);

        // This segment is too large to fit in the remaining space
        let large_content = "x".repeat(400); // ~100 tokens
        let large = ContextSegment::new("large", large_content, 0.9, "test");
        assert!(!window.add_segment(large));

        // Only the small segment should be present
        assert_eq!(window.segment_count(), 1);
    }

    #[test]
    fn test_build_prompt_with_context() {
        let mut window = ContextWindow::new(8192);
        let seg = ContextSegment::new("s1", "Relevant information here", 0.8, "vec_search");
        window.add_segment(seg);

        let messages = window.build_prompt("You are a helpful assistant.", "What happened?");
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].role, "system");
        assert!(messages[1].content.contains("Relevant information here"));
        assert!(messages[1].content.contains("What happened?"));
    }

    #[test]
    fn test_build_prompt_without_context() {
        let window = ContextWindow::new(8192);
        let messages = window.build_prompt("System prompt", "User query");
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[1].content, "User query");
    }

    #[test]
    fn test_token_estimation() {
        assert_eq!(estimate_tokens(""), 0);
        assert_eq!(estimate_tokens("abcd"), 1);
        assert_eq!(estimate_tokens("abcdefgh"), 2);
        // 100 chars -> 25 tokens
        let hundred_chars = "a".repeat(100);
        assert_eq!(estimate_tokens(&hundred_chars), 25);
    }
}
