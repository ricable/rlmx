//! Intent decomposition: multi-intent extraction from a single utterance.
//!
//! A single voice utterance may reference multiple life domains (e.g.,
//! "Cancel my dentist appointment and order more dog food" yields intents
//! in Health and Shopping domains). The `MultiIntentDecomposer` handles
//! this decomposition with domain embedding similarity (stub cosine sim).

pub use rlmx_kernel::{Intent, LifeDomain};
use serde::{Deserialize, Serialize};

/// All 12 kernel life domain variants for iteration.
const ALL_LIFE_DOMAINS: [LifeDomain; 12] = [
    LifeDomain::Finance,
    LifeDomain::Health,
    LifeDomain::Legal,
    LifeDomain::Career,
    LifeDomain::Education,
    LifeDomain::Home,
    LifeDomain::Shopping,
    LifeDomain::Travel,
    LifeDomain::Social,
    LifeDomain::Government,
    LifeDomain::Automotive,
    LifeDomain::Pet,
];

/// Returns stub keyword associations for domain classification.
fn keywords_for_domain(domain: &LifeDomain) -> &'static [&'static str] {
    match domain {
        LifeDomain::Finance => &[
            "pay", "money", "transfer", "balance", "bank", "invest", "bill", "budget", "savings",
            "stock", "crypto", "price",
        ],
        LifeDomain::Health => &[
            "doctor",
            "appointment",
            "medicine",
            "health",
            "dentist",
            "symptom",
            "prescription",
            "exercise",
            "workout",
            "calories",
            "hospital",
            "emergency",
            "help",
            "urgent",
            "911",
            "fire",
            "ambulance",
            "police",
            "danger",
            "accident",
            "sos",
        ],
        LifeDomain::Legal => &[
            "lawyer",
            "contract",
            "sue",
            "legal",
            "court",
            "rights",
            "law",
            "attorney",
            "compliance",
            "regulation",
        ],
        LifeDomain::Career => &[
            "todo", "task", "note", "list", "project", "deadline", "plan", "organize", "workflow",
            "focus",
        ],
        LifeDomain::Education => &[
            "learn",
            "study",
            "course",
            "class",
            "teach",
            "homework",
            "exam",
            "lecture",
            "tutorial",
            "school",
            "university",
            "schedule",
            "meeting",
            "remind",
            "calendar",
            "event",
            "tomorrow",
            "today",
            "book",
            "cancel",
            "reschedule",
        ],
        LifeDomain::Home => &[
            "light",
            "thermostat",
            "lock",
            "alarm",
            "camera",
            "smart",
            "temperature",
            "door",
            "garage",
            "vacuum",
        ],
        LifeDomain::Shopping => &[
            "buy", "order", "purchase", "cart", "shop", "price", "deal", "discount", "deliver",
            "product", "food", "grocery",
        ],
        LifeDomain::Travel => &[
            "flight",
            "hotel",
            "travel",
            "trip",
            "destination",
            "airport",
            "train",
            "uber",
            "lyft",
            "taxi",
        ],
        LifeDomain::Social => &[
            "play",
            "music",
            "movie",
            "game",
            "watch",
            "listen",
            "show",
            "stream",
            "concert",
            "podcast",
            "call",
            "text",
            "message",
            "email",
            "send",
            "reply",
            "chat",
            "contact",
            "phone",
            "notification",
        ],
        LifeDomain::Government => &[
            "tax", "license", "passport", "permit", "visa", "census", "vote", "dmv", "registry",
            "filing",
        ],
        LifeDomain::Automotive => &[
            "car",
            "gas",
            "parking",
            "mechanic",
            "tire",
            "oil",
            "engine",
            "mileage",
            "insurance",
            "drive",
        ],
        LifeDomain::Pet => &[
            "vet", "dog", "cat", "pet", "walk", "feed", "groom", "kibble", "leash", "puppy",
            "kitten",
        ],
    }
}

/// An entity extracted from a transcript segment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedEntity {
    /// The entity type (e.g., "date", "amount", "location").
    pub entity_type: String,
    /// The extracted value.
    pub value: String,
    /// Character offsets in the transcript (start, end).
    pub span: (usize, usize),
}

/// Convert extracted entities to simple string values for kernel Intent.
fn entities_to_strings(entities: &[ExtractedEntity]) -> Vec<String> {
    entities.iter().map(|e| e.value.clone()).collect()
}

/// Classifies a transcript into a primary life domain.
///
/// Stub implementation using keyword matching. In production this would
/// delegate to the expanded TinyDancerRouter (18-dim input per ADR-019).
pub struct IntentClassifier {
    /// Minimum confidence threshold for accepting a classification.
    pub confidence_threshold: f32,
}

impl IntentClassifier {
    pub fn new(confidence_threshold: f32) -> Self {
        Self {
            confidence_threshold,
        }
    }

    /// Classify a transcript into a life domain with confidence.
    ///
    /// Returns `None` if no domain scores above the confidence threshold.
    pub fn classify(&self, transcript: &str) -> Option<(LifeDomain, f32)> {
        let lower = transcript.to_lowercase();
        let mut best: Option<(LifeDomain, f32)> = None;

        for domain in &ALL_LIFE_DOMAINS {
            let score = Self::keyword_score(&lower, keywords_for_domain(domain));
            if score > self.confidence_threshold && best.as_ref().is_none_or(|(_, s)| score > *s) {
                best = Some((*domain, score));
            }
        }

        best
    }

    fn keyword_score(text: &str, keywords: &[&str]) -> f32 {
        let words: Vec<&str> = text.split_whitespace().collect();
        if words.is_empty() {
            return 0.0;
        }
        let hits = keywords
            .iter()
            .filter(|kw| words.iter().any(|w| w.contains(**kw)))
            .count();
        // Normalize: at least one hit gives a base confidence, more hits
        // increase it up to a cap of 0.95.
        if hits == 0 {
            return 0.0;
        }
        (0.4 + 0.55 * (hits as f32 / keywords.len() as f32).min(1.0)).min(0.95)
    }
}

/// Decomposes a single utterance into multiple `Intent` structs spanning
/// different life domains. Uses clause splitting and per-clause classification.
pub struct MultiIntentDecomposer {
    classifier: IntentClassifier,
}

impl MultiIntentDecomposer {
    pub fn new(confidence_threshold: f32) -> Self {
        Self {
            classifier: IntentClassifier::new(confidence_threshold),
        }
    }

    /// Decompose a transcript into zero or more intents, sorted by urgency
    /// (highest first). Intents with confidence below 0.3 are discarded
    /// per DDD-008 invariant.
    pub fn decompose(&self, transcript: &str) -> Vec<Intent> {
        let clauses = Self::split_clauses(transcript);
        let mut intents = Vec::new();

        for clause in &clauses {
            if let Some((domain, confidence)) = self.classifier.classify(clause) {
                if confidence < 0.3 {
                    tracing::debug!(confidence, clause, "discarding low-confidence intent");
                    continue;
                }

                let action = Self::extract_action(clause);
                let entities = Self::extract_entities(clause, transcript);
                let urgency = Self::estimate_urgency(clause, domain);

                intents.push(Intent {
                    domain,
                    action,
                    entities: entities_to_strings(&entities),
                    urgency: urgency as f64,
                    confidence: confidence as f64,
                });
            }
        }

        // Sort by urgency descending (highest first).
        intents.sort_by(|a, b| {
            b.urgency
                .partial_cmp(&a.urgency)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        intents
    }

    /// Split a transcript into clauses using conjunctions and punctuation.
    fn split_clauses(transcript: &str) -> Vec<String> {
        let delimiters = [" and ", " then ", " also ", " plus ", ", and ", ". "];
        let mut parts = vec![transcript.to_string()];

        for delim in &delimiters {
            let mut new_parts = Vec::new();
            for part in &parts {
                for segment in part.split(delim) {
                    let trimmed = segment.trim().to_string();
                    if !trimmed.is_empty() {
                        new_parts.push(trimmed);
                    }
                }
            }
            parts = new_parts;
        }

        parts
    }

    /// Extract the primary action verb from a clause (stub).
    fn extract_action(clause: &str) -> String {
        let action_words = [
            "cancel", "book", "order", "buy", "pay", "send", "call", "search", "compare",
            "schedule", "remind", "play", "turn", "set", "check", "find", "get", "start", "stop",
            "open", "close", "lock", "unlock",
        ];
        let lower = clause.to_lowercase();
        for word in &action_words {
            if lower.contains(word) {
                return word.to_string();
            }
        }
        "query".to_string()
    }

    /// Extract entities from a clause (stub implementation).
    fn extract_entities(clause: &str, full_transcript: &str) -> Vec<ExtractedEntity> {
        let mut entities = Vec::new();

        // Detect monetary amounts (very simplistic).
        let lower = clause.to_lowercase();
        if let Some(pos) = lower.find('$') {
            let value_end = lower[pos + 1..]
                .find(|c: char| !c.is_ascii_digit() && c != '.' && c != ',')
                .map(|i| pos + 1 + i)
                .unwrap_or(lower.len());
            let value = &lower[pos..value_end];
            if value.len() > 1 {
                let offset = full_transcript.to_lowercase().find(value).unwrap_or(pos);
                entities.push(ExtractedEntity {
                    entity_type: "amount".to_string(),
                    value: value.to_string(),
                    span: (offset, offset + value.len()),
                });
            }
        }

        // Detect time-related words.
        let time_words = [
            "tomorrow",
            "today",
            "tonight",
            "monday",
            "tuesday",
            "wednesday",
            "thursday",
            "friday",
            "saturday",
            "sunday",
            "morning",
            "afternoon",
            "evening",
        ];
        for tw in &time_words {
            if let Some(pos) = lower.find(tw) {
                let offset = full_transcript.to_lowercase().find(tw).unwrap_or(pos);
                entities.push(ExtractedEntity {
                    entity_type: "date".to_string(),
                    value: tw.to_string(),
                    span: (offset, offset + tw.len()),
                });
            }
        }

        entities
    }

    /// Estimate urgency based on linguistic markers and domain.
    fn estimate_urgency(clause: &str, domain: LifeDomain) -> f32 {
        let lower = clause.to_lowercase();
        let mut urgency: f32 = 0.3; // base urgency

        // Health domain with emergency keywords always has high urgency.
        let emergency_words = ["emergency", "911", "ambulance", "fire", "sos", "danger"];
        if domain == LifeDomain::Health && emergency_words.iter().any(|w| lower.contains(w)) {
            return 1.0;
        }

        let urgent_markers = [
            "urgent",
            "asap",
            "immediately",
            "right now",
            "hurry",
            "emergency",
            "critical",
            "important",
        ];
        for marker in &urgent_markers {
            if lower.contains(marker) {
                urgency += 0.3;
            }
        }

        // Education items for today/tomorrow get a bump (calendar-like).
        if domain == LifeDomain::Education
            && (lower.contains("today") || lower.contains("tomorrow"))
        {
            urgency += 0.2;
        }

        urgency.min(1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_life_domains_returns_12() {
        assert_eq!(ALL_LIFE_DOMAINS.len(), 12);
    }

    #[test]
    fn test_classifier_finance() {
        let clf = IntentClassifier::new(0.3);
        let result = clf.classify("pay my electricity bill");
        assert!(result.is_some());
        let (domain, conf) = result.unwrap();
        assert_eq!(domain, LifeDomain::Finance);
        assert!(conf >= 0.3);
    }

    #[test]
    fn test_classifier_health() {
        let clf = IntentClassifier::new(0.3);
        let result = clf.classify("refill my prescription medicine from the doctor");
        assert!(result.is_some());
        let (domain, _) = result.unwrap();
        assert_eq!(domain, LifeDomain::Health);
    }

    #[test]
    fn test_classifier_empty_returns_none() {
        let clf = IntentClassifier::new(0.3);
        assert!(clf.classify("").is_none());
    }

    #[test]
    fn test_classifier_gibberish_returns_none() {
        let clf = IntentClassifier::new(0.3);
        assert!(clf.classify("xyzzy plugh foo bar baz").is_none());
    }

    #[test]
    fn test_decomposer_single_intent() {
        let decomposer = MultiIntentDecomposer::new(0.3);
        let intents = decomposer.decompose("order more dog food");
        assert!(!intents.is_empty());
        assert_eq!(intents[0].domain, LifeDomain::Shopping);
        assert_eq!(intents[0].action, "order");
    }

    #[test]
    fn test_decomposer_multi_intent() {
        let decomposer = MultiIntentDecomposer::new(0.3);
        let intents = decomposer.decompose("cancel my dentist appointment and order more dog food");
        assert!(
            intents.len() >= 2,
            "expected at least 2 intents, got {}",
            intents.len()
        );

        let domains: Vec<LifeDomain> = intents.iter().map(|i| i.domain).collect();
        assert!(
            domains.contains(&LifeDomain::Shopping),
            "missing Shopping domain"
        );
    }

    #[test]
    fn test_decomposer_empty_transcript() {
        let decomposer = MultiIntentDecomposer::new(0.3);
        let intents = decomposer.decompose("");
        assert!(intents.is_empty());
    }

    #[test]
    fn test_decomposer_urgency_ordering() {
        let decomposer = MultiIntentDecomposer::new(0.3);
        let intents =
            decomposer.decompose("call 911 for an emergency and also order some groceries");
        if intents.len() >= 2 {
            assert!(
                intents[0].urgency >= intents[1].urgency,
                "intents should be sorted by urgency descending"
            );
        }
    }

    #[test]
    fn test_extract_action_cancel() {
        let action = MultiIntentDecomposer::extract_action("cancel my subscription");
        assert_eq!(action, "cancel");
    }

    #[test]
    fn test_extract_action_default() {
        let action = MultiIntentDecomposer::extract_action("something without a known verb");
        assert_eq!(action, "query");
    }

    #[test]
    fn test_entity_extraction_amount() {
        let entities = MultiIntentDecomposer::extract_entities("pay $50 for it", "pay $50 for it");
        assert!(!entities.is_empty());
        assert_eq!(entities[0].entity_type, "amount");
    }

    #[test]
    fn test_entity_extraction_date() {
        let entities = MultiIntentDecomposer::extract_entities(
            "schedule for tomorrow",
            "schedule for tomorrow",
        );
        let dates: Vec<_> = entities
            .iter()
            .filter(|e| e.entity_type == "date")
            .collect();
        assert!(!dates.is_empty());
        assert_eq!(dates[0].value, "tomorrow");
    }

    #[test]
    fn test_emergency_always_max_urgency() {
        let urgency =
            MultiIntentDecomposer::estimate_urgency("call 911 emergency", LifeDomain::Health);
        assert!((urgency - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_urgency_marker_boost() {
        let base = MultiIntentDecomposer::estimate_urgency("buy groceries", LifeDomain::Shopping);
        let boosted = MultiIntentDecomposer::estimate_urgency(
            "buy groceries urgent asap",
            LifeDomain::Shopping,
        );
        assert!(boosted > base);
    }

    #[test]
    fn test_split_clauses() {
        let clauses = MultiIntentDecomposer::split_clauses("do A and then do B also do C");
        assert!(
            clauses.len() >= 3,
            "expected >= 3 clauses, got {:?}",
            clauses
        );
    }
}
