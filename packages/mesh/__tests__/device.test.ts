import { describe, it, expect } from "vitest";
import {
  zoneToSwarmZone,
  isCoordinatorZone,
  isPrivacyAnchorZone,
  defaultZoneForDeviceType,
  defaultCapabilities,
  createMeshDevice,
  isDeviceOnline,
  ALL_ZONES,
} from "../src/index.js";
import type { DeviceZone, DeviceType, MeshDevice } from "../src/index.js";

// ---------------------------------------------------------------------------
// Zone helpers
// ---------------------------------------------------------------------------

describe("zoneToSwarmZone", () => {
  it("maps all zones to swarm zone strings", () => {
    expect(zoneToSwarmZone("ADesktop")).toBe("zone-a-desktop");
    expect(zoneToSwarmZone("AMobile")).toBe("zone-a-mobile");
    expect(zoneToSwarmZone("BCloud")).toBe("zone-b");
    expect(zoneToSwarmZone("CEdge")).toBe("zone-c");
    expect(zoneToSwarmZone("DBrowser")).toBe("zone-d");
  });
});

describe("isCoordinatorZone", () => {
  it("returns true only for ADesktop", () => {
    expect(isCoordinatorZone("ADesktop")).toBe(true);
    expect(isCoordinatorZone("AMobile")).toBe(false);
    expect(isCoordinatorZone("BCloud")).toBe(false);
    expect(isCoordinatorZone("CEdge")).toBe(false);
    expect(isCoordinatorZone("DBrowser")).toBe(false);
  });
});

describe("isPrivacyAnchorZone", () => {
  it("returns true only for CEdge", () => {
    expect(isPrivacyAnchorZone("CEdge")).toBe(true);
    expect(isPrivacyAnchorZone("ADesktop")).toBe(false);
    expect(isPrivacyAnchorZone("AMobile")).toBe(false);
    expect(isPrivacyAnchorZone("BCloud")).toBe(false);
    expect(isPrivacyAnchorZone("DBrowser")).toBe(false);
  });
});

// ---------------------------------------------------------------------------
// Device type -> zone mapping
// ---------------------------------------------------------------------------

describe("defaultZoneForDeviceType", () => {
  it("maps Laptop to ADesktop", () => {
    expect(defaultZoneForDeviceType("Laptop")).toBe("ADesktop");
  });

  it("maps Phone to AMobile", () => {
    expect(defaultZoneForDeviceType("Phone")).toBe("AMobile");
  });

  it("maps HomeHub to CEdge", () => {
    expect(defaultZoneForDeviceType("HomeHub")).toBe("CEdge");
  });

  it("maps CloudNode to BCloud", () => {
    expect(defaultZoneForDeviceType("CloudNode")).toBe("BCloud");
  });

  it("maps Browser to DBrowser", () => {
    expect(defaultZoneForDeviceType("Browser")).toBe("DBrowser");
  });

  it("maps Sensor to CEdge", () => {
    expect(defaultZoneForDeviceType("Sensor")).toBe("CEdge");
  });
});

// ---------------------------------------------------------------------------
// DeviceCapabilities defaults
// ---------------------------------------------------------------------------

describe("defaultCapabilities", () => {
  it("returns sensible defaults", () => {
    const caps = defaultCapabilities();
    expect(caps.cpuCores).toBe(4);
    expect(caps.memoryMb).toBe(8192);
    expect(caps.hasGpu).toBe(false);
    expect(caps.hasBattery).toBe(false);
    expect(caps.gpuType).toBeUndefined();
    expect(caps.batteryPct).toBeUndefined();
  });
});

// ---------------------------------------------------------------------------
// MeshDevice creation
// ---------------------------------------------------------------------------

describe("createMeshDevice", () => {
  it("creates a device with default zone from its type", () => {
    const dev = createMeshDevice(
      "dev-001",
      "my-laptop",
      "Laptop",
      defaultCapabilities(),
      "tok_abc",
    );
    expect(dev.zone).toBe("ADesktop");
    expect(dev.status).toBe("Online");
    expect(dev.name).toBe("my-laptop");
    expect(dev.deviceType).toBe("Laptop");
    expect(dev.token).toBe("tok_abc");
  });

  it("creates a HomeHub device in CEdge zone", () => {
    const dev = createMeshDevice(
      "dev-002",
      "hub",
      "HomeHub",
      defaultCapabilities(),
      "tok_hub",
    );
    expect(dev.zone).toBe("CEdge");
  });
});

// ---------------------------------------------------------------------------
// isDeviceOnline
// ---------------------------------------------------------------------------

describe("isDeviceOnline", () => {
  function makeDevice(status: MeshDevice["status"]): MeshDevice {
    const dev = createMeshDevice("d", "d", "Laptop", defaultCapabilities(), "t");
    dev.status = status;
    return dev;
  }

  it("Online is reachable", () => {
    expect(isDeviceOnline(makeDevice("Online"))).toBe(true);
  });

  it("Syncing is reachable", () => {
    expect(isDeviceOnline(makeDevice("Syncing"))).toBe(true);
  });

  it("Offline is not reachable", () => {
    expect(isDeviceOnline(makeDevice("Offline"))).toBe(false);
  });

  it("Degraded is not reachable", () => {
    expect(isDeviceOnline(makeDevice("Degraded"))).toBe(false);
  });
});

// ---------------------------------------------------------------------------
// ALL_ZONES
// ---------------------------------------------------------------------------

describe("ALL_ZONES", () => {
  it("contains exactly 5 zones", () => {
    expect(ALL_ZONES).toHaveLength(5);
  });

  it("contains all expected zones", () => {
    const expected: DeviceZone[] = [
      "ADesktop",
      "AMobile",
      "BCloud",
      "CEdge",
      "DBrowser",
    ];
    for (const z of expected) {
      expect(ALL_ZONES).toContain(z);
    }
  });
});
