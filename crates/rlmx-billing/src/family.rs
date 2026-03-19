//! Family plan: shared groups with per-member privacy boundaries.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::{BillingError, BillingResult};

/// Maximum members in a family group.
pub const MAX_FAMILY_MEMBERS: usize = 6;

/// Role within a family group.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FamilyRole {
    /// Group owner — manages membership and billing.
    Owner,
    /// Adult member — full access within privacy boundary.
    Adult,
    /// Child member — restricted access, parental controls.
    Child,
}

/// Privacy boundary for a family member, controlling data visibility.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyBoundary {
    /// Whether this member's agent usage is visible to the owner.
    pub usage_visible_to_owner: bool,
    /// Whether this member's voice transcripts are shared.
    pub transcripts_shared: bool,
    /// Whether this member's agent data is isolated from other members.
    pub data_isolated: bool,
}

impl Default for PrivacyBoundary {
    fn default() -> Self {
        Self {
            usage_visible_to_owner: true,
            transcripts_shared: false,
            data_isolated: true,
        }
    }
}

impl PrivacyBoundary {
    /// Privacy boundary for child members: owner-visible, isolated data.
    pub fn child_default() -> Self {
        Self {
            usage_visible_to_owner: true,
            transcripts_shared: false,
            data_isolated: true,
        }
    }
}

/// A member within a family group.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FamilyMember {
    pub user_id: Uuid,
    pub role: FamilyRole,
    pub privacy_boundary: PrivacyBoundary,
    pub joined_at: DateTime<Utc>,
}

/// A family group sharing a Family-tier subscription.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FamilyGroup {
    pub id: Uuid,
    pub owner: Uuid,
    pub members: Vec<FamilyMember>,
    /// Agent IDs shared across the family group.
    pub shared_agents: Vec<Uuid>,
    pub created_at: DateTime<Utc>,
}

impl FamilyGroup {
    /// Create a new family group with the given owner.
    pub fn new(owner_id: Uuid) -> Self {
        let owner_member = FamilyMember {
            user_id: owner_id,
            role: FamilyRole::Owner,
            privacy_boundary: PrivacyBoundary {
                usage_visible_to_owner: true,
                transcripts_shared: false,
                data_isolated: false,
            },
            joined_at: Utc::now(),
        };
        Self {
            id: Uuid::new_v4(),
            owner: owner_id,
            members: vec![owner_member],
            shared_agents: Vec::new(),
            created_at: Utc::now(),
        }
    }

    /// Add a member to the family group.
    pub fn add_member(
        &mut self,
        user_id: Uuid,
        role: FamilyRole,
        privacy: PrivacyBoundary,
    ) -> BillingResult<()> {
        if self.members.len() >= MAX_FAMILY_MEMBERS {
            return Err(BillingError::FamilyGroupFull {
                max: MAX_FAMILY_MEMBERS,
            });
        }
        if self.members.iter().any(|m| m.user_id == user_id) {
            return Err(BillingError::MemberAlreadyExists(user_id));
        }
        self.members.push(FamilyMember {
            user_id,
            role,
            privacy_boundary: privacy,
            joined_at: Utc::now(),
        });
        tracing::info!(family_id = %self.id, user_id = %user_id, role = ?role, "family member added");
        Ok(())
    }

    /// Remove a member from the family group. Cannot remove the owner.
    pub fn remove_member(&mut self, user_id: Uuid) -> BillingResult<()> {
        if user_id == self.owner {
            return Err(BillingError::CannotRemoveOwner);
        }
        let before = self.members.len();
        self.members.retain(|m| m.user_id != user_id);
        if self.members.len() == before {
            return Err(BillingError::MemberNotFound(user_id));
        }
        tracing::info!(family_id = %self.id, user_id = %user_id, "family member removed");
        Ok(())
    }

    /// Share an agent with all family members.
    pub fn share_agent(&mut self, agent_id: Uuid) {
        if !self.shared_agents.contains(&agent_id) {
            self.shared_agents.push(agent_id);
            tracing::info!(family_id = %self.id, agent_id = %agent_id, "agent shared with family");
        }
    }

    /// Unshare an agent from the family group.
    pub fn unshare_agent(&mut self, agent_id: &Uuid) {
        self.shared_agents.retain(|a| a != agent_id);
    }

    /// Get a member by user_id.
    pub fn get_member(&self, user_id: &Uuid) -> Option<&FamilyMember> {
        self.members.iter().find(|m| &m.user_id == user_id)
    }

    /// Number of members in the group.
    pub fn member_count(&self) -> usize {
        self.members.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_family_group() {
        let owner = Uuid::new_v4();
        let group = FamilyGroup::new(owner);
        assert_eq!(group.owner, owner);
        assert_eq!(group.member_count(), 1);
        assert_eq!(group.members[0].role, FamilyRole::Owner);
    }

    #[test]
    fn add_and_remove_members() {
        let owner = Uuid::new_v4();
        let mut group = FamilyGroup::new(owner);

        let member = Uuid::new_v4();
        group
            .add_member(member, FamilyRole::Adult, PrivacyBoundary::default())
            .unwrap();
        assert_eq!(group.member_count(), 2);

        group.remove_member(member).unwrap();
        assert_eq!(group.member_count(), 1);
    }

    #[test]
    fn cannot_remove_owner() {
        let owner = Uuid::new_v4();
        let mut group = FamilyGroup::new(owner);
        assert!(group.remove_member(owner).is_err());
    }

    #[test]
    fn max_members_enforced() {
        let owner = Uuid::new_v4();
        let mut group = FamilyGroup::new(owner);
        // Add 5 more (total 6)
        for _ in 0..5 {
            group
                .add_member(
                    Uuid::new_v4(),
                    FamilyRole::Adult,
                    PrivacyBoundary::default(),
                )
                .unwrap();
        }
        assert_eq!(group.member_count(), MAX_FAMILY_MEMBERS);
        // 7th should fail
        let result = group.add_member(
            Uuid::new_v4(),
            FamilyRole::Adult,
            PrivacyBoundary::default(),
        );
        assert!(result.is_err());
    }

    #[test]
    fn duplicate_member_rejected() {
        let owner = Uuid::new_v4();
        let mut group = FamilyGroup::new(owner);
        let member = Uuid::new_v4();
        group
            .add_member(member, FamilyRole::Adult, PrivacyBoundary::default())
            .unwrap();
        let result = group.add_member(member, FamilyRole::Adult, PrivacyBoundary::default());
        assert!(result.is_err());
    }

    #[test]
    fn share_and_unshare_agent() {
        let owner = Uuid::new_v4();
        let mut group = FamilyGroup::new(owner);
        let agent = Uuid::new_v4();

        group.share_agent(agent);
        assert_eq!(group.shared_agents.len(), 1);

        // Idempotent
        group.share_agent(agent);
        assert_eq!(group.shared_agents.len(), 1);

        group.unshare_agent(&agent);
        assert!(group.shared_agents.is_empty());
    }

    #[test]
    fn remove_nonexistent_member() {
        let owner = Uuid::new_v4();
        let mut group = FamilyGroup::new(owner);
        assert!(group.remove_member(Uuid::new_v4()).is_err());
    }
}
