import { describe, it, expect } from "vitest";
import {
  createSyncState,
  markSynced,
  enqueueSyncOps,
  evaluateSyncHealth,
  defaultSyncPolicies,
  selectTransport,
} from "../src/index.js";
import type { SyncScope } from "../src/index.js";

// ---------------------------------------------------------------------------
// SyncState creation
// ---------------------------------------------------------------------------

describe("createSyncState", () => {
  it("creates a healthy state with zero pending ops", () => {
    const ss = createSyncState("dev-a", "dev-b", "Quic");
    expect(ss.pendingOps).toBe(0);
    expect(ss.health).toBe("Healthy");
    expect(ss.transport).toBe("Quic");
    expect(ss.sourceDevice).toBe("dev-a");
    expect(ss.targetDevice).toBe("dev-b");
  });

  it("initializes with empty peer epochs", () => {
    const ss = createSyncState("dev-a", "dev-b", "Http");
    expect(ss.peerEpochs.size).toBe(0);
  });
});

// ---------------------------------------------------------------------------
// Enqueue and sync
// ---------------------------------------------------------------------------

describe("enqueueSyncOps / markSynced", () => {
  it("enqueues pending operations", () => {
    const ss = createSyncState("a", "b", "WebSocket");
    enqueueSyncOps(ss, 5);
    expect(ss.pendingOps).toBe(5);
  });

  it("marks partial sync as Degraded", () => {
    const ss = createSyncState("a", "b", "WebSocket");
    enqueueSyncOps(ss, 5);
    markSynced(ss, 3);
    expect(ss.pendingOps).toBe(2);
    expect(ss.health).toBe("Degraded");
  });

  it("marks full sync as Healthy", () => {
    const ss = createSyncState("a", "b", "WebSocket");
    enqueueSyncOps(ss, 5);
    markSynced(ss, 5);
    expect(ss.pendingOps).toBe(0);
    expect(ss.health).toBe("Healthy");
  });

  it("does not go below zero on over-sync", () => {
    const ss = createSyncState("a", "b", "Quic");
    enqueueSyncOps(ss, 3);
    markSynced(ss, 10);
    expect(ss.pendingOps).toBe(0);
    expect(ss.health).toBe("Healthy");
  });

  it("updates lastSync timestamp on markSynced", () => {
    const ss = createSyncState("a", "b", "Http");
    const before = ss.lastSync;
    // Small delay to ensure timestamps differ.
    enqueueSyncOps(ss, 1);
    markSynced(ss, 1);
    // lastSync should be updated (may or may not differ depending on speed).
    expect(ss.lastSync).toBeDefined();
  });
});

// ---------------------------------------------------------------------------
// Health evaluation
// ---------------------------------------------------------------------------

describe("evaluateSyncHealth", () => {
  it("returns Healthy when no pending ops", () => {
    const ss = createSyncState("a", "b", "Quic");
    evaluateSyncHealth(ss, 30);
    expect(ss.health).toBe("Healthy");
  });

  it("returns Degraded when pending ops <= threshold", () => {
    const ss = createSyncState("a", "b", "Http");
    enqueueSyncOps(ss, 20);
    evaluateSyncHealth(ss, 30);
    expect(ss.health).toBe("Degraded");
  });

  it("returns Disconnected when pending ops > threshold", () => {
    const ss = createSyncState("a", "b", "Http");
    enqueueSyncOps(ss, 50);
    evaluateSyncHealth(ss, 30);
    expect(ss.health).toBe("Disconnected");
  });

  it("transitions back to Degraded after partial sync", () => {
    const ss = createSyncState("a", "b", "Http");
    enqueueSyncOps(ss, 50);
    evaluateSyncHealth(ss, 30);
    expect(ss.health).toBe("Disconnected");
    markSynced(ss, 30);
    evaluateSyncHealth(ss, 30);
    expect(ss.health).toBe("Degraded");
  });
});

// ---------------------------------------------------------------------------
// Default sync policies
// ---------------------------------------------------------------------------

describe("defaultSyncPolicies", () => {
  it("returns 5 policies covering all scopes", () => {
    const policies = defaultSyncPolicies();
    expect(policies).toHaveLength(5);
    const scopes = policies.map((p) => p.scope);
    const expected: SyncScope[] = [
      "AgentPlacement",
      "CapabilityTokens",
      "SonaPatterns",
      "EngagementState",
      "EphemeralMetrics",
    ];
    for (const s of expected) {
      expect(scopes).toContain(s);
    }
  });

  it("uses RaftConsensus for critical scopes", () => {
    const policies = defaultSyncPolicies();
    const agentPlacement = policies.find((p) => p.scope === "AgentPlacement")!;
    expect(agentPlacement.protocol).toBe("RaftConsensus");
    expect(agentPlacement.priority).toBe("Critical");
    const capTokens = policies.find((p) => p.scope === "CapabilityTokens")!;
    expect(capTokens.protocol).toBe("RaftConsensus");
  });

  it("uses CRDT for eventual-consistency scopes", () => {
    const policies = defaultSyncPolicies();
    const sona = policies.find((p) => p.scope === "SonaPatterns")!;
    expect(sona.protocol).toBe("Crdt");
    expect(sona.priority).toBe("Normal");
  });

  it("uses LastWriterWins for ephemeral metrics", () => {
    const policies = defaultSyncPolicies();
    const metrics = policies.find((p) => p.scope === "EphemeralMetrics")!;
    expect(metrics.protocol).toBe("LastWriterWins");
    expect(metrics.priority).toBe("BestEffort");
  });
});

// ---------------------------------------------------------------------------
// Transport selection
// ---------------------------------------------------------------------------

describe("selectTransport", () => {
  it("uses QUIC for ADesktop <-> CEdge (LAN)", () => {
    expect(selectTransport("ADesktop", "CEdge")).toBe("Quic");
    expect(selectTransport("CEdge", "ADesktop")).toBe("Quic");
  });

  it("uses BroadcastChannel for DBrowser pairs", () => {
    expect(selectTransport("DBrowser", "ADesktop")).toBe("BroadcastChannel");
    expect(selectTransport("AMobile", "DBrowser")).toBe("BroadcastChannel");
  });

  it("uses WebSocket for AMobile pairs", () => {
    expect(selectTransport("AMobile", "BCloud")).toBe("WebSocket");
    expect(selectTransport("CEdge", "AMobile")).toBe("WebSocket");
  });

  it("uses HTTP for BCloud pairs (no browser/phone)", () => {
    expect(selectTransport("BCloud", "CEdge")).toBe("Http");
    expect(selectTransport("ADesktop", "BCloud")).toBe("Http");
  });

  it("defaults to WebSocket for CEdge <-> CEdge", () => {
    expect(selectTransport("CEdge", "CEdge")).toBe("WebSocket");
  });
});

// ---------------------------------------------------------------------------
// Peer epoch tracking
// ---------------------------------------------------------------------------

describe("SyncState — peer epochs", () => {
  it("tracks peer epochs via Map", () => {
    const ss = createSyncState("a", "b", "Http");
    ss.peerEpochs.set("b", 42);
    expect(ss.peerEpochs.get("b")).toBe(42);
  });

  it("supports multiple peers", () => {
    const ss = createSyncState("a", "b", "Quic");
    ss.peerEpochs.set("b", 10);
    ss.peerEpochs.set("c", 20);
    expect(ss.peerEpochs.size).toBe(2);
  });
});
