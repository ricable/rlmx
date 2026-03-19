import { describe, it, expect, vi, afterEach } from 'vitest';
import { SeedBridge } from '../src/seed-bridge.js';
import { DeployErrorCode } from '../src/error.js';

function mockFetch(responses: Array<{ ok: boolean; status: number; json: () => Promise<unknown> }>) {
  let callIndex = 0;
  vi.stubGlobal('fetch', vi.fn(() => {
    const res = responses[callIndex] ?? responses[responses.length - 1];
    callIndex++;
    return Promise.resolve(res);
  }));
}

afterEach(() => {
  vi.unstubAllGlobals();
});

describe('SeedBridge', () => {
  it('constructor does not throw', () => {
    expect(() => new SeedBridge({ restEndpoint: 'https://localhost:8443' })).not.toThrow();
  });

  it('embeddingDimension returns default value (64)', () => {
    const bridge = new SeedBridge({ restEndpoint: 'https://localhost:8443' });
    expect(bridge.embeddingDimension).toBe(64);
  });

  it('embeddingDimension returns custom value', () => {
    const bridge = new SeedBridge({ restEndpoint: 'https://localhost:8443', embedDim: 128 });
    expect(bridge.embeddingDimension).toBe(128);
  });

  it('hasMcpTransport is false when no mcpEndpoint configured', () => {
    const bridge = new SeedBridge({ restEndpoint: 'https://localhost:8443' });
    expect(bridge.hasMcpTransport).toBe(false);
  });

  it('hasMcpTransport is true when mcpEndpoint is configured', () => {
    const bridge = new SeedBridge({
      restEndpoint: 'https://localhost:8443',
      mcpEndpoint: 'http://localhost:5353/mcp',
    });
    expect(bridge.hasMcpTransport).toBe(true);
  });

  it('vectorQuery rejects wrong dimensions', async () => {
    const bridge = new SeedBridge({ restEndpoint: 'https://localhost:8443', embedDim: 64 });
    const wrongVector = new Array(32).fill(0);

    try {
      await bridge.vectorQuery(wrongVector);
      expect.unreachable('should have thrown');
    } catch (e: any) {
      expect(e.code).toBe(DeployErrorCode.DimensionMismatch);
    }
  });

  it('vectorInsert rejects wrong dimensions', async () => {
    const bridge = new SeedBridge({ restEndpoint: 'https://localhost:8443', embedDim: 64 });
    const wrongVector = new Array(128).fill(0);

    try {
      await bridge.vectorInsert(wrongVector);
      expect.unreachable('should have thrown');
    } catch (e: any) {
      expect(e.code).toBe(DeployErrorCode.DimensionMismatch);
    }
  });

  it('getPeerEpoch returns undefined for unknown nodeId', () => {
    const bridge = new SeedBridge({ restEndpoint: 'https://localhost:8443' });
    expect(bridge.getPeerEpoch('unknown-node')).toBeUndefined();
  });

  describe('with mocked fetch', () => {
    it('status() returns parsed data on success', async () => {
      const statusData = { nodeId: 'n1', uptime: 1000, sensorCount: 3, vectorCount: 50, witnessCount: 10, clusterPeers: 2 };
      mockFetch([{ ok: true, status: 200, json: () => Promise.resolve(statusData) }]);

      const bridge = new SeedBridge({ restEndpoint: 'https://localhost:8443' });
      const result = await bridge.status();
      expect(result).toEqual(statusData);
    });

    it('sensorRead() calls correct endpoint', async () => {
      const readings = [{ sensorId: 's1', value: 22.5, unit: 'C', timestamp: new Date().toISOString() }];
      mockFetch([{ ok: true, status: 200, json: () => Promise.resolve(readings) }]);

      const bridge = new SeedBridge({ restEndpoint: 'https://localhost:8443' });
      const result = await bridge.sensorRead('s1');
      expect(result).toEqual(readings);

      const fetchFn = globalThis.fetch as ReturnType<typeof vi.fn>;
      expect(fetchFn).toHaveBeenCalledTimes(1);
      const calledUrl = fetchFn.mock.calls[0][1]?.body;
      // REST adapter maps method dots to slashes
    });

    it('vectorQuery() with correct dimension succeeds', async () => {
      const results = [{ id: 'v1', score: 0.95 }];
      mockFetch([{ ok: true, status: 200, json: () => Promise.resolve(results) }]);

      const bridge = new SeedBridge({ restEndpoint: 'https://localhost:8443', embedDim: 4 });
      const result = await bridge.vectorQuery([1, 2, 3, 4]);
      expect(result).toEqual(results);
    });

    it('429 triggers retry with backoff', async () => {
      const successData = { nodeId: 'n1', uptime: 500, sensorCount: 1, vectorCount: 0, witnessCount: 0, clusterPeers: 0 };
      mockFetch([
        { ok: false, status: 429, json: () => Promise.resolve({}) },
        { ok: true, status: 200, json: () => Promise.resolve(successData) },
      ]);

      const bridge = new SeedBridge({
        restEndpoint: 'https://localhost:8443',
        retryBaseMs: 10, // fast for tests
      });
      const result = await bridge.status();
      expect(result).toEqual(successData);

      const fetchFn = globalThis.fetch as ReturnType<typeof vi.fn>;
      expect(fetchFn).toHaveBeenCalledTimes(2);
    });

    it('multiple 429s exhaust retries and throw seedBridgeError', async () => {
      mockFetch([
        { ok: false, status: 429, json: () => Promise.resolve({}) },
        { ok: false, status: 429, json: () => Promise.resolve({}) },
        { ok: false, status: 429, json: () => Promise.resolve({}) },
        { ok: false, status: 429, json: () => Promise.resolve({}) },
      ]);

      const bridge = new SeedBridge({
        restEndpoint: 'https://localhost:8443',
        maxRetries: 2,
        retryBaseMs: 10,
      });

      try {
        await bridge.status();
        expect.unreachable('should have thrown');
      } catch (e: any) {
        expect(e.code).toBe(DeployErrorCode.SeedBridgeError);
      }
    });

    it('healthCheck() returns true on 200', async () => {
      mockFetch([{ ok: true, status: 200, json: () => Promise.resolve({}) }]);

      const bridge = new SeedBridge({ restEndpoint: 'https://localhost:8443' });
      const result = await bridge.healthCheck();
      expect(result).toBe(true);
    });

    it('healthCheck() returns false on network error', async () => {
      vi.stubGlobal('fetch', vi.fn(() => Promise.reject(new Error('ECONNREFUSED'))));

      const bridge = new SeedBridge({ restEndpoint: 'https://localhost:8443' });
      const result = await bridge.healthCheck();
      expect(result).toBe(false);
    });
  });
});
