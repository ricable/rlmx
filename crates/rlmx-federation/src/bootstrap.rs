//! New user bootstrap: load federated pattern packs for instant agent intelligence.
//!
//! When a new user installs an agent, the bootstrap process:
//! 1. Downloads the latest federated pattern pack for that domain.
//! 2. Imports anonymized patterns into the agent's SONA bank.
//! 3. Applies the aggregated LoRA delta (with EWC++ preservation).
//! 4. The agent is immediately competent — no personal history needed.

use tracing::{debug, info, warn};

use rlmx_cognitive::sona::{LoraDelta, Sona};
use rlmx_cognitive::voice_patterns::AnonymizedPattern;
use rlmx_kernel::LifeDomain;

use crate::distribution::{FederationPackage, PackageDistributor};
use crate::error::{FederationError, Result};

/// Result of bootstrapping a SONA instance from a federated package.
#[derive(Debug)]
pub struct BootstrapResult {
    /// Number of patterns imported.
    pub patterns_imported: usize,
    /// Whether a LoRA delta was applied.
    pub lora_applied: bool,
    /// The domain that was bootstrapped.
    pub domain: LifeDomain,
    /// Package version used.
    pub package_version: String,
}

/// Bootstrap a SONA instance from the latest federated package for a domain.
///
/// This is the primary entry point for new-user agent bootstrapping.
pub fn bootstrap_from_latest(
    sona: &mut Sona,
    distributor: &PackageDistributor,
    domain: LifeDomain,
) -> Result<BootstrapResult> {
    let package = distributor.latest_for_domain(domain).ok_or_else(|| {
        FederationError::BootstrapFailed(format!(
            "no federated package available for domain {domain:?}"
        ))
    })?;

    bootstrap_from_package(sona, package)
}

/// Bootstrap a SONA instance from a specific federation package.
pub fn bootstrap_from_package(
    sona: &mut Sona,
    package: &FederationPackage,
) -> Result<BootstrapResult> {
    // Verify package integrity before importing.
    if !package.verify_integrity() {
        return Err(FederationError::BootstrapFailed(
            "package integrity check failed".into(),
        ));
    }

    info!(
        domain = ?package.domain,
        version = %package.version,
        "bootstrapping SONA from federated package"
    );

    // Deserialize patterns.
    let patterns: Vec<AnonymizedPattern> =
        serde_json::from_slice(&package.patterns).map_err(|e| {
            FederationError::BootstrapFailed(format!("pattern deserialization failed: {e}"))
        })?;

    // Import patterns into SONA's pattern bank.
    let patterns_imported = import_patterns(sona, &patterns);

    // Apply LoRA delta if present.
    let lora_applied = if let Some(ref lora_bytes) = package.overlay_lora {
        match apply_federated_lora(sona, lora_bytes) {
            Ok(()) => true,
            Err(e) => {
                warn!(error = %e, "failed to apply federated LoRA, continuing without it");
                false
            }
        }
    } else {
        false
    };

    debug!(patterns_imported, lora_applied, "bootstrap complete");

    Ok(BootstrapResult {
        patterns_imported,
        lora_applied,
        domain: package.domain,
        package_version: package.version.clone(),
    })
}

/// Import anonymized patterns into a SONA pattern bank.
///
/// Converts anonymized patterns back into SONA-compatible format.
/// Returns the number of patterns successfully imported.
fn import_patterns(sona: &mut Sona, patterns: &[AnonymizedPattern]) -> usize {
    let mut imported = 0;
    for pattern in patterns {
        // Convert anonymized pattern to a SONA record.
        // We use the sanitized embedding as the query text approximation.
        let action_str = pattern.actions_taken.join(",");
        let quality = pattern.result_quality as f64;

        sona.record_pattern(&action_str, pattern.actions_taken.clone(), quality);
        imported += 1;
    }
    imported
}

/// Apply a federated LoRA delta to a SONA instance.
///
/// Uses EWC++ regularization to preserve any existing personal patterns.
fn apply_federated_lora(sona: &mut Sona, lora_bytes: &[u8]) -> Result<()> {
    let delta: LoraDelta = serde_json::from_slice(lora_bytes).map_err(|e| {
        FederationError::BootstrapFailed(format!("lora deserialization failed: {e}"))
    })?;

    // Apply via SONA's adaptation mechanism with a small quality delta
    // to indicate this is a federated import, not a local improvement.
    let feedback = rlmx_cognitive::sona::AdaptationFeedback {
        layer_name: delta.layer_name,
        gradient: delta.delta_a.clone(),
        rank: delta.rank,
        quality_delta: 0.01, // Small positive delta for federated imports.
    };

    sona.adapt(feedback)
        .map_err(|e| FederationError::BootstrapFailed(format!("lora adaptation failed: {e}")))?;

    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aggregator::AggregatedModel;
    use crate::distribution::FederationPackage;
    use chrono::Utc;
    use rlmx_cognitive::voice_patterns::Modality;
    use uuid::Uuid;

    fn make_package(domain: LifeDomain, with_lora: bool) -> FederationPackage {
        let patterns = vec![AnonymizedPattern {
            id: Uuid::new_v4(),
            sanitized_embedding: vec![0.1; 8],
            actions_taken: vec!["test_action".into()],
            result_quality: 0.85,
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

        let model = AggregatedModel {
            id: Uuid::new_v4(),
            domain,
            contributor_count: 1500,
            patterns,
            lora_delta: lora,
            router_confidence: 0.85,
            created_at: Utc::now(),
        };

        FederationPackage::from_aggregated(&model, Uuid::new_v4(), "1.0.0").unwrap()
    }

    #[test]
    fn test_bootstrap_from_package_no_lora() {
        let mut sona = Sona::new();
        let pkg = make_package(LifeDomain::Finance, false);
        let result = bootstrap_from_package(&mut sona, &pkg).unwrap();
        assert_eq!(result.patterns_imported, 1);
        assert!(!result.lora_applied);
        assert_eq!(result.domain, LifeDomain::Finance);
        assert_eq!(sona.pattern_bank.patterns.len(), 1);
    }

    #[test]
    fn test_bootstrap_from_package_with_lora() {
        let mut sona = Sona::new();
        let pkg = make_package(LifeDomain::Health, true);
        let result = bootstrap_from_package(&mut sona, &pkg).unwrap();
        assert_eq!(result.patterns_imported, 1);
        assert!(result.lora_applied);
        assert_eq!(sona.lora_deltas.len(), 1);
    }

    #[test]
    fn test_bootstrap_from_latest() {
        let mut sona = Sona::new();
        let mut dist = PackageDistributor::new();
        dist.publish(make_package(LifeDomain::Finance, false));
        let result = bootstrap_from_latest(&mut sona, &dist, LifeDomain::Finance).unwrap();
        assert_eq!(result.patterns_imported, 1);
    }

    #[test]
    fn test_bootstrap_from_latest_missing_domain() {
        let mut sona = Sona::new();
        let dist = PackageDistributor::new();
        let result = bootstrap_from_latest(&mut sona, &dist, LifeDomain::Legal);
        assert!(result.is_err());
    }

    #[test]
    fn test_bootstrap_tampered_package_fails() {
        let mut sona = Sona::new();
        let mut pkg = make_package(LifeDomain::Finance, false);
        pkg.patterns.push(0xFF); // tamper
        let result = bootstrap_from_package(&mut sona, &pkg);
        assert!(result.is_err());
    }
}
