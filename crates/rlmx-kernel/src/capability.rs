use chrono::{DateTime, Duration, Utc};
use hmac::{Hmac, Mac};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::collections::HashMap;
use uuid::Uuid;

use crate::types::{KernelError, KernelResult, ProcessId, SyscallPermission};

type HmacSha256 = Hmac<Sha256>;

/// A capability token that authorises a process to invoke specific syscalls.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityToken {
    pub id: Uuid,
    pub owner: ProcessId,
    pub granted_syscalls: Vec<SyscallPermission>,
    pub scope: String,
    pub expiry: DateTime<Utc>,
    /// HMAC-SHA256 signature over the token payload (hex-encoded).
    pub issuer_signature: String,
}

impl CapabilityToken {
    /// Check whether this token permits the given syscall permission.
    pub fn permits(&self, permission: &SyscallPermission) -> bool {
        self.granted_syscalls.contains(&SyscallPermission::All)
            || self.granted_syscalls.contains(permission)
    }

    /// Check whether the token has expired.
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expiry
    }

    /// Compute the payload bytes used for signing/verifying this token.
    fn signing_payload(&self) -> Vec<u8> {
        // Deterministic payload: id + owner + syscalls + scope + expiry
        let mut payload = Vec::new();
        payload.extend_from_slice(self.id.as_bytes());
        payload.extend_from_slice(self.owner.as_bytes());
        for perm in &self.granted_syscalls {
            let s = format!("{:?}", perm);
            payload.extend_from_slice(s.as_bytes());
        }
        payload.extend_from_slice(self.scope.as_bytes());
        payload.extend_from_slice(self.expiry.to_rfc3339().as_bytes());
        payload
    }
}

/// Manages creation, validation, and revocation of capability tokens.
pub struct CapabilityManager {
    tokens: HashMap<Uuid, CapabilityToken>,
    revoked: HashMap<Uuid, DateTime<Utc>>,
    /// HMAC signing key for token signatures.
    signing_key: Vec<u8>,
}

impl CapabilityManager {
    pub fn new() -> Self {
        let mut key = vec![0u8; 32];
        rand::thread_rng().fill_bytes(&mut key);
        Self::with_key(key)
    }

    /// Create a CapabilityManager with a specific HMAC signing key.
    pub fn with_key(signing_key: Vec<u8>) -> Self {
        Self {
            tokens: HashMap::new(),
            revoked: HashMap::new(),
            signing_key,
        }
    }

    /// Compute HMAC-SHA256 signature for a token.
    fn sign(&self, token: &CapabilityToken) -> String {
        let mut mac = HmacSha256::new_from_slice(&self.signing_key)
            .expect("HMAC accepts any key length");
        mac.update(&token.signing_payload());
        hex::encode(mac.finalize().into_bytes())
    }

    /// Verify that a token's signature is valid.
    fn verify_signature(&self, token: &CapabilityToken) -> bool {
        // Constant-time comparison via hmac verify
        let mut mac = HmacSha256::new_from_slice(&self.signing_key)
            .expect("HMAC accepts any key length");
        mac.update(&token.signing_payload());
        let sig_bytes = match hex::decode(&token.issuer_signature) {
            Ok(b) => b,
            Err(_) => return false,
        };
        mac.verify_slice(&sig_bytes).is_ok()
    }

    /// Create a new capability token with the given permissions and scope.
    /// The token is valid for `ttl` from now.
    pub fn create_token(
        &mut self,
        owner: ProcessId,
        granted_syscalls: Vec<SyscallPermission>,
        scope: String,
        ttl: Duration,
    ) -> CapabilityToken {
        let mut token = CapabilityToken {
            id: Uuid::new_v4(),
            owner,
            granted_syscalls,
            scope,
            expiry: Utc::now() + ttl,
            issuer_signature: String::new(),
        };
        token.issuer_signature = self.sign(&token);
        self.tokens.insert(token.id, token.clone());
        token
    }

    /// Validate that a token is known, not revoked, not expired, has a valid
    /// signature, belongs to the specified owner, and permits the requested syscall.
    pub fn validate(
        &self,
        token_id: &Uuid,
        owner: &ProcessId,
        permission: &SyscallPermission,
    ) -> KernelResult<()> {
        if self.revoked.contains_key(token_id) {
            return Err(KernelError::CapabilityDenied(
                "token has been revoked".into(),
            ));
        }

        let token = self
            .tokens
            .get(token_id)
            .ok_or_else(|| KernelError::CapabilityDenied("unknown token".into()))?;

        if token.owner != *owner {
            return Err(KernelError::CapabilityDenied(
                "token owner mismatch".into(),
            ));
        }

        if !self.verify_signature(token) {
            return Err(KernelError::CapabilityDenied(
                "invalid token signature".into(),
            ));
        }

        if token.is_expired() {
            return Err(KernelError::CapabilityDenied("token expired".into()));
        }

        if !token.permits(permission) {
            return Err(KernelError::CapabilityDenied(format!(
                "token does not permit {:?}",
                permission
            )));
        }

        Ok(())
    }

    /// Revoke a token so it can no longer be used, and remove it from the tokens map.
    pub fn revoke(&mut self, token_id: &Uuid) {
        self.revoked.insert(*token_id, Utc::now());
        self.tokens.remove(token_id);
    }

    /// Look up a token by id.
    pub fn get_token(&self, token_id: &Uuid) -> Option<&CapabilityToken> {
        self.tokens.get(token_id)
    }

    /// Create a child token that is a subset of a parent token's permissions.
    /// The child may only have permissions the parent already has.
    pub fn derive_child_token(
        &mut self,
        parent_id: &Uuid,
        child_owner: ProcessId,
        child_permissions: Vec<SyscallPermission>,
        scope: String,
        ttl: Duration,
    ) -> KernelResult<CapabilityToken> {
        // Check if parent is revoked.
        if self.revoked.contains_key(parent_id) {
            return Err(KernelError::CapabilityDenied(
                "parent token has been revoked".into(),
            ));
        }

        let parent = self
            .tokens
            .get(parent_id)
            .ok_or_else(|| KernelError::CapabilityDenied("parent token not found".into()))?;

        // Ensure parent is still valid.
        if parent.is_expired() {
            return Err(KernelError::CapabilityDenied("parent token expired".into()));
        }

        // A child cannot request permissions the parent does not have,
        // unless the parent has All.
        if !parent.granted_syscalls.contains(&SyscallPermission::All) {
            for perm in &child_permissions {
                if !parent.granted_syscalls.contains(perm) {
                    return Err(KernelError::CapabilityDenied(format!(
                        "parent does not grant {:?}",
                        perm
                    )));
                }
            }
        }

        // Child expiry cannot exceed parent expiry.
        let child_expiry = std::cmp::min(Utc::now() + ttl, parent.expiry);

        let mut child = CapabilityToken {
            id: Uuid::new_v4(),
            owner: child_owner,
            granted_syscalls: child_permissions,
            scope,
            expiry: child_expiry,
            issuer_signature: String::new(),
        };
        child.issuer_signature = self.sign(&child);

        self.tokens.insert(child.id, child.clone());
        Ok(child)
    }
}

impl Default for CapabilityManager {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn test_owner() -> ProcessId {
        Uuid::new_v4()
    }

    #[test]
    fn test_create_and_validate_token() {
        let mut mgr = CapabilityManager::new();
        let owner = test_owner();
        let token = mgr.create_token(
            owner,
            vec![SyscallPermission::VecInsert, SyscallPermission::VecSearch],
            "test-scope".into(),
            Duration::hours(1),
        );

        assert!(mgr.validate(&token.id, &owner, &SyscallPermission::VecInsert).is_ok());
        assert!(mgr.validate(&token.id, &owner, &SyscallPermission::VecSearch).is_ok());
        assert!(mgr
            .validate(&token.id, &owner, &SyscallPermission::ProcessFork)
            .is_err());
    }

    #[test]
    fn test_validate_wrong_owner() {
        let mut mgr = CapabilityManager::new();
        let owner = test_owner();
        let wrong_owner = test_owner();
        let token = mgr.create_token(
            owner,
            vec![SyscallPermission::All],
            "admin".into(),
            Duration::hours(1),
        );

        assert!(mgr.validate(&token.id, &wrong_owner, &SyscallPermission::VecInsert).is_err());
    }

    #[test]
    fn test_signature_verification() {
        let mut mgr = CapabilityManager::new();
        let owner = test_owner();
        let token = mgr.create_token(
            owner,
            vec![SyscallPermission::All],
            "admin".into(),
            Duration::hours(1),
        );

        // Valid signature
        assert!(mgr.verify_signature(&token));

        // Tampered signature
        let mut tampered = token.clone();
        tampered.issuer_signature = "deadbeef".into();
        assert!(!mgr.verify_signature(&tampered));
    }

    #[test]
    fn test_revoke_token() {
        let mut mgr = CapabilityManager::new();
        let owner = test_owner();
        let token = mgr.create_token(
            owner,
            vec![SyscallPermission::All],
            "admin".into(),
            Duration::hours(1),
        );

        assert!(mgr.validate(&token.id, &owner, &SyscallPermission::VecInsert).is_ok());

        mgr.revoke(&token.id);

        // Token should be revoked and removed from tokens map
        assert!(mgr.validate(&token.id, &owner, &SyscallPermission::VecInsert).is_err());
        assert!(mgr.get_token(&token.id).is_none());
    }

    #[test]
    fn test_derive_child_token_subset_only() {
        let mut mgr = CapabilityManager::new();
        let parent_owner = test_owner();
        let child_owner = test_owner();
        let parent = mgr.create_token(
            parent_owner,
            vec![SyscallPermission::VecInsert, SyscallPermission::VecSearch],
            "parent".into(),
            Duration::hours(2),
        );

        // Child requests only VecInsert -- should succeed.
        let child = mgr
            .derive_child_token(
                &parent.id,
                child_owner,
                vec![SyscallPermission::VecInsert],
                "child".into(),
                Duration::hours(1),
            )
            .unwrap();

        assert!(mgr.validate(&child.id, &child_owner, &SyscallPermission::VecInsert).is_ok());
        assert!(mgr.validate(&child.id, &child_owner, &SyscallPermission::VecSearch).is_err());

        // Child requests ProcessFork which parent does not have -- should fail.
        let bad = mgr.derive_child_token(
            &parent.id,
            child_owner,
            vec![SyscallPermission::ProcessFork],
            "bad-child".into(),
            Duration::hours(1),
        );
        assert!(bad.is_err());
    }

    #[test]
    fn test_derive_child_from_revoked_parent_fails() {
        let mut mgr = CapabilityManager::new();
        let parent_owner = test_owner();
        let child_owner = test_owner();
        let parent = mgr.create_token(
            parent_owner,
            vec![SyscallPermission::All],
            "parent".into(),
            Duration::hours(2),
        );

        mgr.revoke(&parent.id);

        let result = mgr.derive_child_token(
            &parent.id,
            child_owner,
            vec![SyscallPermission::VecInsert],
            "child".into(),
            Duration::hours(1),
        );
        assert!(result.is_err());
    }
}
