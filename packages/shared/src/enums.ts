/**
 * Syscall permission families — maps 1:1 to rlmx-kernel SyscallPermission.
 * 17 named permissions + All wildcard (18 variants total).
 */
export enum SyscallPermission {
  VecInsert = 'VecInsert',
  VecSearch = 'VecSearch',
  VecDelete = 'VecDelete',
  GraphQuery = 'GraphQuery',
  GraphCut = 'GraphCut',
  GraphDiffuse = 'GraphDiffuse',
  ProcessFork = 'ProcessFork',
  ProcessSend = 'ProcessSend',
  ProcessRecv = 'ProcessRecv',
  StateMutate = 'StateMutate',
  AttentionSelect = 'AttentionSelect',
  HaltCheck = 'HaltCheck',
  VoiceTranscribe = 'VoiceTranscribe',
  VoiceSynthesize = 'VoiceSynthesize',
  IntentRoute = 'IntentRoute',
  MeshSync = 'MeshSync',
  FederationContribute = 'FederationContribute',
  ArtifactWrite = 'ArtifactWrite',
  All = 'All',
}

/** All SyscallPermission values (excluding All wildcard). */
export const SYSCALL_PERMISSIONS: readonly SyscallPermission[] = [
  SyscallPermission.VecInsert,
  SyscallPermission.VecSearch,
  SyscallPermission.VecDelete,
  SyscallPermission.GraphQuery,
  SyscallPermission.GraphCut,
  SyscallPermission.GraphDiffuse,
  SyscallPermission.ProcessFork,
  SyscallPermission.ProcessSend,
  SyscallPermission.ProcessRecv,
  SyscallPermission.StateMutate,
  SyscallPermission.AttentionSelect,
  SyscallPermission.HaltCheck,
  SyscallPermission.VoiceTranscribe,
  SyscallPermission.VoiceSynthesize,
  SyscallPermission.IntentRoute,
  SyscallPermission.MeshSync,
  SyscallPermission.FederationContribute,
  SyscallPermission.ArtifactWrite,
] as const;

/**
 * Life domain categories for intent decomposition and routing.
 * CLOSED set — exactly 12 variants. Do not extend without ADR approval.
 */
export enum LifeDomain {
  Finance = 'Finance',
  Health = 'Health',
  Legal = 'Legal',
  Career = 'Career',
  Education = 'Education',
  Home = 'Home',
  Shopping = 'Shopping',
  Travel = 'Travel',
  Social = 'Social',
  Government = 'Government',
  Automotive = 'Automotive',
  Pet = 'Pet',
}

/** All LifeDomain values. */
export const LIFE_DOMAINS: readonly LifeDomain[] = [
  LifeDomain.Finance,
  LifeDomain.Health,
  LifeDomain.Legal,
  LifeDomain.Career,
  LifeDomain.Education,
  LifeDomain.Home,
  LifeDomain.Shopping,
  LifeDomain.Travel,
  LifeDomain.Social,
  LifeDomain.Government,
  LifeDomain.Automotive,
  LifeDomain.Pet,
] as const;

/**
 * The 17 specialized agent types in the RLMX swarm.
 * Maps 1:1 to rlmx-agents AgentType enum.
 */
export enum AgentType {
  Coordinator = 'Coordinator',
  Researcher = 'Researcher',
  Router = 'Router',
  Experimenter = 'Experimenter',
  Worker = 'Worker',
  Monitor = 'Monitor',
  Reviewer = 'Reviewer',
  Trainer = 'Trainer',
  Validator = 'Validator',
  Replicator = 'Replicator',
  Embedder = 'Embedder',
  Analyst = 'Analyst',
  VoiceCoordinator = 'VoiceCoordinator',
  MarketplaceManager = 'MarketplaceManager',
  MeshCoordinator = 'MeshCoordinator',
  FederationAgent = 'FederationAgent',
  BillingManager = 'BillingManager',
}

/** All AgentType values. */
export const AGENT_TYPES: readonly AgentType[] = [
  AgentType.Coordinator,
  AgentType.Researcher,
  AgentType.Router,
  AgentType.Experimenter,
  AgentType.Worker,
  AgentType.Monitor,
  AgentType.Reviewer,
  AgentType.Trainer,
  AgentType.Validator,
  AgentType.Replicator,
  AgentType.Embedder,
  AgentType.Analyst,
  AgentType.VoiceCoordinator,
  AgentType.MarketplaceManager,
  AgentType.MeshCoordinator,
  AgentType.FederationAgent,
  AgentType.BillingManager,
] as const;

/**
 * The six subscription tiers.
 * Maps 1:1 to rlmx-billing SubscriptionTier enum.
 */
export enum SubscriptionTier {
  Free = 'Free',
  Personal = 'Personal',
  Family = 'Family',
  Pro = 'Pro',
  Enterprise = 'Enterprise',
  Developer = 'Developer',
}

/** All SubscriptionTier values. */
export const SUBSCRIPTION_TIERS: readonly SubscriptionTier[] = [
  SubscriptionTier.Free,
  SubscriptionTier.Personal,
  SubscriptionTier.Family,
  SubscriptionTier.Pro,
  SubscriptionTier.Enterprise,
  SubscriptionTier.Developer,
] as const;

/**
 * Federation mode determines how voice/agent patterns are shared.
 */
export enum FederationMode {
  /** Can receive federated patterns but never contributes. */
  ReceiveOnly = 'ReceiveOnly',
  /** Full bidirectional federation participation. */
  Full = 'Full',
}

/**
 * How the kernel should respond to a voice-originated request.
 */
export enum ResponseMode {
  /** Audio-only reply (no visual output). */
  VoiceOnly = 'VoiceOnly',
  /** Visual-only reply (screen/text, no audio). */
  Visual = 'Visual',
  /** Combined audio + visual reply. */
  Multimodal = 'Multimodal',
  /** Background/ambient notification (low-priority). */
  Ambient = 'Ambient',
}

/**
 * Persona profile used for voice synthesis style selection.
 */
export enum VoicePersona {
  Finance = 'Finance',
  Health = 'Health',
  Legal = 'Legal',
  Shopping = 'Shopping',
  Calendar = 'Calendar',
  Emergency = 'Emergency',
}
