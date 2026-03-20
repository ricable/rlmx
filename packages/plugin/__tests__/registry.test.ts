import { describe, it, expect, beforeEach, vi } from 'vitest';
import {
  PluginRegistry,
  PluginStatus,
  PluginAlreadyRegisteredError,
  PluginNotFoundError,
  ManifestValidationError,
  ParamType,
  actionDef,
} from '../src/index.js';
import type {
  DomainPlugin,
  PluginEvent,
  PluginManifest,
  SafetyConstraint,
  StrategyPreference,
  ActionDefinition,
  IngestAdapter,
} from '../src/index.js';

// ---------------------------------------------------------------------------
// Test helpers — mock plugins
// ---------------------------------------------------------------------------

function createMockPlugin(overrides: Partial<DomainPlugin> = {}): DomainPlugin {
  return {
    name: 'test-plugin',
    version: '0.1.0',
    description: 'A test plugin for unit tests',
    ingestAdapters: () => [],
    strategyPreferences: () => [],
    actionExtensions: () => [],
    systemPromptExtension: () => 'You are a test assistant.',
    safetyConstraints: () => [],
    ...overrides,
  };
}

function createEricssonPlugin(): DomainPlugin {
  return {
    name: 'ericsson-ran',
    version: '0.1.0',
    description:
      'Ericsson Radio Access Network plugin for RuVix. Provides PM counter ingestion.',
    ingestAdapters: () => [],
    strategyPreferences: () => [
      {
        taskPattern: 'classify.*',
        strategy: { kind: 'trm', modelName: 'rf-classifier-v2' },
        trmModel: 'rf-classifier-v2',
        rationale: 'Classification works well with TRM.',
      },
      {
        taskPattern: 'analyze.*',
        strategy: { kind: 'rlm' },
        rationale: 'Analysis needs reasoning.',
      },
      {
        taskPattern: 'monitor.*',
        strategy: {
          kind: 'hybrid',
          triage: 'rf-classifier-v2',
          threshold: 0.85,
        },
        trmModel: 'rf-classifier-v2',
        rationale: 'Monitoring uses hybrid approach.',
      },
      {
        taskPattern: 'traffic.*forecast',
        strategy: { kind: 'trm', modelName: 'lstm-traffic-predictor' },
        trmModel: 'lstm-traffic-predictor',
        rationale: 'Time-series traffic prediction.',
      },
    ],
    actionExtensions: () => [
      actionDef('OptimizeParameter')
        .description('Optimize a radio network parameter.')
        .param('cell_id', ParamType.String, true, 'Target cell')
        .param('parameter', ParamType.String, true, 'Parameter name')
        .param('value', ParamType.Float, true, 'New value')
        .build(),
      actionDef('FeatureLookup')
        .description('Look up feature documentation.')
        .param('feature_code', ParamType.String, true, 'Feature code')
        .build(),
      actionDef('ParameterValidate')
        .description('Validate parameter changes.')
        .param('changes', ParamType.StringArray, true, 'Changes list')
        .build(),
    ],
    systemPromptExtension: () =>
      'You are an Ericsson RAN domain expert. 3GPP standards.',
    safetyConstraints: () => [
      { kind: 'parameterBound', param: 'cio', min: -24, max: 24 },
      { kind: 'parameterBound', param: 'tilt', min: 0, max: 15 },
      {
        kind: 'kpiGuard',
        metric: 'accessibility',
        maxDegradation: 0.02,
      },
      {
        kind: 'humanEscalation',
        condition: 'Changes affecting more than 50 cells',
        actions: ['OptimizeParameter', 'ParameterValidate'],
      },
      {
        kind: 'rateLimit',
        action: 'OptimizeParameter',
        maxCount: 10,
        windowSecs: 3600,
      },
    ],
    embeddingConfig: () => ({
      modelName: 'ericsson-ran-embed-v1',
      dimensions: 768,
    }),
    distanceMetric: () => 'Cosine' as any,
    graphSchema: () => ({
      nodeTypes: [
        { name: 'Cell', properties: { cell_id: 'String' } },
        { name: 'Site', properties: { site_id: 'String' } },
        { name: 'Feature', properties: { feature_code: 'String' } },
      ],
      edgeTypes: [
        {
          name: 'NEIGHBORS',
          sourceType: 'Cell',
          targetType: 'Cell',
          properties: { cio: 'Float' },
        },
        {
          name: 'HOSTED_ON',
          sourceType: 'Cell',
          targetType: 'Site',
          properties: {},
        },
        {
          name: 'ACTIVATES',
          sourceType: 'Cell',
          targetType: 'Feature',
          properties: {},
        },
      ],
    }),
  };
}

function createTemplatePlugin(): DomainPlugin {
  return {
    name: 'my-domain',
    version: '0.1.0',
    description: 'A template domain plugin for RuVix',
    ingestAdapters: () => [],
    strategyPreferences: () => [
      {
        taskPattern: '.*',
        strategy: { kind: 'auto' },
        rationale: 'Default: use automatic strategy selection.',
      },
    ],
    actionExtensions: () => [
      actionDef('HelloWorld')
        .description('A simple hello-world action.')
        .param('name', ParamType.String, true, 'Name to greet')
        .build(),
    ],
    systemPromptExtension: () =>
      'You are a helpful domain assistant. Provide clear, concise answers.',
    safetyConstraints: () => [
      {
        kind: 'rateLimit',
        action: 'HelloWorld',
        maxCount: 100,
        windowSecs: 60,
      },
    ],
  };
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

describe('PluginRegistry', () => {
  let registry: PluginRegistry;

  beforeEach(() => {
    registry = new PluginRegistry();
  });

  // -----------------------------------------------------------------------
  // Registration tests
  // -----------------------------------------------------------------------

  describe('register', () => {
    it('should register a plugin successfully', () => {
      const plugin = createMockPlugin();
      registry.register(plugin);

      expect(registry.size).toBe(1);
      expect(registry.has('test-plugin')).toBe(true);
    });

    it('should register the ericsson plugin and retrieve it', () => {
      const plugin = createEricssonPlugin();
      registry.register(plugin);

      expect(registry.size).toBe(1);
      const retrieved = registry.get('ericsson-ran');
      expect(retrieved).toBeDefined();
      expect(retrieved!.name).toBe('ericsson-ran');
    });

    it('should reject duplicate plugin names', () => {
      const plugin1 = createMockPlugin();
      const plugin2 = createMockPlugin();

      registry.register(plugin1);

      expect(() => registry.register(plugin2)).toThrow(
        PluginAlreadyRegisteredError,
      );
      expect(() => registry.register(plugin2)).toThrow(
        "Plugin 'test-plugin' is already registered",
      );
    });

    it('should register multiple plugins', () => {
      registry.register(createEricssonPlugin());
      registry.register(createTemplatePlugin());

      expect(registry.size).toBe(2);
    });

    it('should accept an explicit manifest override', () => {
      const plugin = createMockPlugin();
      const manifest: PluginManifest = {
        name: 'custom-name',
        version: '1.2.3',
        description: 'Custom description',
        domains: ['telecom'],
      };

      registry.register(plugin, manifest);
      expect(registry.has('custom-name')).toBe(true);
      expect(registry.has('test-plugin')).toBe(false);
    });
  });

  // -----------------------------------------------------------------------
  // Manifest validation tests
  // -----------------------------------------------------------------------

  describe('manifest validation', () => {
    it('should reject empty name', () => {
      const plugin = createMockPlugin({ name: '' });
      expect(() => registry.register(plugin)).toThrow(
        ManifestValidationError,
      );
    });

    it('should reject name with uppercase', () => {
      const plugin = createMockPlugin({ name: 'MyPlugin' });
      expect(() => registry.register(plugin)).toThrow(
        ManifestValidationError,
      );
    });

    it('should reject name starting with number', () => {
      const plugin = createMockPlugin({ name: '1-plugin' });
      expect(() => registry.register(plugin)).toThrow(
        ManifestValidationError,
      );
    });

    it('should reject name with spaces', () => {
      const plugin = createMockPlugin({ name: 'my plugin' });
      expect(() => registry.register(plugin)).toThrow(
        ManifestValidationError,
      );
    });

    it('should reject empty version', () => {
      const plugin = createMockPlugin({ version: '' });
      expect(() => registry.register(plugin)).toThrow(
        ManifestValidationError,
      );
    });

    it('should reject non-semver version', () => {
      const plugin = createMockPlugin({ version: 'abc' });
      expect(() => registry.register(plugin)).toThrow(
        ManifestValidationError,
      );
    });

    it('should reject empty description', () => {
      const plugin = createMockPlugin({ description: '' });
      expect(() => registry.register(plugin)).toThrow(
        ManifestValidationError,
      );
    });

    it('should collect multiple violations', () => {
      const plugin = createMockPlugin({
        name: '',
        version: '',
        description: '',
      });
      try {
        registry.register(plugin);
        expect.unreachable('Should have thrown');
      } catch (err) {
        expect(err).toBeInstanceOf(ManifestValidationError);
        const validationErr = err as ManifestValidationError;
        expect(validationErr.violations.length).toBeGreaterThanOrEqual(3);
      }
    });

    it('should accept valid names with hyphens', () => {
      const plugin = createMockPlugin({ name: 'my-domain-plugin' });
      expect(() => registry.register(plugin)).not.toThrow();
    });

    it('should accept valid semver with pre-release', () => {
      const plugin = createMockPlugin({ version: '1.0.0-beta.1' });
      expect(() => registry.register(plugin)).not.toThrow();
    });
  });

  // -----------------------------------------------------------------------
  // Retrieval tests
  // -----------------------------------------------------------------------

  describe('get / getOrThrow', () => {
    it('should return undefined for nonexistent plugin', () => {
      expect(registry.get('nonexistent')).toBeUndefined();
    });

    it('should retrieve a registered plugin', () => {
      registry.register(createEricssonPlugin());
      const plugin = registry.get('ericsson-ran');
      expect(plugin).toBeDefined();
      expect(plugin!.name).toBe('ericsson-ran');
    });

    it('should throw for nonexistent plugin with getOrThrow', () => {
      expect(() => registry.getOrThrow('nonexistent')).toThrow(
        PluginNotFoundError,
      );
    });

    it('should return the plugin with getOrThrow when it exists', () => {
      registry.register(createTemplatePlugin());
      const plugin = registry.getOrThrow('my-domain');
      expect(plugin.name).toBe('my-domain');
    });
  });

  // -----------------------------------------------------------------------
  // Listing tests
  // -----------------------------------------------------------------------

  describe('list', () => {
    it('should return empty array when no plugins registered', () => {
      expect(registry.list()).toEqual([]);
    });

    it('should list all registered plugins with info', () => {
      registry.register(createEricssonPlugin());
      registry.register(createTemplatePlugin());

      const list = registry.list();
      expect(list).toHaveLength(2);

      const names = list.map((p) => p.name).sort();
      expect(names).toEqual(['ericsson-ran', 'my-domain']);
    });

    it('should include strategy and action counts', () => {
      registry.register(createEricssonPlugin());

      const list = registry.list();
      const ericsson = list.find((p) => p.name === 'ericsson-ran')!;
      expect(ericsson.strategyCount).toBe(4);
      expect(ericsson.actionCount).toBe(3);
      expect(ericsson.status).toBe(PluginStatus.Active);
    });

    it('should list plugins by domain', () => {
      const plugin = createMockPlugin({ name: 'telecom-plugin' });
      registry.register(plugin, {
        name: 'telecom-plugin',
        version: '0.1.0',
        description: 'Telecom plugin',
        domains: ['telecom', 'networking'],
      });

      registry.register(createTemplatePlugin());

      const telecomPlugins = registry.listByDomain('telecom');
      expect(telecomPlugins).toHaveLength(1);
      expect(telecomPlugins[0].name).toBe('telecom-plugin');

      const allPlugins = registry.listByDomain('nonexistent');
      expect(allPlugins).toHaveLength(0);
    });

    it('should list only active plugins', () => {
      registry.register(createEricssonPlugin());
      registry.register(createTemplatePlugin());

      registry.disable('my-domain');

      const active = registry.listActive();
      expect(active).toHaveLength(1);
      expect(active[0].name).toBe('ericsson-ran');
    });

    it('should filter by domain case-insensitively', () => {
      const plugin = createMockPlugin({ name: 'net-plugin' });
      registry.register(plugin, {
        name: 'net-plugin',
        version: '0.1.0',
        description: 'Network plugin',
        domains: ['Networking'],
      });

      expect(registry.listByDomain('networking')).toHaveLength(1);
      expect(registry.listByDomain('NETWORKING')).toHaveLength(1);
    });
  });

  // -----------------------------------------------------------------------
  // Unregistration tests
  // -----------------------------------------------------------------------

  describe('unregister', () => {
    it('should remove a registered plugin', () => {
      registry.register(createEricssonPlugin());
      expect(registry.size).toBe(1);

      const removed = registry.unregister('ericsson-ran');
      expect(removed).toBeDefined();
      expect(removed!.name).toBe('ericsson-ran');
      expect(registry.size).toBe(0);
      expect(registry.get('ericsson-ran')).toBeUndefined();
    });

    it('should return undefined for nonexistent plugin', () => {
      expect(registry.unregister('nonexistent')).toBeUndefined();
    });

    it('should allow re-registering after unregister', () => {
      const plugin = createEricssonPlugin();
      registry.register(plugin);
      registry.unregister('ericsson-ran');

      expect(() => registry.register(createEricssonPlugin())).not.toThrow();
      expect(registry.size).toBe(1);
    });
  });

  // -----------------------------------------------------------------------
  // Enable/disable tests
  // -----------------------------------------------------------------------

  describe('enable / disable', () => {
    it('should disable an active plugin', () => {
      registry.register(createMockPlugin());
      registry.disable('test-plugin');
      expect(registry.getStatus('test-plugin')).toBe(PluginStatus.Disabled);
    });

    it('should re-enable a disabled plugin', () => {
      registry.register(createMockPlugin());
      registry.disable('test-plugin');
      registry.enable('test-plugin');
      expect(registry.getStatus('test-plugin')).toBe(PluginStatus.Active);
    });

    it('should throw when disabling nonexistent plugin', () => {
      expect(() => registry.disable('nonexistent')).toThrow(
        PluginNotFoundError,
      );
    });

    it('should throw when enabling nonexistent plugin', () => {
      expect(() => registry.enable('nonexistent')).toThrow(
        PluginNotFoundError,
      );
    });

    it('should throw when getting status of nonexistent plugin', () => {
      expect(() => registry.getStatus('nonexistent')).toThrow(
        PluginNotFoundError,
      );
    });

    it('should default to Active status on registration', () => {
      registry.register(createMockPlugin());
      expect(registry.getStatus('test-plugin')).toBe(PluginStatus.Active);
    });
  });

  // -----------------------------------------------------------------------
  // Size / isEmpty tests
  // -----------------------------------------------------------------------

  describe('size / isEmpty', () => {
    it('should report empty initially', () => {
      expect(registry.size).toBe(0);
      expect(registry.isEmpty).toBe(true);
    });

    it('should update size after registrations', () => {
      registry.register(createEricssonPlugin());
      expect(registry.size).toBe(1);
      expect(registry.isEmpty).toBe(false);

      registry.register(createTemplatePlugin());
      expect(registry.size).toBe(2);
    });

    it('should update size after unregister', () => {
      registry.register(createMockPlugin());
      registry.unregister('test-plugin');
      expect(registry.size).toBe(0);
      expect(registry.isEmpty).toBe(true);
    });
  });

  // -----------------------------------------------------------------------
  // Event emission tests
  // -----------------------------------------------------------------------

  describe('events', () => {
    it('should emit registered event', () => {
      const events: PluginEvent[] = [];
      registry.on((e) => events.push(e));

      registry.register(createMockPlugin());

      expect(events).toHaveLength(1);
      expect(events[0]).toEqual({
        type: 'registered',
        pluginName: 'test-plugin',
        version: '0.1.0',
      });
    });

    it('should emit unregistered event', () => {
      const events: PluginEvent[] = [];
      registry.register(createMockPlugin());

      registry.on((e) => events.push(e));
      registry.unregister('test-plugin');

      expect(events).toHaveLength(1);
      expect(events[0]).toEqual({
        type: 'unregistered',
        pluginName: 'test-plugin',
      });
    });

    it('should emit disabled/enabled events', () => {
      const events: PluginEvent[] = [];
      registry.register(createMockPlugin());
      registry.on((e) => events.push(e));

      registry.disable('test-plugin');
      registry.enable('test-plugin');

      expect(events).toHaveLength(2);
      expect(events[0]).toEqual({
        type: 'disabled',
        pluginName: 'test-plugin',
      });
      expect(events[1]).toEqual({
        type: 'enabled',
        pluginName: 'test-plugin',
      });
    });

    it('should support unsubscribe', () => {
      const events: PluginEvent[] = [];
      const unsub = registry.on((e) => events.push(e));

      registry.register(createMockPlugin());
      expect(events).toHaveLength(1);

      unsub();
      registry.register(createTemplatePlugin());
      expect(events).toHaveLength(1); // No new events after unsub
    });

    it('should support multiple listeners', () => {
      const events1: PluginEvent[] = [];
      const events2: PluginEvent[] = [];
      registry.on((e) => events1.push(e));
      registry.on((e) => events2.push(e));

      registry.register(createMockPlugin());

      expect(events1).toHaveLength(1);
      expect(events2).toHaveLength(1);
    });

    it('should swallow listener errors without breaking registry', () => {
      registry.on(() => {
        throw new Error('listener crashed');
      });
      const events: PluginEvent[] = [];
      registry.on((e) => events.push(e));

      // Should not throw even though first listener throws
      expect(() => registry.register(createMockPlugin())).not.toThrow();
      expect(events).toHaveLength(1);
    });

    it('should remove all listeners', () => {
      const events: PluginEvent[] = [];
      registry.on((e) => events.push(e));
      registry.removeAllListeners();

      registry.register(createMockPlugin());
      expect(events).toHaveLength(0);
    });

    it('should not emit unregistered for nonexistent plugin', () => {
      const events: PluginEvent[] = [];
      registry.on((e) => events.push(e));

      registry.unregister('nonexistent');
      expect(events).toHaveLength(0);
    });
  });

  // -----------------------------------------------------------------------
  // Safety engine tests
  // -----------------------------------------------------------------------

  describe('checkSafety', () => {
    it('should allow valid parameter values', () => {
      const constraints: SafetyConstraint[] = [
        { kind: 'parameterBound', param: 'cio', min: -24, max: 24 },
      ];

      const result = registry.checkSafety(
        'OptimizeParameter',
        { cio: 5 },
        constraints,
      );
      expect(result.status).toBe('allowed');
    });

    it('should reject out-of-bounds parameter values', () => {
      const constraints: SafetyConstraint[] = [
        { kind: 'parameterBound', param: 'cio', min: -24, max: 24 },
      ];

      const result = registry.checkSafety(
        'OptimizeParameter',
        { cio: 30 },
        constraints,
      );
      expect(result.status).toBe('rejected');
      if (result.status === 'rejected') {
        expect(result.reason).toContain('cio');
        expect(result.reason).toContain('30');
      }
    });

    it('should reject missing required parameters', () => {
      const constraints: SafetyConstraint[] = [
        { kind: 'parameterBound', param: 'cio', min: -24, max: 24 },
      ];

      const result = registry.checkSafety(
        'OptimizeParameter',
        {},
        constraints,
      );
      expect(result.status).toBe('rejected');
      if (result.status === 'rejected') {
        expect(result.reason).toContain('cio');
        expect(result.reason).toContain('missing');
      }
    });

    it('should reject non-numeric parameter values', () => {
      const constraints: SafetyConstraint[] = [
        { kind: 'parameterBound', param: 'cio', min: -24, max: 24 },
      ];

      const result = registry.checkSafety(
        'OptimizeParameter',
        { cio: 'abc' },
        constraints,
      );
      expect(result.status).toBe('rejected');
      if (result.status === 'rejected') {
        expect(result.reason).toContain('non-numeric');
      }
    });

    it('should enforce rate limits', () => {
      const constraints: SafetyConstraint[] = [
        {
          kind: 'rateLimit',
          action: 'TestAction',
          maxCount: 3,
          windowSecs: 3600,
        },
      ];

      // First 3 calls should be allowed
      for (let i = 0; i < 3; i++) {
        const result = registry.checkSafety('TestAction', {}, constraints);
        expect(result.status).toBe('allowed');
      }

      // 4th call should be rate limited
      const result = registry.checkSafety('TestAction', {}, constraints);
      expect(result.status).toBe('rejected');
      if (result.status === 'rejected') {
        expect(result.reason).toContain('Rate limit');
        expect(result.reason).toContain('TestAction');
      }
    });

    it('should not apply rate limit to different actions', () => {
      const constraints: SafetyConstraint[] = [
        {
          kind: 'rateLimit',
          action: 'ActionA',
          maxCount: 1,
          windowSecs: 3600,
        },
      ];

      // Different action should not be rate limited
      const result = registry.checkSafety('ActionB', {}, constraints);
      expect(result.status).toBe('allowed');
    });

    it('should require approval for human escalation', () => {
      const constraints: SafetyConstraint[] = [
        {
          kind: 'humanEscalation',
          condition: 'Changes affecting more than 50 cells',
          actions: ['OptimizeParameter'],
        },
      ];

      const result = registry.checkSafety(
        'OptimizeParameter',
        {},
        constraints,
      );
      expect(result.status).toBe('requiresApproval');
      if (result.status === 'requiresApproval') {
        expect(result.reason).toContain('Human approval');
      }
    });

    it('should not require approval for unlisted actions', () => {
      const constraints: SafetyConstraint[] = [
        {
          kind: 'humanEscalation',
          condition: 'Changes affecting more than 50 cells',
          actions: ['OptimizeParameter'],
        },
      ];

      const result = registry.checkSafety('FeatureLookup', {}, constraints);
      expect(result.status).toBe('allowed');
    });

    it('should allow kpiGuard (placeholder always allows)', () => {
      const constraints: SafetyConstraint[] = [
        { kind: 'kpiGuard', metric: 'accessibility', maxDegradation: 0.02 },
      ];

      const result = registry.checkSafety(
        'OptimizeParameter',
        {},
        constraints,
      );
      expect(result.status).toBe('allowed');
    });

    it('should return most restrictive result across constraints', () => {
      const constraints: SafetyConstraint[] = [
        { kind: 'parameterBound', param: 'cio', min: -24, max: 24 },
        {
          kind: 'humanEscalation',
          condition: 'Large change set',
          actions: ['OptimizeParameter'],
        },
      ];

      // Both constraints apply: rejection is more restrictive than approval
      const result = registry.checkSafety(
        'OptimizeParameter',
        { cio: 30 },
        constraints,
      );
      expect(result.status).toBe('rejected');
    });

    it('should prefer requiresApproval over allowed', () => {
      const constraints: SafetyConstraint[] = [
        { kind: 'parameterBound', param: 'cio', min: -24, max: 24 },
        {
          kind: 'humanEscalation',
          condition: 'Large change set',
          actions: ['OptimizeParameter'],
        },
      ];

      // Valid param but human escalation triggers
      const result = registry.checkSafety(
        'OptimizeParameter',
        { cio: 5 },
        constraints,
      );
      expect(result.status).toBe('requiresApproval');
    });
  });

  // -----------------------------------------------------------------------
  // Ericsson plugin metadata tests (matching Rust test suite)
  // -----------------------------------------------------------------------

  describe('ericsson plugin metadata', () => {
    it('should have correct name and version', () => {
      const plugin = createEricssonPlugin();
      expect(plugin.name).toBe('ericsson-ran');
      expect(plugin.version).toBe('0.1.0');
      expect(plugin.description).toContain('Ericsson');
    });

    it('should have 4 strategy preferences', () => {
      const plugin = createEricssonPlugin();
      const strategies = plugin.strategyPreferences();
      expect(strategies).toHaveLength(4);
    });

    it('should have 3 action extensions', () => {
      const plugin = createEricssonPlugin();
      const actions = plugin.actionExtensions();
      expect(actions).toHaveLength(3);
      const names = actions.map((a) => a.name);
      expect(names).toContain('OptimizeParameter');
      expect(names).toContain('FeatureLookup');
      expect(names).toContain('ParameterValidate');
    });

    it('should have 5 safety constraints', () => {
      const plugin = createEricssonPlugin();
      const constraints = plugin.safetyConstraints();
      expect(constraints).toHaveLength(5);
    });

    it('should have embedding config with 768 dimensions', () => {
      const plugin = createEricssonPlugin();
      const config = plugin.embeddingConfig?.();
      expect(config).toBeDefined();
      expect(config!.dimensions).toBe(768);
    });

    it('should have graph schema with 3 node types and 3 edge types', () => {
      const plugin = createEricssonPlugin();
      const schema = plugin.graphSchema?.();
      expect(schema).toBeDefined();
      expect(schema!.nodeTypes).toHaveLength(3);
      expect(schema!.edgeTypes).toHaveLength(3);
    });

    it('should have system prompt mentioning 3GPP', () => {
      const plugin = createEricssonPlugin();
      const prompt = plugin.systemPromptExtension();
      expect(prompt).toContain('Ericsson');
      expect(prompt).toContain('3GPP');
    });
  });

  // -----------------------------------------------------------------------
  // Template plugin tests (matching Rust test suite)
  // -----------------------------------------------------------------------

  describe('template plugin', () => {
    it('should load with correct metadata', () => {
      const plugin = createTemplatePlugin();
      expect(plugin.name).toBe('my-domain');
      expect(plugin.version).toBe('0.1.0');
      expect(plugin.description).not.toBe('');
    });

    it('should have 1 strategy, 1 action, 1 constraint', () => {
      const plugin = createTemplatePlugin();
      expect(plugin.strategyPreferences()).toHaveLength(1);
      expect(plugin.actionExtensions()).toHaveLength(1);
      expect(plugin.safetyConstraints()).toHaveLength(1);
    });

    it('should have non-empty system prompt', () => {
      const plugin = createTemplatePlugin();
      expect(plugin.systemPromptExtension()).not.toBe('');
    });

    it('should register in a registry', () => {
      registry.register(createTemplatePlugin());
      expect(registry.get('my-domain')).toBeDefined();
    });
  });

  // -----------------------------------------------------------------------
  // ActionDefinitionBuilder tests
  // -----------------------------------------------------------------------

  describe('ActionDefinitionBuilder', () => {
    it('should build an action definition with builder pattern', () => {
      const action = actionDef('TestAction')
        .description('A test action')
        .param('name', ParamType.String, true, 'The name')
        .param('value', ParamType.Float, false, 'The value')
        .param('tags', ParamType.StringArray, false, 'Tags')
        .build();

      expect(action.name).toBe('TestAction');
      expect(action.description).toBe('A test action');
      expect(action.params).toHaveLength(3);
      expect(action.params[0].name).toBe('name');
      expect(action.params[0].paramType).toBe(ParamType.String);
      expect(action.params[0].required).toBe(true);
      expect(action.params[1].paramType).toBe(ParamType.Float);
      expect(action.params[1].required).toBe(false);
      expect(action.params[2].paramType).toBe(ParamType.StringArray);
    });
  });

  // -----------------------------------------------------------------------
  // Strategy preferences serialization tests
  // -----------------------------------------------------------------------

  describe('strategy preferences', () => {
    it('should support all strategy kinds', () => {
      const prefs: StrategyPreference[] = [
        {
          taskPattern: 'classify.*',
          strategy: { kind: 'trm', modelName: 'model-a' },
          trmModel: 'model-a',
          rationale: 'Classification works well with TRM.',
        },
        {
          taskPattern: 'analyze.*',
          strategy: { kind: 'rlm' },
          rationale: 'Analysis needs reasoning.',
        },
        {
          taskPattern: 'monitor.*',
          strategy: {
            kind: 'hybrid',
            triage: 'model-a',
            threshold: 0.8,
          },
          trmModel: 'model-a',
          rationale: 'Monitoring uses hybrid approach.',
        },
        {
          taskPattern: 'default.*',
          strategy: { kind: 'auto' },
          rationale: 'Auto-select strategy.',
        },
      ];

      // Verify round-trip via JSON
      const json = JSON.stringify(prefs);
      const deserialized: StrategyPreference[] = JSON.parse(json);

      expect(deserialized).toHaveLength(4);
      expect(deserialized[0].taskPattern).toBe('classify.*');
      expect(deserialized[0].strategy).toEqual({
        kind: 'trm',
        modelName: 'model-a',
      });
      expect(deserialized[2].strategy).toEqual({
        kind: 'hybrid',
        triage: 'model-a',
        threshold: 0.8,
      });
    });
  });

  // -----------------------------------------------------------------------
  // Manifest retrieval test
  // -----------------------------------------------------------------------

  describe('getManifest', () => {
    it('should return the manifest for a registered plugin', () => {
      const plugin = createMockPlugin({ name: 'my-plugin' });
      const manifest: PluginManifest = {
        name: 'my-plugin',
        version: '1.0.0',
        description: 'My cool plugin',
        domains: ['telecom'],
        author: 'Cedric',
      };

      registry.register(plugin, manifest);
      const retrieved = registry.getManifest('my-plugin');

      expect(retrieved.name).toBe('my-plugin');
      expect(retrieved.version).toBe('1.0.0');
      expect(retrieved.domains).toEqual(['telecom']);
      expect(retrieved.author).toBe('Cedric');
    });

    it('should return a copy (not a reference)', () => {
      registry.register(createMockPlugin());
      const m1 = registry.getManifest('test-plugin');
      const m2 = registry.getManifest('test-plugin');
      expect(m1).toEqual(m2);
      expect(m1).not.toBe(m2);
    });

    it('should throw for nonexistent plugin', () => {
      expect(() => registry.getManifest('nonexistent')).toThrow(
        PluginNotFoundError,
      );
    });
  });

  // -----------------------------------------------------------------------
  // Full integration test
  // -----------------------------------------------------------------------

  describe('integration', () => {
    it('should support full CRUD lifecycle with events', () => {
      const events: PluginEvent[] = [];
      registry.on((e) => events.push(e));

      // Create
      registry.register(createEricssonPlugin());
      registry.register(createTemplatePlugin());
      expect(registry.size).toBe(2);

      // Read
      const list = registry.list();
      expect(list).toHaveLength(2);
      expect(registry.get('ericsson-ran')).toBeDefined();
      expect(registry.get('my-domain')).toBeDefined();

      // Update (disable/enable)
      registry.disable('my-domain');
      expect(registry.getStatus('my-domain')).toBe(PluginStatus.Disabled);
      expect(registry.listActive()).toHaveLength(1);

      registry.enable('my-domain');
      expect(registry.getStatus('my-domain')).toBe(PluginStatus.Active);
      expect(registry.listActive()).toHaveLength(2);

      // Delete
      const removed = registry.unregister('ericsson-ran');
      expect(removed).toBeDefined();
      expect(registry.size).toBe(1);
      expect(registry.get('ericsson-ran')).toBeUndefined();

      // Verify events
      expect(events).toHaveLength(5); // 2 registered + disabled + enabled + unregistered
      expect(events[0].type).toBe('registered');
      expect(events[1].type).toBe('registered');
      expect(events[2].type).toBe('disabled');
      expect(events[3].type).toBe('enabled');
      expect(events[4].type).toBe('unregistered');
    });

    it('should run safety checks against plugin constraints', () => {
      registry.register(createEricssonPlugin());
      const plugin = registry.getOrThrow('ericsson-ran');
      const constraints = plugin.safetyConstraints();

      // Valid CIO and tilt values, action not in escalation or rate limit
      const allowed = registry.checkSafety(
        'FeatureLookup',
        { cio: 5, tilt: 3 },
        constraints,
      );
      // FeatureLookup is not in humanEscalation list, and not in rateLimit action
      expect(allowed.status).toBe('allowed');

      // OptimizeParameter triggers humanEscalation
      const approval = registry.checkSafety(
        'OptimizeParameter',
        { cio: 5, tilt: 3 },
        constraints,
      );
      expect(approval.status).toBe('requiresApproval');

      // Out of bounds triggers rejection (most restrictive wins)
      const rejected = registry.checkSafety(
        'OptimizeParameter',
        { cio: 30 },
        constraints,
      );
      expect(rejected.status).toBe('rejected');
    });
  });
});
