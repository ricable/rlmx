import { describe, it, expect } from 'vitest';
import { AgentType, SyscallPermission, AGENT_TYPES, SYSCALL_PERMISSIONS } from '@aix/shared';
import { AgentPermissions, PermissionRegistry } from '../src/registry.js';

// ---------------------------------------------------------------------------
// AgentPermissions unit tests
// ---------------------------------------------------------------------------

describe('AgentPermissions', () => {
  it('starts empty when constructed with no args', () => {
    const perms = new AgentPermissions();
    expect(perms.count).toBe(0);
    expect(perms.has(SyscallPermission.VecSearch)).toBe(false);
  });

  it('initializes from an iterable', () => {
    const perms = new AgentPermissions([SyscallPermission.VecSearch, SyscallPermission.VecInsert]);
    expect(perms.count).toBe(2);
    expect(perms.has(SyscallPermission.VecSearch)).toBe(true);
    expect(perms.has(SyscallPermission.VecInsert)).toBe(true);
  });

  it('grant adds a permission', () => {
    const perms = new AgentPermissions([SyscallPermission.VecSearch]);
    perms.grant(SyscallPermission.VecInsert);
    expect(perms.has(SyscallPermission.VecInsert)).toBe(true);
    expect(perms.count).toBe(2);
  });

  it('revoke removes a permission', () => {
    const perms = new AgentPermissions([SyscallPermission.VecSearch, SyscallPermission.VecInsert]);
    perms.revoke(SyscallPermission.VecSearch);
    expect(perms.has(SyscallPermission.VecSearch)).toBe(false);
    expect(perms.count).toBe(1);
  });

  it('is iterable', () => {
    const input = [SyscallPermission.VecSearch, SyscallPermission.ProcessSend];
    const perms = new AgentPermissions(input);
    const result = [...perms];
    expect(result).toHaveLength(2);
    expect(result).toContain(SyscallPermission.VecSearch);
    expect(result).toContain(SyscallPermission.ProcessSend);
  });

  it('toArray returns frozen copy', () => {
    const perms = new AgentPermissions([SyscallPermission.VecSearch]);
    const arr = perms.toArray();
    expect(Object.isFrozen(arr)).toBe(true);
    expect(arr).toContain(SyscallPermission.VecSearch);
  });
});

// ---------------------------------------------------------------------------
// PermissionRegistry — Coordinator (full access)
// ---------------------------------------------------------------------------

describe('PermissionRegistry — Coordinator', () => {
  it('has all 17 concrete permissions', () => {
    const perms = PermissionRegistry.permissionsFor(AgentType.Coordinator);
    expect(perms.count).toBe(17);
    for (const p of SYSCALL_PERMISSIONS) {
      expect(perms.has(p)).toBe(true);
    }
  });

  it('can spawn all types except Coordinator', () => {
    for (const t of AGENT_TYPES) {
      if (t === AgentType.Coordinator) {
        expect(PermissionRegistry.canSpawn(AgentType.Coordinator, t)).toBe(false);
      } else {
        expect(PermissionRegistry.canSpawn(AgentType.Coordinator, t)).toBe(true);
      }
    }
  });
});

// ---------------------------------------------------------------------------
// PermissionRegistry — Universal invariants (all 17 types)
// ---------------------------------------------------------------------------

describe('PermissionRegistry — universal invariants', () => {
  it('all 17 agent types have VecSearch', () => {
    for (const t of AGENT_TYPES) {
      const perms = PermissionRegistry.permissionsFor(t);
      expect(perms.has(SyscallPermission.VecSearch)).toBe(true);
    }
  });

  it('all 17 agent types have ProcessSend and ProcessRecv', () => {
    for (const t of AGENT_TYPES) {
      const perms = PermissionRegistry.permissionsFor(t);
      expect(perms.has(SyscallPermission.ProcessSend)).toBe(true);
      expect(perms.has(SyscallPermission.ProcessRecv)).toBe(true);
    }
  });

  it('all 17 agent types have VoiceSynthesize and IntentRoute', () => {
    for (const t of AGENT_TYPES) {
      const perms = PermissionRegistry.permissionsFor(t);
      expect(perms.has(SyscallPermission.VoiceSynthesize)).toBe(true);
      expect(perms.has(SyscallPermission.IntentRoute)).toBe(true);
    }
  });

  it('exactly 17 agent types are covered', () => {
    expect(AGENT_TYPES).toHaveLength(17);
  });

  it('permissionsFor returns non-empty for all types', () => {
    for (const t of AGENT_TYPES) {
      expect(PermissionRegistry.permissionsFor(t).count).toBeGreaterThan(0);
    }
  });
});

// ---------------------------------------------------------------------------
// PermissionRegistry — VoiceTranscribe is restricted
// ---------------------------------------------------------------------------

describe('PermissionRegistry — VoiceTranscribe restriction', () => {
  const VOICE_TRANSCRIBE_AGENTS: AgentType[] = [
    AgentType.Coordinator,
    AgentType.Router,
    AgentType.VoiceCoordinator,
  ];

  it('only Coordinator, Router, and VoiceCoordinator have VoiceTranscribe', () => {
    for (const t of AGENT_TYPES) {
      const perms = PermissionRegistry.permissionsFor(t);
      const shouldHave = VOICE_TRANSCRIBE_AGENTS.includes(t);
      expect(perms.has(SyscallPermission.VoiceTranscribe)).toBe(shouldHave);
    }
  });
});

// ---------------------------------------------------------------------------
// PermissionRegistry — MeshSync is restricted
// ---------------------------------------------------------------------------

describe('PermissionRegistry — MeshSync restriction', () => {
  it('only Coordinator and MeshCoordinator have MeshSync', () => {
    for (const t of AGENT_TYPES) {
      const perms = PermissionRegistry.permissionsFor(t);
      const shouldHave = t === AgentType.Coordinator || t === AgentType.MeshCoordinator;
      expect(perms.has(SyscallPermission.MeshSync)).toBe(shouldHave);
    }
  });
});

// ---------------------------------------------------------------------------
// PermissionRegistry — FederationContribute is restricted
// ---------------------------------------------------------------------------

describe('PermissionRegistry — FederationContribute restriction', () => {
  it('only Coordinator and FederationAgent have FederationContribute', () => {
    for (const t of AGENT_TYPES) {
      const perms = PermissionRegistry.permissionsFor(t);
      const shouldHave = t === AgentType.Coordinator || t === AgentType.FederationAgent;
      expect(perms.has(SyscallPermission.FederationContribute)).toBe(shouldHave);
    }
  });
});

// ---------------------------------------------------------------------------
// PermissionRegistry — ProcessFork matches canFork
// ---------------------------------------------------------------------------

describe('PermissionRegistry — ProcessFork consistency', () => {
  const FORK_AGENTS: AgentType[] = [
    AgentType.Coordinator,
    AgentType.Researcher,
    AgentType.Experimenter,
    AgentType.VoiceCoordinator,
    AgentType.MeshCoordinator,
  ];

  it('ProcessFork permission matches the canFork set', () => {
    for (const t of AGENT_TYPES) {
      const perms = PermissionRegistry.permissionsFor(t);
      const shouldHaveFork = FORK_AGENTS.includes(t);
      expect(perms.has(SyscallPermission.ProcessFork)).toBe(shouldHaveFork);
    }
  });
});

// ---------------------------------------------------------------------------
// PermissionRegistry — individual agent type tests (1:1 with Rust tests)
// ---------------------------------------------------------------------------

describe('PermissionRegistry — Router', () => {
  it('cannot mutate state', () => {
    const perms = PermissionRegistry.permissionsFor(AgentType.Router);
    expect(perms.has(SyscallPermission.StateMutate)).toBe(false);
  });

  it('has VoiceTranscribe', () => {
    const perms = PermissionRegistry.permissionsFor(AgentType.Router);
    expect(perms.has(SyscallPermission.VoiceTranscribe)).toBe(true);
  });

  it('has HaltCheck', () => {
    const perms = PermissionRegistry.permissionsFor(AgentType.Router);
    expect(perms.has(SyscallPermission.HaltCheck)).toBe(true);
  });
});

describe('PermissionRegistry — Validator', () => {
  it('cannot mutate state', () => {
    const perms = PermissionRegistry.permissionsFor(AgentType.Validator);
    expect(perms.has(SyscallPermission.StateMutate)).toBe(false);
  });

  it('has HaltCheck', () => {
    const perms = PermissionRegistry.permissionsFor(AgentType.Validator);
    expect(perms.has(SyscallPermission.HaltCheck)).toBe(true);
  });

  it('does not have ProcessFork', () => {
    const perms = PermissionRegistry.permissionsFor(AgentType.Validator);
    expect(perms.has(SyscallPermission.ProcessFork)).toBe(false);
  });
});

describe('PermissionRegistry — Embedder', () => {
  it('has VecInsert and VecSearch', () => {
    const perms = PermissionRegistry.permissionsFor(AgentType.Embedder);
    expect(perms.has(SyscallPermission.VecInsert)).toBe(true);
    expect(perms.has(SyscallPermission.VecSearch)).toBe(true);
  });

  it('does not have StateMutate, GraphQuery, or ProcessFork', () => {
    const perms = PermissionRegistry.permissionsFor(AgentType.Embedder);
    expect(perms.has(SyscallPermission.StateMutate)).toBe(false);
    expect(perms.has(SyscallPermission.GraphQuery)).toBe(false);
    expect(perms.has(SyscallPermission.ProcessFork)).toBe(false);
  });
});

describe('PermissionRegistry — Researcher', () => {
  it('has ProcessFork and AttentionSelect', () => {
    const perms = PermissionRegistry.permissionsFor(AgentType.Researcher);
    expect(perms.has(SyscallPermission.ProcessFork)).toBe(true);
    expect(perms.has(SyscallPermission.AttentionSelect)).toBe(true);
  });

  it('has GraphQuery and GraphDiffuse', () => {
    const perms = PermissionRegistry.permissionsFor(AgentType.Researcher);
    expect(perms.has(SyscallPermission.GraphQuery)).toBe(true);
    expect(perms.has(SyscallPermission.GraphDiffuse)).toBe(true);
  });

  it('does not have StateMutate', () => {
    const perms = PermissionRegistry.permissionsFor(AgentType.Researcher);
    expect(perms.has(SyscallPermission.StateMutate)).toBe(false);
  });
});

describe('PermissionRegistry — VoiceCoordinator', () => {
  it('has VoiceTranscribe, VoiceSynthesize, and IntentRoute', () => {
    const perms = PermissionRegistry.permissionsFor(AgentType.VoiceCoordinator);
    expect(perms.has(SyscallPermission.VoiceTranscribe)).toBe(true);
    expect(perms.has(SyscallPermission.VoiceSynthesize)).toBe(true);
    expect(perms.has(SyscallPermission.IntentRoute)).toBe(true);
  });

  it('has ProcessFork and AttentionSelect', () => {
    const perms = PermissionRegistry.permissionsFor(AgentType.VoiceCoordinator);
    expect(perms.has(SyscallPermission.ProcessFork)).toBe(true);
    expect(perms.has(SyscallPermission.AttentionSelect)).toBe(true);
  });

  it('does not have StateMutate', () => {
    const perms = PermissionRegistry.permissionsFor(AgentType.VoiceCoordinator);
    expect(perms.has(SyscallPermission.StateMutate)).toBe(false);
  });
});

describe('PermissionRegistry — MarketplaceManager', () => {
  it('has StateMutate, VecInsert, VecDelete', () => {
    const perms = PermissionRegistry.permissionsFor(AgentType.MarketplaceManager);
    expect(perms.has(SyscallPermission.StateMutate)).toBe(true);
    expect(perms.has(SyscallPermission.VecInsert)).toBe(true);
    expect(perms.has(SyscallPermission.VecDelete)).toBe(true);
  });

  it('has VoiceSynthesize and IntentRoute', () => {
    const perms = PermissionRegistry.permissionsFor(AgentType.MarketplaceManager);
    expect(perms.has(SyscallPermission.VoiceSynthesize)).toBe(true);
    expect(perms.has(SyscallPermission.IntentRoute)).toBe(true);
  });

  it('does not have ProcessFork or VoiceTranscribe', () => {
    const perms = PermissionRegistry.permissionsFor(AgentType.MarketplaceManager);
    expect(perms.has(SyscallPermission.ProcessFork)).toBe(false);
    expect(perms.has(SyscallPermission.VoiceTranscribe)).toBe(false);
  });
});

describe('PermissionRegistry — MeshCoordinator', () => {
  it('has MeshSync, ProcessFork, GraphQuery', () => {
    const perms = PermissionRegistry.permissionsFor(AgentType.MeshCoordinator);
    expect(perms.has(SyscallPermission.MeshSync)).toBe(true);
    expect(perms.has(SyscallPermission.ProcessFork)).toBe(true);
    expect(perms.has(SyscallPermission.GraphQuery)).toBe(true);
  });

  it('has VoiceSynthesize and IntentRoute', () => {
    const perms = PermissionRegistry.permissionsFor(AgentType.MeshCoordinator);
    expect(perms.has(SyscallPermission.VoiceSynthesize)).toBe(true);
    expect(perms.has(SyscallPermission.IntentRoute)).toBe(true);
  });

  it('does not have VoiceTranscribe or StateMutate', () => {
    const perms = PermissionRegistry.permissionsFor(AgentType.MeshCoordinator);
    expect(perms.has(SyscallPermission.VoiceTranscribe)).toBe(false);
    expect(perms.has(SyscallPermission.StateMutate)).toBe(false);
  });
});

describe('PermissionRegistry — FederationAgent', () => {
  it('has FederationContribute, VecSearch, AttentionSelect', () => {
    const perms = PermissionRegistry.permissionsFor(AgentType.FederationAgent);
    expect(perms.has(SyscallPermission.FederationContribute)).toBe(true);
    expect(perms.has(SyscallPermission.VecSearch)).toBe(true);
    expect(perms.has(SyscallPermission.AttentionSelect)).toBe(true);
  });

  it('has VoiceSynthesize and IntentRoute', () => {
    const perms = PermissionRegistry.permissionsFor(AgentType.FederationAgent);
    expect(perms.has(SyscallPermission.VoiceSynthesize)).toBe(true);
    expect(perms.has(SyscallPermission.IntentRoute)).toBe(true);
  });

  it('does not have VoiceTranscribe, StateMutate, or ProcessFork', () => {
    const perms = PermissionRegistry.permissionsFor(AgentType.FederationAgent);
    expect(perms.has(SyscallPermission.VoiceTranscribe)).toBe(false);
    expect(perms.has(SyscallPermission.StateMutate)).toBe(false);
    expect(perms.has(SyscallPermission.ProcessFork)).toBe(false);
  });
});

describe('PermissionRegistry — BillingManager', () => {
  it('has StateMutate, VecInsert, VecDelete', () => {
    const perms = PermissionRegistry.permissionsFor(AgentType.BillingManager);
    expect(perms.has(SyscallPermission.StateMutate)).toBe(true);
    expect(perms.has(SyscallPermission.VecInsert)).toBe(true);
    expect(perms.has(SyscallPermission.VecDelete)).toBe(true);
  });

  it('has VoiceSynthesize and IntentRoute', () => {
    const perms = PermissionRegistry.permissionsFor(AgentType.BillingManager);
    expect(perms.has(SyscallPermission.VoiceSynthesize)).toBe(true);
    expect(perms.has(SyscallPermission.IntentRoute)).toBe(true);
  });

  it('does not have VoiceTranscribe, ProcessFork, or MeshSync', () => {
    const perms = PermissionRegistry.permissionsFor(AgentType.BillingManager);
    expect(perms.has(SyscallPermission.VoiceTranscribe)).toBe(false);
    expect(perms.has(SyscallPermission.ProcessFork)).toBe(false);
    expect(perms.has(SyscallPermission.MeshSync)).toBe(false);
  });
});

describe('PermissionRegistry — Worker', () => {
  it('has StateMutate and vector ops', () => {
    const perms = PermissionRegistry.permissionsFor(AgentType.Worker);
    expect(perms.has(SyscallPermission.StateMutate)).toBe(true);
    expect(perms.has(SyscallPermission.VecInsert)).toBe(true);
    expect(perms.has(SyscallPermission.VecSearch)).toBe(true);
    expect(perms.has(SyscallPermission.VecDelete)).toBe(true);
  });

  it('does not have ProcessFork or GraphQuery', () => {
    const perms = PermissionRegistry.permissionsFor(AgentType.Worker);
    expect(perms.has(SyscallPermission.ProcessFork)).toBe(false);
    expect(perms.has(SyscallPermission.GraphQuery)).toBe(false);
  });
});

describe('PermissionRegistry — Experimenter', () => {
  it('has broad access including fork and state mutation', () => {
    const perms = PermissionRegistry.permissionsFor(AgentType.Experimenter);
    expect(perms.has(SyscallPermission.ProcessFork)).toBe(true);
    expect(perms.has(SyscallPermission.StateMutate)).toBe(true);
    expect(perms.has(SyscallPermission.VecInsert)).toBe(true);
    expect(perms.has(SyscallPermission.VecDelete)).toBe(true);
    expect(perms.has(SyscallPermission.GraphQuery)).toBe(true);
    expect(perms.has(SyscallPermission.GraphDiffuse)).toBe(true);
    expect(perms.has(SyscallPermission.AttentionSelect)).toBe(true);
  });
});

describe('PermissionRegistry — Monitor', () => {
  it('has read-only access with HaltCheck', () => {
    const perms = PermissionRegistry.permissionsFor(AgentType.Monitor);
    expect(perms.has(SyscallPermission.VecSearch)).toBe(true);
    expect(perms.has(SyscallPermission.GraphQuery)).toBe(true);
    expect(perms.has(SyscallPermission.HaltCheck)).toBe(true);
    expect(perms.has(SyscallPermission.AttentionSelect)).toBe(true);
  });

  it('does not have write access', () => {
    const perms = PermissionRegistry.permissionsFor(AgentType.Monitor);
    expect(perms.has(SyscallPermission.VecInsert)).toBe(false);
    expect(perms.has(SyscallPermission.VecDelete)).toBe(false);
    expect(perms.has(SyscallPermission.StateMutate)).toBe(false);
    expect(perms.has(SyscallPermission.ProcessFork)).toBe(false);
  });
});

describe('PermissionRegistry — Analyst', () => {
  it('has graph operations including GraphCut', () => {
    const perms = PermissionRegistry.permissionsFor(AgentType.Analyst);
    expect(perms.has(SyscallPermission.GraphQuery)).toBe(true);
    expect(perms.has(SyscallPermission.GraphCut)).toBe(true);
    expect(perms.has(SyscallPermission.GraphDiffuse)).toBe(true);
    expect(perms.has(SyscallPermission.AttentionSelect)).toBe(true);
    expect(perms.has(SyscallPermission.StateMutate)).toBe(true);
  });
});

describe('PermissionRegistry — Reviewer', () => {
  it('has graph and attention access', () => {
    const perms = PermissionRegistry.permissionsFor(AgentType.Reviewer);
    expect(perms.has(SyscallPermission.GraphQuery)).toBe(true);
    expect(perms.has(SyscallPermission.GraphDiffuse)).toBe(true);
    expect(perms.has(SyscallPermission.AttentionSelect)).toBe(true);
    expect(perms.has(SyscallPermission.StateMutate)).toBe(true);
  });
});

describe('PermissionRegistry — Trainer', () => {
  it('has vector ops and state mutation', () => {
    const perms = PermissionRegistry.permissionsFor(AgentType.Trainer);
    expect(perms.has(SyscallPermission.VecInsert)).toBe(true);
    expect(perms.has(SyscallPermission.VecSearch)).toBe(true);
    expect(perms.has(SyscallPermission.VecDelete)).toBe(true);
    expect(perms.has(SyscallPermission.StateMutate)).toBe(true);
    expect(perms.has(SyscallPermission.AttentionSelect)).toBe(true);
  });
});

describe('PermissionRegistry — Replicator', () => {
  it('has vector ops and state mutation for sync', () => {
    const perms = PermissionRegistry.permissionsFor(AgentType.Replicator);
    expect(perms.has(SyscallPermission.VecInsert)).toBe(true);
    expect(perms.has(SyscallPermission.VecSearch)).toBe(true);
    expect(perms.has(SyscallPermission.VecDelete)).toBe(true);
    expect(perms.has(SyscallPermission.StateMutate)).toBe(true);
  });
});

// ---------------------------------------------------------------------------
// PermissionRegistry — validate() helper
// ---------------------------------------------------------------------------

describe('PermissionRegistry.validate()', () => {
  it('returns true for valid permission', () => {
    expect(PermissionRegistry.validate(AgentType.Worker, SyscallPermission.VecSearch)).toBe(true);
  });

  it('returns false for denied permission', () => {
    expect(PermissionRegistry.validate(AgentType.Worker, SyscallPermission.ProcessFork)).toBe(false);
  });

  it('returns true for All wildcard', () => {
    expect(PermissionRegistry.validate(AgentType.Worker, SyscallPermission.All)).toBe(true);
  });
});

// ---------------------------------------------------------------------------
// PermissionRegistry — spawn hierarchy
// ---------------------------------------------------------------------------

describe('PermissionRegistry — spawn hierarchy', () => {
  it('Researcher can spawn Worker, Experimenter, and Embedder', () => {
    expect(PermissionRegistry.canSpawn(AgentType.Researcher, AgentType.Worker)).toBe(true);
    expect(PermissionRegistry.canSpawn(AgentType.Researcher, AgentType.Experimenter)).toBe(true);
    expect(PermissionRegistry.canSpawn(AgentType.Researcher, AgentType.Embedder)).toBe(true);
  });

  it('Researcher cannot spawn Coordinator or Analyst', () => {
    expect(PermissionRegistry.canSpawn(AgentType.Researcher, AgentType.Coordinator)).toBe(false);
    expect(PermissionRegistry.canSpawn(AgentType.Researcher, AgentType.Analyst)).toBe(false);
  });

  it('Experimenter can spawn Workers only', () => {
    expect(PermissionRegistry.canSpawn(AgentType.Experimenter, AgentType.Worker)).toBe(true);
    expect(PermissionRegistry.canSpawn(AgentType.Experimenter, AgentType.Researcher)).toBe(false);
  });

  it('VoiceCoordinator can spawn Worker and Embedder', () => {
    expect(PermissionRegistry.canSpawn(AgentType.VoiceCoordinator, AgentType.Worker)).toBe(true);
    expect(PermissionRegistry.canSpawn(AgentType.VoiceCoordinator, AgentType.Embedder)).toBe(true);
    expect(PermissionRegistry.canSpawn(AgentType.VoiceCoordinator, AgentType.Coordinator)).toBe(false);
  });

  it('MeshCoordinator can spawn Workers only', () => {
    expect(PermissionRegistry.canSpawn(AgentType.MeshCoordinator, AgentType.Worker)).toBe(true);
    expect(PermissionRegistry.canSpawn(AgentType.MeshCoordinator, AgentType.Coordinator)).toBe(false);
  });

  it('non-forking agents cannot spawn anything', () => {
    expect(PermissionRegistry.canSpawn(AgentType.Worker, AgentType.Worker)).toBe(false);
    expect(PermissionRegistry.canSpawn(AgentType.Monitor, AgentType.Worker)).toBe(false);
    expect(PermissionRegistry.canSpawn(AgentType.Validator, AgentType.Worker)).toBe(false);
    expect(PermissionRegistry.canSpawn(AgentType.FederationAgent, AgentType.Worker)).toBe(false);
    expect(PermissionRegistry.canSpawn(AgentType.BillingManager, AgentType.Worker)).toBe(false);
  });

  it('spawnableChildren returns correct list for Coordinator', () => {
    const children = PermissionRegistry.spawnableChildren(AgentType.Coordinator);
    expect(children).toHaveLength(16); // all except Coordinator
    expect(children).not.toContain(AgentType.Coordinator);
    expect(children).toContain(AgentType.Worker);
    expect(children).toContain(AgentType.VoiceCoordinator);
    expect(children).toContain(AgentType.BillingManager);
  });

  it('spawnableChildren returns empty for Worker', () => {
    expect(PermissionRegistry.spawnableChildren(AgentType.Worker)).toHaveLength(0);
  });
});

// ---------------------------------------------------------------------------
// Full 17x17 matrix exhaustive check: permission count per agent type
// ---------------------------------------------------------------------------

describe('PermissionRegistry — permission counts per agent type', () => {
  const expectedCounts: Record<string, number> = {
    Coordinator: 17,
    Researcher: 10,
    Router: 8,
    Experimenter: 12,
    Worker: 8,
    Monitor: 8,
    Reviewer: 9,
    Trainer: 9,
    Validator: 7,
    Replicator: 8,
    Embedder: 6,
    Analyst: 10,
    VoiceCoordinator: 9,
    MarketplaceManager: 8,
    MeshCoordinator: 11,
    FederationAgent: 7,
    BillingManager: 8,
  };

  for (const t of AGENT_TYPES) {
    it(`${t} has ${expectedCounts[t]} permissions`, () => {
      const perms = PermissionRegistry.permissionsFor(t);
      expect(perms.count).toBe(expectedCounts[t]);
    });
  }
});
