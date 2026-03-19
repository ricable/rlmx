//! Federation package distribution.
//!
//! A `FederationPackage` wraps an aggregated model for distribution to devices
//! via the RVF container format (SegmentType::TransferPrior). Packages contain:
//! - Aggregated LoRA deltas (overlay).
//! - Top-K pattern bank.
//! - Updated router weights.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tracing::{debug, info};
use uuid::Uuid;

use rlmx_kernel::LifeDomain;

use crate::aggregator::AggregatedModel;
use crate::error::{FederationError, Result};

/// A distributable federation package containing aggregated updates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationPackage {
    /// Unique identifier for this package.
    pub id: Uuid,
    /// The domain this package covers.
    pub domain: LifeDomain,
    /// Which federation cycle produced this package.
    pub cycle_id: Uuid,
    /// Serialized overlay LoRA delta (if present).
    pub overlay_lora: Option<Vec<u8>>,
    /// Serialized pattern bank.
    pub patterns: Vec<u8>,
    /// Serialized router weights (stub: domain confidence as JSON).
    pub router_weights: Vec<u8>,
    /// Semantic version of this package.
    pub version: String,
    /// Size in bytes of the total package.
    pub size_bytes: u64,
    /// SHA-256 checksum over all content segments.
    pub checksum: String,
    /// Number of contributors that produced this package.
    pub contributor_count: usize,
    /// When this package was created.
    pub created_at: DateTime<Utc>,
}

impl FederationPackage {
    /// Build a distribution package from an aggregated model.
    pub fn from_aggregated(
        model: &AggregatedModel,
        cycle_id: Uuid,
        version: impl Into<String>,
    ) -> Result<Self> {
        let version = version.into();

        // Serialize the patterns.
        let patterns_bytes = serde_json::to_vec(&model.patterns).map_err(|e| {
            FederationError::DistributionFailed(format!("pattern serialization failed: {e}"))
        })?;

        // Serialize the LoRA delta if present.
        let lora_bytes = model
            .lora_delta
            .as_ref()
            .map(serde_json::to_vec)
            .transpose()
            .map_err(|e| {
                FederationError::DistributionFailed(format!("lora serialization failed: {e}"))
            })?;

        // Serialize router weights (stub: just the confidence value).
        let router_bytes = serde_json::to_vec(&model.router_confidence).map_err(|e| {
            FederationError::DistributionFailed(format!("router weight serialization failed: {e}"))
        })?;

        // Compute total size.
        let lora_size = lora_bytes.as_ref().map(|b| b.len()).unwrap_or(0);
        let size_bytes = (patterns_bytes.len() + lora_size + router_bytes.len()) as u64;

        // Compute checksum over all content.
        let checksum = {
            let mut hasher = Sha256::new();
            hasher.update(&patterns_bytes);
            if let Some(ref lb) = lora_bytes {
                hasher.update(lb);
            }
            hasher.update(&router_bytes);
            hex::encode(hasher.finalize())
        };

        info!(
            domain = ?model.domain,
            contributor_count = model.contributor_count,
            size_bytes,
            version = %version,
            "built federation package"
        );

        Ok(Self {
            id: Uuid::new_v4(),
            domain: model.domain,
            cycle_id,
            overlay_lora: lora_bytes,
            patterns: patterns_bytes,
            router_weights: router_bytes,
            version,
            size_bytes,
            checksum,
            contributor_count: model.contributor_count,
            created_at: Utc::now(),
        })
    }

    /// Verify the integrity of this package by recomputing its checksum.
    pub fn verify_integrity(&self) -> bool {
        let mut hasher = Sha256::new();
        hasher.update(&self.patterns);
        if let Some(ref lb) = self.overlay_lora {
            hasher.update(lb);
        }
        hasher.update(&self.router_weights);
        let computed = hex::encode(hasher.finalize());
        computed == self.checksum
    }

    /// Check if this package contains a LoRA overlay.
    pub fn has_lora(&self) -> bool {
        self.overlay_lora.is_some()
    }
}

/// Manages distribution of federation packages to devices.
#[derive(Debug, Default)]
pub struct PackageDistributor {
    /// All published packages, keyed by domain.
    pub packages: Vec<FederationPackage>,
}

impl PackageDistributor {
    /// Create a new, empty distributor.
    pub fn new() -> Self {
        Self {
            packages: Vec::new(),
        }
    }

    /// Publish a package for distribution.
    pub fn publish(&mut self, package: FederationPackage) {
        debug!(
            domain = ?package.domain,
            version = %package.version,
            "publishing federation package"
        );
        self.packages.push(package);
    }

    /// Get the latest package for a given domain.
    pub fn latest_for_domain(&self, domain: LifeDomain) -> Option<&FederationPackage> {
        self.packages
            .iter()
            .filter(|p| p.domain == domain)
            .max_by_key(|p| p.created_at)
    }

    /// Get a package by its ID.
    pub fn get_by_id(&self, id: Uuid) -> Result<&FederationPackage> {
        self.packages
            .iter()
            .find(|p| p.id == id)
            .ok_or(FederationError::PackageNotFound(id))
    }

    /// List all available packages.
    pub fn list_all(&self) -> &[FederationPackage] {
        &self.packages
    }

    /// Total number of published packages.
    pub fn count(&self) -> usize {
        self.packages.len()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use rlmx_cognitive::sona::LoraDelta;
    use rlmx_cognitive::voice_patterns::{AnonymizedPattern, Modality};

    fn make_aggregated_model(domain: LifeDomain, with_lora: bool) -> AggregatedModel {
        let patterns = vec![AnonymizedPattern {
            id: Uuid::new_v4(),
            sanitized_embedding: vec![0.1; 8],
            actions_taken: vec!["action".into()],
            result_quality: 0.9,
            emotion_bucket: None,
            noisy_urgency: 0.5,
            noisy_satisfaction: None,
            interaction_modality: Modality::Voice,
            timestamp: Utc::now(),
        }];

        let lora = if with_lora {
            Some(LoraDelta {
                layer_name: "layer_0".into(),
                delta_a: vec![0.1, 0.2],
                delta_b: vec![0.3],
                rank: 1,
                applied_at: Utc::now(),
            })
        } else {
            None
        };

        AggregatedModel {
            id: Uuid::new_v4(),
            domain,
            contributor_count: 1500,
            patterns,
            lora_delta: lora,
            router_confidence: 0.85,
            created_at: Utc::now(),
        }
    }

    #[test]
    fn test_package_from_aggregated() {
        let model = make_aggregated_model(LifeDomain::Finance, true);
        let cycle_id = Uuid::new_v4();
        let pkg = FederationPackage::from_aggregated(&model, cycle_id, "1.0.0").unwrap();
        assert_eq!(pkg.domain, LifeDomain::Finance);
        assert_eq!(pkg.cycle_id, cycle_id);
        assert_eq!(pkg.version, "1.0.0");
        assert!(pkg.has_lora());
        assert!(pkg.size_bytes > 0);
        assert!(!pkg.checksum.is_empty());
    }

    #[test]
    fn test_package_integrity_verification() {
        let model = make_aggregated_model(LifeDomain::Health, false);
        let pkg = FederationPackage::from_aggregated(&model, Uuid::new_v4(), "1.0.0").unwrap();
        assert!(pkg.verify_integrity());
    }

    #[test]
    fn test_package_integrity_tampered() {
        let model = make_aggregated_model(LifeDomain::Health, false);
        let mut pkg = FederationPackage::from_aggregated(&model, Uuid::new_v4(), "1.0.0").unwrap();
        pkg.patterns.push(0xFF); // tamper
        assert!(!pkg.verify_integrity());
    }

    #[test]
    fn test_distributor_publish_and_latest() {
        let mut dist = PackageDistributor::new();
        let model = make_aggregated_model(LifeDomain::Finance, false);
        let pkg = FederationPackage::from_aggregated(&model, Uuid::new_v4(), "1.0.0").unwrap();
        dist.publish(pkg);

        assert_eq!(dist.count(), 1);
        let latest = dist.latest_for_domain(LifeDomain::Finance);
        assert!(latest.is_some());
        assert_eq!(latest.unwrap().version, "1.0.0");
    }

    #[test]
    fn test_distributor_latest_returns_none_for_missing_domain() {
        let dist = PackageDistributor::new();
        assert!(dist.latest_for_domain(LifeDomain::Legal).is_none());
    }
}
