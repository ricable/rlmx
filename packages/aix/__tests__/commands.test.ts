// ---------------------------------------------------------------------------
// aix — CLI command tests
//
// Validates that all 14 command groups are registered and the program
// structure matches the Rust CLI.
// ---------------------------------------------------------------------------

import { describe, it, expect } from 'vitest';
import { createProgram } from '../src/index.js';

describe('aix CLI', () => {
  const program = createProgram();

  it('should have name "aix"', () => {
    expect(program.name()).toBe('aix');
  });

  it('should have version "0.1.0"', () => {
    expect(program.version()).toBe('0.1.0');
  });

  it('should register all 14 command groups', () => {
    const commandNames = program.commands.map((c) => c.name());
    expect(commandNames).toContain('serve');
    expect(commandNames).toContain('query');
    expect(commandNames).toContain('voice');
    expect(commandNames).toContain('agent');
    expect(commandNames).toContain('swarm');
    expect(commandNames).toContain('marketplace');
    expect(commandNames).toContain('mesh');
    expect(commandNames).toContain('billing');
    expect(commandNames).toContain('federation');
    expect(commandNames).toContain('engagement');
    expect(commandNames).toContain('phone');
    expect(commandNames).toContain('research');
    expect(commandNames).toContain('sandbox');
    expect(commandNames).toContain('edge');
    expect(commandNames).toHaveLength(14);
  });

  describe('voice subcommands', () => {
    it('should have start, transcribe, intents, session', () => {
      const voice = program.commands.find((c) => c.name() === 'voice')!;
      const subNames = voice.commands.map((c) => c.name());
      expect(subNames).toContain('start');
      expect(subNames).toContain('transcribe');
      expect(subNames).toContain('intents');
      expect(subNames).toContain('session');
    });
  });

  describe('agent subcommands', () => {
    it('should have spawn, list, kill', () => {
      const agent = program.commands.find((c) => c.name() === 'agent')!;
      const subNames = agent.commands.map((c) => c.name());
      expect(subNames).toContain('spawn');
      expect(subNames).toContain('list');
      expect(subNames).toContain('kill');
    });
  });

  describe('swarm subcommands', () => {
    it('should have start, status, topology', () => {
      const swarm = program.commands.find((c) => c.name() === 'swarm')!;
      const subNames = swarm.commands.map((c) => c.name());
      expect(subNames).toContain('start');
      expect(subNames).toContain('status');
      expect(subNames).toContain('topology');
    });
  });

  describe('marketplace subcommands', () => {
    it('should have search, install, list, featured, publish', () => {
      const mp = program.commands.find((c) => c.name() === 'marketplace')!;
      const subNames = mp.commands.map((c) => c.name());
      expect(subNames).toContain('search');
      expect(subNames).toContain('install');
      expect(subNames).toContain('list');
      expect(subNames).toContain('featured');
      expect(subNames).toContain('publish');
    });
  });

  describe('mesh subcommands', () => {
    it('should have status, devices, add-device, sync, fleet, failover', () => {
      const mesh = program.commands.find((c) => c.name() === 'mesh')!;
      const subNames = mesh.commands.map((c) => c.name());
      expect(subNames).toContain('status');
      expect(subNames).toContain('devices');
      expect(subNames).toContain('add-device');
      expect(subNames).toContain('sync');
      expect(subNames).toContain('fleet');
      expect(subNames).toContain('failover');
    });
  });

  describe('billing subcommands', () => {
    it('should have status, upgrade, usage, family, developer', () => {
      const billing = program.commands.find((c) => c.name() === 'billing')!;
      const subNames = billing.commands.map((c) => c.name());
      expect(subNames).toContain('status');
      expect(subNames).toContain('upgrade');
      expect(subNames).toContain('usage');
      expect(subNames).toContain('family');
      expect(subNames).toContain('developer');
    });
  });

  describe('federation subcommands', () => {
    it('should have status, contribute, bootstrap', () => {
      const fed = program.commands.find((c) => c.name() === 'federation')!;
      const subNames = fed.commands.map((c) => c.name());
      expect(subNames).toContain('status');
      expect(subNames).toContain('contribute');
      expect(subNames).toContain('bootstrap');
    });
  });

  describe('engagement subcommands', () => {
    it('should have score, savings, streak, achievements', () => {
      const eng = program.commands.find((c) => c.name() === 'engagement')!;
      const subNames = eng.commands.map((c) => c.name());
      expect(subNames).toContain('score');
      expect(subNames).toContain('savings');
      expect(subNames).toContain('streak');
      expect(subNames).toContain('achievements');
    });
  });

  describe('phone subcommands', () => {
    it('should have status, agents, battery', () => {
      const phone = program.commands.find((c) => c.name() === 'phone')!;
      const subNames = phone.commands.map((c) => c.name());
      expect(subNames).toContain('status');
      expect(subNames).toContain('agents');
      expect(subNames).toContain('battery');
    });
  });

  describe('research subcommands', () => {
    it('should have start, status, list', () => {
      const research = program.commands.find((c) => c.name() === 'research')!;
      const subNames = research.commands.map((c) => c.name());
      expect(subNames).toContain('start');
      expect(subNames).toContain('status');
      expect(subNames).toContain('list');
    });
  });

  describe('sandbox subcommands', () => {
    it('should have spawn, terminate, status, list', () => {
      const sandbox = program.commands.find((c) => c.name() === 'sandbox')!;
      const subNames = sandbox.commands.map((c) => c.name());
      expect(subNames).toContain('spawn');
      expect(subNames).toContain('terminate');
      expect(subNames).toContain('status');
      expect(subNames).toContain('list');
    });
  });

  describe('edge subcommands', () => {
    it('should have status, models', () => {
      const edge = program.commands.find((c) => c.name() === 'edge')!;
      const subNames = edge.commands.map((c) => c.name());
      expect(subNames).toContain('status');
      expect(subNames).toContain('models');
    });
  });
});
