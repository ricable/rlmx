import { describe, it, expect, vi } from 'vitest';
import { DiscoveryBridge } from '../src/bridge.js';
import type { DiscoveredNode } from '../src/bridge.js';

function makeSeedNode(id = 'seed-1'): DiscoveredNode {
  return {
    id,
    name: 'test-seed',
    host: '192.168.1.100',
    port: 5353,
    serviceType: '_cognitum._tcp',
    origin: 'seed',
    txtRecords: { sensor_profile: 'temperature', version: '0.2.0', agent_name: 'TempSensor' },
    discoveredAt: new Date().toISOString(),
  };
}

describe('DiscoveryBridge', () => {
  it('start() sets isRunning to true', async () => {
    const bridge = new DiscoveryBridge();
    expect(bridge.isRunning).toBe(false);
    await bridge.start();
    expect(bridge.isRunning).toBe(true);
  });

  it('stop() sets isRunning to false and clears nodes', async () => {
    const bridge = new DiscoveryBridge();
    await bridge.start();
    bridge.addNode(makeSeedNode());
    expect(bridge.getNodes()).toHaveLength(1);

    await bridge.stop();
    expect(bridge.isRunning).toBe(false);
    expect(bridge.getNodes()).toHaveLength(0);
  });

  it('addNode adds to getNodes()', () => {
    const bridge = new DiscoveryBridge();
    const node = makeSeedNode();
    bridge.addNode(node);
    expect(bridge.getNodes()).toHaveLength(1);
    expect(bridge.getNodes()[0].id).toBe('seed-1');
  });

  it('addNode emits "node-discovered" event', () => {
    const bridge = new DiscoveryBridge();
    const handler = vi.fn();
    bridge.on('node-discovered', handler);

    const node = makeSeedNode();
    bridge.addNode(node);

    expect(handler).toHaveBeenCalledTimes(1);
    expect(handler).toHaveBeenCalledWith(node);
  });

  it('addNode for existing node does not re-emit event', () => {
    const bridge = new DiscoveryBridge();
    const handler = vi.fn();
    bridge.on('node-discovered', handler);

    const node = makeSeedNode();
    bridge.addNode(node);
    bridge.addNode(node); // same id again

    expect(handler).toHaveBeenCalledTimes(1);
  });

  it('removeNode removes from getNodes()', () => {
    const bridge = new DiscoveryBridge();
    bridge.addNode(makeSeedNode('n1'));
    bridge.addNode(makeSeedNode('n2'));
    expect(bridge.getNodes()).toHaveLength(2);

    bridge.removeNode('n1');
    expect(bridge.getNodes()).toHaveLength(1);
    expect(bridge.getNodes()[0].id).toBe('n2');
  });

  it('removeNode emits "node-lost" event', () => {
    const bridge = new DiscoveryBridge();
    const handler = vi.fn();
    bridge.on('node-lost', handler);

    const node = makeSeedNode();
    bridge.addNode(node);
    bridge.removeNode('seed-1');

    expect(handler).toHaveBeenCalledTimes(1);
    expect(handler).toHaveBeenCalledWith(node);
  });

  it('removeNode for unknown id does not emit event', () => {
    const bridge = new DiscoveryBridge();
    const handler = vi.fn();
    bridge.on('node-lost', handler);

    bridge.removeNode('nonexistent');
    expect(handler).not.toHaveBeenCalled();
  });

  it('getNodesByOrigin filters correctly', () => {
    const bridge = new DiscoveryBridge();
    const seedNode = makeSeedNode('s1');
    const rlmxNode: DiscoveredNode = { ...makeSeedNode('r1'), origin: 'rlmx' };
    bridge.addNode(seedNode);
    bridge.addNode(rlmxNode);

    expect(bridge.getNodesByOrigin('seed')).toHaveLength(1);
    expect(bridge.getNodesByOrigin('seed')[0].id).toBe('s1');
    expect(bridge.getNodesByOrigin('rlmx')).toHaveLength(1);
    expect(bridge.getNodesByOrigin('custom')).toHaveLength(0);
  });

  it('resolveServiceType resolves "_cognitum._tcp" to "seed"', () => {
    const bridge = new DiscoveryBridge();
    expect(bridge.resolveServiceType('_cognitum._tcp')).toBe('seed');
  });

  it('resolveServiceType resolves "_rlmx._tcp" to "rlmx"', () => {
    const bridge = new DiscoveryBridge();
    expect(bridge.resolveServiceType('_rlmx._tcp')).toBe('rlmx');
  });

  it('resolveServiceType returns undefined for unknown type', () => {
    const bridge = new DiscoveryBridge();
    expect(bridge.resolveServiceType('_unknown._tcp')).toBeUndefined();
  });

  it('registerServiceType adds custom mapping', () => {
    const bridge = new DiscoveryBridge();
    expect(bridge.resolveServiceType('_custom._tcp')).toBeUndefined();

    bridge.registerServiceType('_custom._tcp', 'my-service');
    expect(bridge.resolveServiceType('_custom._tcp')).toBe('my-service');
  });

  it('manifestForSeedNode generates valid manifest with correct origin', () => {
    const bridge = new DiscoveryBridge();
    const node = makeSeedNode();
    const manifest = bridge.manifestForSeedNode(node);

    expect(manifest.id).toBe('seed-seed-1');
    expect(manifest.origin.type).toBe('seed');
    expect(manifest.lifeDomain).toBeDefined();
    expect(manifest.transports.length).toBeGreaterThan(0);
    expect(manifest.deployment.profiles).toContain('seed-compatible');
  });

  it('manifestForSeedNode uses TXT records for name and version', () => {
    const bridge = new DiscoveryBridge();
    const node = makeSeedNode();
    const manifest = bridge.manifestForSeedNode(node);

    expect(manifest.name).toBe('TempSensor');
    expect(manifest.version).toBe('0.2.0');
  });

  it('off() removes event handler', () => {
    const bridge = new DiscoveryBridge();
    const handler = vi.fn();
    bridge.on('node-discovered', handler);
    bridge.off('node-discovered', handler);

    bridge.addNode(makeSeedNode());
    expect(handler).not.toHaveBeenCalled();
  });
});
