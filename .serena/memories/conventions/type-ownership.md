# Type Ownership Rules

Each crate owns specific types — never duplicate across crate boundaries.

| Crate | Owned Types |
|-------|-------------|
| `rlmx-kernel` | `SyscallPermission`, `Strategy`, `ProcessId`, `LifeDomain`, `Intent`, `ResponseMode`, `VoicePersona`, `ApprovalTier`, `ApprovalGate`, `TriggerBinding`, `TriggerRegistry`, `AgentCard`, `A2ASkill` |
| `rlmx-mesh` | `MeshId`, `DeviceId`, `DeviceZone`, `MeshDevice` |
| `rlmx-federation` | `FederationCycle`, `Contribution`, `FederationPackage` |
| `rlmx-billing` | `SubscriptionTier`, `TierLimits`, `FamilyGroup`, `BudgetLedger`, `BudgetPolicy` |
| `rlmx-artifact` | `ArtifactId`, `Artifact`, `ArtifactDiff`, `ContentAddressedStore`, `BranchManager` |
| `rlmx-evolve` | `FunctionId`, `EvolvedFunction`, `FunctionStatus`, `ScoreResult`, `FeedbackDecision` |
| `rlmx-channels` | `ChannelAdapter`, `ChannelMessage`, `ChannelRegistry`, `ChannelId` |
| `packages/deploy` | Deploy manifest types, bridge adapters (domain-specific, not in kernel/shared) |

**Rule:** New crate domain events use crate-local enums, NOT the kernel `DomainEvent` enum.
