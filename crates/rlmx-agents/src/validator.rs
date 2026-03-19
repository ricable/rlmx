use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::types::AgentId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub target_id: Uuid,
    pub valid: bool,
    pub chain_length: usize,
    pub errors: Vec<String>,
    pub validated_at: DateTime<Utc>,
}

/// Validator agent — verifies witness chains and proof integrity.
pub struct ValidatorAgent {
    pub id: AgentId,
    pub validations: Vec<ValidationResult>,
}

impl ValidatorAgent {
    pub fn new() -> Self {
        Self {
            id: AgentId::new(),
            validations: Vec::new(),
        }
    }

    /// Validate a witness chain. Each entry must have a "hash" field and
    /// consecutive entries should reference the previous hash via "prev_hash".
    pub fn validate_chain(
        &mut self,
        target_id: Uuid,
        chain: &[serde_json::Value],
    ) -> ValidationResult {
        let mut errors = Vec::new();

        if chain.is_empty() {
            errors.push("Empty chain".into());
            let result = ValidationResult {
                target_id,
                valid: false,
                chain_length: 0,
                errors,
                validated_at: Utc::now(),
            };
            self.validations.push(result.clone());
            return result;
        }

        // Validate each link in the chain
        let mut prev_hash: Option<String> = None;

        for (i, entry) in chain.iter().enumerate() {
            let current_hash = entry.get("hash").and_then(|v| v.as_str()).map(String::from);

            if current_hash.is_none() {
                errors.push(format!("Entry {i} missing 'hash' field"));
            }

            if let Some(ref expected_prev) = prev_hash {
                let entry_prev = entry.get("prev_hash").and_then(|v| v.as_str());

                match entry_prev {
                    Some(actual_prev) if actual_prev == expected_prev => {}
                    Some(actual_prev) => {
                        errors.push(format!(
                            "Entry {i} prev_hash mismatch: expected {expected_prev}, got {actual_prev}"
                        ));
                    }
                    None => {
                        errors.push(format!("Entry {i} missing 'prev_hash' field"));
                    }
                }
            }

            prev_hash = current_hash;
        }

        let valid = errors.is_empty();
        let result = ValidationResult {
            target_id,
            valid,
            chain_length: chain.len(),
            errors,
            validated_at: Utc::now(),
        };

        self.validations.push(result.clone());
        result
    }

    /// Get all validation results.
    pub fn validations(&self) -> &[ValidationResult] {
        &self.validations
    }

    /// Count valid vs invalid validations.
    pub fn stats(&self) -> (usize, usize) {
        let valid = self.validations.iter().filter(|v| v.valid).count();
        let invalid = self.validations.len() - valid;
        (valid, invalid)
    }
}

impl Default for ValidatorAgent {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_empty_chain() {
        let mut validator = ValidatorAgent::new();
        let result = validator.validate_chain(Uuid::new_v4(), &[]);
        assert!(!result.valid);
        assert_eq!(result.chain_length, 0);
        assert!(!result.errors.is_empty());
    }

    #[test]
    fn test_validate_valid_chain() {
        let mut validator = ValidatorAgent::new();
        let chain = vec![
            serde_json::json!({"hash": "abc123", "data": "genesis"}),
            serde_json::json!({"hash": "def456", "prev_hash": "abc123", "data": "block1"}),
            serde_json::json!({"hash": "ghi789", "prev_hash": "def456", "data": "block2"}),
        ];
        let result = validator.validate_chain(Uuid::new_v4(), &chain);
        assert!(result.valid);
        assert_eq!(result.chain_length, 3);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_validate_broken_chain() {
        let mut validator = ValidatorAgent::new();
        let chain = vec![
            serde_json::json!({"hash": "abc123"}),
            serde_json::json!({"hash": "def456", "prev_hash": "WRONG"}),
        ];
        let result = validator.validate_chain(Uuid::new_v4(), &chain);
        assert!(!result.valid);
        assert!(!result.errors.is_empty());
    }

    #[test]
    fn test_validate_missing_hash() {
        let mut validator = ValidatorAgent::new();
        let chain = vec![serde_json::json!({"data": "no hash"})];
        let result = validator.validate_chain(Uuid::new_v4(), &chain);
        assert!(!result.valid);
    }

    #[test]
    fn test_validate_missing_prev_hash() {
        let mut validator = ValidatorAgent::new();
        let chain = vec![
            serde_json::json!({"hash": "abc123"}),
            serde_json::json!({"hash": "def456"}),
        ];
        let result = validator.validate_chain(Uuid::new_v4(), &chain);
        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.contains("prev_hash")));
    }

    #[test]
    fn test_validations_accumulate() {
        let mut validator = ValidatorAgent::new();
        validator.validate_chain(Uuid::new_v4(), &[]);
        validator.validate_chain(Uuid::new_v4(), &[serde_json::json!({"hash": "a"})]);
        assert_eq!(validator.validations().len(), 2);
    }

    #[test]
    fn test_stats() {
        let mut validator = ValidatorAgent::new();
        validator.validate_chain(Uuid::new_v4(), &[]);
        validator.validate_chain(Uuid::new_v4(), &[serde_json::json!({"hash": "a"})]);
        let (valid, invalid) = validator.stats();
        assert_eq!(valid, 1);
        assert_eq!(invalid, 1);
    }
}
