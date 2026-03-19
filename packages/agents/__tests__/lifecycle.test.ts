import { describe, it, expect } from 'vitest';
import { AgentLifecycle } from '../src/lifecycle.js';
import { InvalidTransitionError } from '../src/errors.js';
import type { AgentStatus } from '../src/types.js';

// ---------------------------------------------------------------------------
// Static method tests
// ---------------------------------------------------------------------------

describe('AgentLifecycle — valid forward flow', () => {
  it('Pending -> Starting', () => {
    expect(AgentLifecycle.isValidTransition('Pending', 'Starting')).toBe(true);
  });
  it('Starting -> Running', () => {
    expect(AgentLifecycle.isValidTransition('Starting', 'Running')).toBe(true);
  });
  it('Running -> Stopping', () => {
    expect(AgentLifecycle.isValidTransition('Running', 'Stopping')).toBe(true);
  });
  it('Stopping -> Terminated', () => {
    expect(AgentLifecycle.isValidTransition('Stopping', 'Terminated')).toBe(true);
  });
});

describe('AgentLifecycle — pause/resume', () => {
  it('Running -> Paused', () => {
    expect(AgentLifecycle.isValidTransition('Running', 'Paused')).toBe(true);
  });
  it('Paused -> Running (resume)', () => {
    expect(AgentLifecycle.isValidTransition('Paused', 'Running')).toBe(true);
  });
  it('Paused -> Stopping', () => {
    expect(AgentLifecycle.isValidTransition('Paused', 'Stopping')).toBe(true);
  });
});

describe('AgentLifecycle — failure paths', () => {
  it('Starting -> Failed', () => {
    expect(AgentLifecycle.isValidTransition('Starting', 'Failed')).toBe(true);
  });
  it('Running -> Failed', () => {
    expect(AgentLifecycle.isValidTransition('Running', 'Failed')).toBe(true);
  });
  it('Pending -> Failed is NOT valid', () => {
    expect(AgentLifecycle.isValidTransition('Pending', 'Failed')).toBe(false);
  });
  it('Paused -> Failed is NOT valid', () => {
    expect(AgentLifecycle.isValidTransition('Paused', 'Failed')).toBe(false);
  });
});

describe('AgentLifecycle — invalid transitions', () => {
  it('cannot skip states (Pending -> Running)', () => {
    expect(AgentLifecycle.isValidTransition('Pending', 'Running')).toBe(false);
  });
  it('cannot go backwards (Running -> Starting)', () => {
    expect(AgentLifecycle.isValidTransition('Running', 'Starting')).toBe(false);
  });
  it('cannot resurrect from Terminated', () => {
    expect(AgentLifecycle.isValidTransition('Terminated', 'Running')).toBe(false);
  });
  it('cannot resurrect from Failed', () => {
    expect(AgentLifecycle.isValidTransition('Failed', 'Running')).toBe(false);
  });
  it('cannot go from Pending to Terminated', () => {
    expect(AgentLifecycle.isValidTransition('Pending', 'Terminated')).toBe(false);
  });
  it('cannot go from Stopping to Running', () => {
    expect(AgentLifecycle.isValidTransition('Stopping', 'Running')).toBe(false);
  });
});

describe('AgentLifecycle.transition()', () => {
  it('returns next state on valid transition', () => {
    expect(AgentLifecycle.transition('Pending', 'Starting')).toBe('Starting');
  });

  it('throws InvalidTransitionError on invalid transition', () => {
    expect(() => AgentLifecycle.transition('Terminated', 'Running')).toThrow(
      InvalidTransitionError,
    );
  });
});

describe('AgentLifecycle.validNextStates()', () => {
  it('Running has 3 valid next states', () => {
    const next = AgentLifecycle.validNextStates('Running');
    expect(next).toHaveLength(3);
    expect(next).toContain('Paused');
    expect(next).toContain('Stopping');
    expect(next).toContain('Failed');
  });

  it('Pending has 1 valid next state', () => {
    const next = AgentLifecycle.validNextStates('Pending');
    expect(next).toHaveLength(1);
    expect(next).toContain('Starting');
  });

  it('Starting has 2 valid next states', () => {
    const next = AgentLifecycle.validNextStates('Starting');
    expect(next).toHaveLength(2);
    expect(next).toContain('Running');
    expect(next).toContain('Failed');
  });

  it('Paused has 2 valid next states', () => {
    const next = AgentLifecycle.validNextStates('Paused');
    expect(next).toHaveLength(2);
    expect(next).toContain('Running');
    expect(next).toContain('Stopping');
  });

  it('Stopping has 1 valid next state', () => {
    const next = AgentLifecycle.validNextStates('Stopping');
    expect(next).toHaveLength(1);
    expect(next).toContain('Terminated');
  });
});

describe('AgentLifecycle.isTerminal()', () => {
  it('Terminated is terminal', () => {
    expect(AgentLifecycle.isTerminal('Terminated')).toBe(true);
  });
  it('Failed is terminal', () => {
    expect(AgentLifecycle.isTerminal('Failed')).toBe(true);
  });
  it('Running is not terminal', () => {
    expect(AgentLifecycle.isTerminal('Running')).toBe(false);
  });
  it('Pending is not terminal', () => {
    expect(AgentLifecycle.isTerminal('Pending')).toBe(false);
  });
});

describe('AgentLifecycle — terminal states have no next', () => {
  it('Terminated has no valid next states', () => {
    expect(AgentLifecycle.validNextStates('Terminated')).toHaveLength(0);
  });
  it('Failed has no valid next states', () => {
    expect(AgentLifecycle.validNextStates('Failed')).toHaveLength(0);
  });
});

// ---------------------------------------------------------------------------
// Instance method tests
// ---------------------------------------------------------------------------

describe('AgentLifecycle instance', () => {
  it('starts in Pending by default', () => {
    const lc = new AgentLifecycle();
    expect(lc.status).toBe('Pending');
  });

  it('can be initialized with a custom state', () => {
    const lc = new AgentLifecycle('Running');
    expect(lc.status).toBe('Running');
  });

  it('transitionTo mutates internal state', () => {
    const lc = new AgentLifecycle();
    lc.transitionTo('Starting');
    expect(lc.status).toBe('Starting');
    lc.transitionTo('Running');
    expect(lc.status).toBe('Running');
  });

  it('transitionTo throws on invalid transition', () => {
    const lc = new AgentLifecycle();
    expect(() => lc.transitionTo('Running')).toThrow(InvalidTransitionError);
    expect(lc.status).toBe('Pending'); // unchanged
  });

  it('full lifecycle: Pending -> Starting -> Running -> Stopping -> Terminated', () => {
    const lc = new AgentLifecycle();
    lc.transitionTo('Starting');
    lc.transitionTo('Running');
    lc.transitionTo('Stopping');
    lc.transitionTo('Terminated');
    expect(lc.status).toBe('Terminated');
    expect(lc.isTerminal).toBe(true);
  });

  it('isTerminal reflects current state', () => {
    const lc = new AgentLifecycle('Running');
    expect(lc.isTerminal).toBe(false);
    lc.transitionTo('Failed');
    expect(lc.isTerminal).toBe(true);
  });

  it('validNextStates reflects current state', () => {
    const lc = new AgentLifecycle('Running');
    expect(lc.validNextStates).toContain('Paused');
    expect(lc.validNextStates).toContain('Stopping');
    expect(lc.validNextStates).toContain('Failed');
  });

  it('pause and resume cycle', () => {
    const lc = new AgentLifecycle('Running');
    lc.transitionTo('Paused');
    expect(lc.status).toBe('Paused');
    lc.transitionTo('Running');
    expect(lc.status).toBe('Running');
  });
});
