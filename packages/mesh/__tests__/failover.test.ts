import { describe, it, expect } from "vitest";
import {
  defaultFailoverPolicies,
  failoverPolicyForZone,
  evaluateDegradation,
} from "../src/index.js";
import type { DeviceZone } from "../src/index.js";

// ---------------------------------------------------------------------------
// Default failover policies
// ---------------------------------------------------------------------------

describe("defaultFailoverPolicies", () => {
  it("covers all 5 zones", () => {
    const policies = defaultFailoverPolicies();
    expect(policies).toHaveLength(5);
    const zones = policies.map((p) => p.zone);
    const expected: DeviceZone[] = [
      "ADesktop",
      "AMobile",
      "BCloud",
      "CEdge",
      "DBrowser",
    ];
    for (const z of expected) {
      expect(zones).toContain(z);
    }
  });
});

describe("failoverPolicyForZone", () => {
  it("returns laptop policy with BCloud fallback and agent migration", () => {
    const policy = failoverPolicyForZone("ADesktop");
    expect(policy.fallbackZone).toBe("BCloud");
    expect(policy.migrateAgents).toBe(true);
    expect(policy.degradation).toBe("Reduced");
  });

  it("returns home hub policy with no fallback (INVARIANT)", () => {
    const policy = failoverPolicyForZone("CEdge");
    expect(policy.fallbackZone).toBeUndefined();
    expect(policy.degradation).toBe("Offline");
    expect(policy.migrateAgents).toBe(false);
  });

  it("returns phone policy with BCloud fallback", () => {
    const policy = failoverPolicyForZone("AMobile");
    expect(policy.fallbackZone).toBe("BCloud");
    expect(policy.migrateAgents).toBe(false);
  });

  it("returns cloud policy with ADesktop fallback", () => {
    const policy = failoverPolicyForZone("BCloud");
    expect(policy.fallbackZone).toBe("ADesktop");
  });

  it("returns browser policy with CacheOnly degradation", () => {
    const policy = failoverPolicyForZone("DBrowser");
    expect(policy.degradation).toBe("CacheOnly");
    expect(policy.fallbackZone).toBeUndefined();
    expect(policy.migrateAgents).toBe(false);
  });
});

// ---------------------------------------------------------------------------
// Degradation evaluation
// ---------------------------------------------------------------------------

describe("evaluateDegradation", () => {
  it("returns Full when all zones online", () => {
    const deg = evaluateDegradation([
      "ADesktop",
      "AMobile",
      "BCloud",
      "CEdge",
      "DBrowser",
    ]);
    expect(deg.level).toBe("Full");
    expect(deg.offlineZones).toHaveLength(0);
    expect(deg.availableZones).toHaveLength(5);
    expect(deg.description).toBe("All zones operational");
  });

  it("returns Reduced when coordinator (ADesktop) is online", () => {
    const deg = evaluateDegradation(["ADesktop", "AMobile"]);
    expect(deg.level).toBe("Reduced");
    expect(deg.offlineZones).toHaveLength(3);
    expect(deg.description).toContain("3 zone(s) offline");
  });

  it("returns Offline when no coordinator is available", () => {
    const deg = evaluateDegradation(["AMobile", "CEdge"]);
    expect(deg.level).toBe("Offline");
    expect(deg.description).toContain("coordinator unavailable");
  });

  it("returns CacheOnly when no zones are online", () => {
    const deg = evaluateDegradation([]);
    expect(deg.level).toBe("CacheOnly");
    expect(deg.offlineZones).toHaveLength(5);
    expect(deg.availableZones).toHaveLength(0);
  });

  it("returns Reduced with single coordinator", () => {
    const deg = evaluateDegradation(["ADesktop"]);
    expect(deg.level).toBe("Reduced");
    expect(deg.offlineZones).toHaveLength(4);
  });

  it("returns Offline with only browser and cloud", () => {
    const deg = evaluateDegradation(["DBrowser", "BCloud"]);
    expect(deg.level).toBe("Offline");
  });

  it("tracks offline zones correctly", () => {
    const deg = evaluateDegradation(["ADesktop", "CEdge"]);
    expect(deg.offlineZones).toContain("AMobile");
    expect(deg.offlineZones).toContain("BCloud");
    expect(deg.offlineZones).toContain("DBrowser");
    expect(deg.offlineZones).not.toContain("ADesktop");
    expect(deg.offlineZones).not.toContain("CEdge");
  });
});

// ---------------------------------------------------------------------------
// INVARIANT: Home hub offline => no fallback, queue syncs
// ---------------------------------------------------------------------------

describe("Failover invariant — home hub is sole long-term data store", () => {
  it("has no fallback zone for CEdge", () => {
    const policy = failoverPolicyForZone("CEdge");
    expect(policy.fallbackZone).toBeUndefined();
  });

  it("does not migrate agents when CEdge goes offline", () => {
    const policy = failoverPolicyForZone("CEdge");
    expect(policy.migrateAgents).toBe(false);
  });

  it("sets degradation to Offline (not Reduced) when CEdge down", () => {
    const policy = failoverPolicyForZone("CEdge");
    expect(policy.degradation).toBe("Offline");
  });

  it("description mentions OfflineOutbox queuing", () => {
    const policy = failoverPolicyForZone("CEdge");
    expect(policy.description).toContain("queue syncs");
  });
});
