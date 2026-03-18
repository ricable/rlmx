use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::types::{KernelError, KernelResult, SyscallPermission};

/// A capability token that authorises a process to invoke specific syscalls.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityToken {
    pub id: Uuid,
    pub granted_syscalls: Vec<SyscallPermission>,
    pub scope: String,
    pub expiry: DateTime<Utc>,
    /// Placeholder for Ed25519 issuer signature (hex-encoded).
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
}

/// Manages creation, validation, and revocation of capability tokens.
pub struct CapabilityManager {
    tokens: HashMap<Uuid, CapabilityToken>,
    revoked: HashMap<Uuid, DateTime<Utc>>,
}

impl CapabilityManager {
    pub fn new() -> Self {
        Self {
            tokens: HashMap::new(),
            revoked: HashMap::new(),
        }
    }

    /// Create a new capability token with the given permissions and scope.
    /// The token is valid for `ttl` from now.
    pub fn create_token(
        &mut self,
        granted_syscalls: Vec<SyscallPermission>,
        scope: String,
        ttl: Duration,
    ) -> CapabilityToken {
        let token = CapabilityToken {
            id: Uuid::new_v4(),
            granted_syscalls,
            scope,
            expiry: Utc::now() + ttl,
            issuer_signature: "placeholder-signature".into(),
        };
        self.tokens.insert(token.id, token.clone());
        token
    }

    /// Validate that a token is known, not revoked, not expired, and permits
    /// the requested syscall.
    pub fn validate(
        &self,
        token_id: &Uuid,
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

    /// Revoke a token so it can no longer be used.
    pub fn revoke(&mut self, token_id: &Uuid) {
        self.revoked.insert(*token_id, Utc::now());
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
        child_permissions: Vec<SyscallPermission>,
        scope: String,
        ttl: Duration,
    ) -> KernelResult<CapabilityToken> {
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

        let child = CapabilityToken {
            id: Uuid::new_v4(),
            granted_syscalls: child_permissions,
            scope,
            expiry: child_expiry,
            issuer_signature: "placeholder-child-signature".into(),
        };

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

    #[test]
    fn test_create_and_validate_token() {
        let mut mgr = CapabilityManager::new();
        let token = mgr.create_token(
            vec![SyscallPermission::VecInsert, SyscallPermission::VecSearch],
            "test-scope".into(),
            Duration::hours(1),
        );

        assert!(mgr.validate(&token.id, &SyscallPermission::VecInsert).is_ok());
        assert!(mgr.validate(&token.id, &SyscallPermission::VecSearch).is_ok());
        assert!(mgr
            .validate(&token.id, &SyscallPermission::ProcessFork)
            .is_err());
    }

    #[test]
    fn test_revoke_token() {
        let mut mgr = CapabilityManager::new();
        let token = mgr.create_token(
            vec![SyscallPermission::All],
            "admin".into(),
            Duration::hours(1),
        );

        assert!(mgr.validate(&token.id, &SyscallPermission::VecInsert).is_ok());

        mgr.revoke(&token.id);

        assert!(mgr.validate(&token.id, &SyscallPermission::VecInsert).is_err());
    }

    #[test]
    fn test_derive_child_token_subset_only() {
        let mut mgr = CapabilityManager::new();
        let parent = mgr.create_token(
            vec![SyscallPermission::VecInsert, SyscallPermission::VecSearch],
            "parent".into(),
            Duration::hours(2),
        );

        // Child requests only VecInsert -- should succeed.
        let child = mgr
            .derive_child_token(
                &parent.id,
                vec![SyscallPermission::VecInsert],
                "child".into(),
                Duration::hours(1),
            )
            .unwrap();

        assert!(mgr.validate(&child.id, &SyscallPermission::VecInsert).is_ok());
        assert!(mgr.validate(&child.id, &SyscallPermission::VecSearch).is_err());

        // Child requests ProcessFork which parent does not have -- should fail.
        let bad = mgr.derive_child_token(
            &parent.id,
            vec![SyscallPermission::ProcessFork],
            "bad-child".into(),
            Duration::hours(1),
        );
        assert!(bad.is_err());
    }
}
