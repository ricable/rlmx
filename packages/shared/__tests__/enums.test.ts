import { describe, it, expect } from 'vitest';
import {
  SyscallPermission,
  SYSCALL_PERMISSIONS,
  LifeDomain,
  LIFE_DOMAINS,
  AgentType,
  AGENT_TYPES,
  SubscriptionTier,
  SUBSCRIPTION_TIERS,
  FederationMode,
  ResponseMode,
  VoicePersona,
} from '../src/enums.js';

describe('SyscallPermission', () => {
  it('has exactly 17 named permissions (excluding All)', () => {
    expect(SYSCALL_PERMISSIONS).toHaveLength(17);
  });

  it('has 18 total enum values including All', () => {
    const allValues = Object.values(SyscallPermission);
    expect(allValues).toHaveLength(18);
  });

  it('contains all expected variants', () => {
    const expected = [
      'VecInsert', 'VecSearch', 'VecDelete',
      'GraphQuery', 'GraphCut', 'GraphDiffuse',
      'ProcessFork', 'ProcessSend', 'ProcessRecv',
      'StateMutate', 'AttentionSelect', 'HaltCheck',
      'VoiceTranscribe', 'VoiceSynthesize', 'IntentRoute',
      'MeshSync', 'FederationContribute', 'All',
    ];
    for (const variant of expected) {
      expect(SyscallPermission[variant as keyof typeof SyscallPermission]).toBe(variant);
    }
  });

  it('SYSCALL_PERMISSIONS does not include All wildcard', () => {
    expect(SYSCALL_PERMISSIONS).not.toContain(SyscallPermission.All);
  });

  it('string enum values match their keys', () => {
    expect(SyscallPermission.VecInsert).toBe('VecInsert');
    expect(SyscallPermission.FederationContribute).toBe('FederationContribute');
  });
});

describe('LifeDomain', () => {
  it('has exactly 12 variants (CLOSED set)', () => {
    expect(LIFE_DOMAINS).toHaveLength(12);
    expect(Object.values(LifeDomain)).toHaveLength(12);
  });

  it('contains all expected domains', () => {
    const expected = [
      'Finance', 'Health', 'Legal', 'Career', 'Education', 'Home',
      'Shopping', 'Travel', 'Social', 'Government', 'Automotive', 'Pet',
    ];
    expect(LIFE_DOMAINS).toEqual(expected);
  });

  it('LIFE_DOMAINS matches Object.values order', () => {
    expect([...LIFE_DOMAINS]).toEqual(Object.values(LifeDomain));
  });
});

describe('AgentType', () => {
  it('has exactly 17 variants', () => {
    expect(AGENT_TYPES).toHaveLength(17);
    expect(Object.values(AgentType)).toHaveLength(17);
  });

  it('contains all expected types', () => {
    const expected = [
      'Coordinator', 'Researcher', 'Router', 'Experimenter',
      'Worker', 'Monitor', 'Reviewer', 'Trainer',
      'Validator', 'Replicator', 'Embedder', 'Analyst',
      'VoiceCoordinator', 'MarketplaceManager',
      'MeshCoordinator', 'FederationAgent', 'BillingManager',
    ];
    expect(AGENT_TYPES).toEqual(expected);
  });

  it('string enum values match their keys', () => {
    expect(AgentType.Coordinator).toBe('Coordinator');
    expect(AgentType.BillingManager).toBe('BillingManager');
    expect(AgentType.VoiceCoordinator).toBe('VoiceCoordinator');
  });
});

describe('SubscriptionTier', () => {
  it('has exactly 6 tiers', () => {
    expect(SUBSCRIPTION_TIERS).toHaveLength(6);
    expect(Object.values(SubscriptionTier)).toHaveLength(6);
  });

  it('contains all expected tiers', () => {
    const expected = ['Free', 'Personal', 'Family', 'Pro', 'Enterprise', 'Developer'];
    expect(SUBSCRIPTION_TIERS).toEqual(expected);
  });
});

describe('FederationMode', () => {
  it('has exactly 2 variants', () => {
    expect(Object.values(FederationMode)).toHaveLength(2);
  });

  it('has ReceiveOnly and Full', () => {
    expect(FederationMode.ReceiveOnly).toBe('ReceiveOnly');
    expect(FederationMode.Full).toBe('Full');
  });
});

describe('ResponseMode', () => {
  it('has exactly 4 variants', () => {
    expect(Object.values(ResponseMode)).toHaveLength(4);
  });

  it('contains VoiceOnly, Visual, Multimodal, Ambient', () => {
    expect(ResponseMode.VoiceOnly).toBe('VoiceOnly');
    expect(ResponseMode.Visual).toBe('Visual');
    expect(ResponseMode.Multimodal).toBe('Multimodal');
    expect(ResponseMode.Ambient).toBe('Ambient');
  });
});

describe('VoicePersona', () => {
  it('has exactly 6 personas', () => {
    expect(Object.values(VoicePersona)).toHaveLength(6);
  });

  it('contains all expected personas', () => {
    const expected = ['Finance', 'Health', 'Legal', 'Shopping', 'Calendar', 'Emergency'];
    expect(Object.values(VoicePersona)).toEqual(expected);
  });
});
