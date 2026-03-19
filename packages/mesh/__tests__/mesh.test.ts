import { describe, it, expect } from "vitest";
import {
  PersonalMesh,
  MeshError,
  defaultCapabilities,
} from "../src/index.js";
import type { MeshDomainEvent } from "../src/index.js";

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

let deviceCounter = 0;
function nextDeviceId(): string {
  return `dev-${++deviceCounter}`;
}

function createMesh(): PersonalMesh {
  return new PersonalMesh(`mesh-${Date.now()}`);
}

function addLaptop(mesh: PersonalMesh): string {
  const id = nextDeviceId();
  mesh.addDevice(id, "laptop", "Laptop", defaultCapabilities(), "tok");
  return id;
}

function addHub(mesh: PersonalMesh): string {
  const id = nextDeviceId();
  mesh.addDevice(id, "hub", "HomeHub", defaultCapabilities(), "tok");
  return id;
}

function addPhone(mesh: PersonalMesh): string {
  const id = nextDeviceId();
  mesh.addDevice(id, "phone", "Phone", defaultCapabilities(), "tok");
  return id;
}

// ---------------------------------------------------------------------------
// Device management
// ---------------------------------------------------------------------------

describe("PersonalMesh — device management", () => {
  it("adds a device successfully", () => {
    const mesh = createMesh();
    const id = addLaptop(mesh);
    expect(mesh.devices).toHaveLength(1);
    expect(mesh.devices[0].deviceId).toBe(id);
    expect(mesh.devices[0].zone).toBe("ADesktop");
  });

  it("creates sync states when adding devices", () => {
    const mesh = createMesh();
    addLaptop(mesh);
    addHub(mesh);
    // One sync state for the pair.
    expect(mesh.syncStates).toHaveLength(1);
  });

  it("creates N-1 sync states for Nth device", () => {
    const mesh = createMesh();
    addLaptop(mesh);
    addHub(mesh);
    addPhone(mesh);
    // 3 devices => 3 pairs: (laptop,hub), (laptop,phone), (hub,phone)
    expect(mesh.syncStates).toHaveLength(3);
  });

  it("rejects duplicate device", () => {
    const mesh = createMesh();
    const id = nextDeviceId();
    mesh.addDevice(id, "a", "Laptop", defaultCapabilities(), "t");
    expect(() =>
      mesh.addDevice(id, "b", "Phone", defaultCapabilities(), "t"),
    ).toThrow(MeshError);
    try {
      mesh.addDevice(id, "b", "Phone", defaultCapabilities(), "t");
    } catch (e) {
      expect(e).toBeInstanceOf(MeshError);
      expect((e as MeshError).code).toBe("DEVICE_ALREADY_EXISTS");
    }
  });

  it("enforces max device limit of 10", () => {
    const mesh = createMesh();
    for (let i = 0; i < 10; i++) {
      mesh.addDevice(nextDeviceId(), `dev-${i}`, "Browser", defaultCapabilities(), "t");
    }
    expect(() =>
      mesh.addDevice(nextDeviceId(), "one-too-many", "Browser", defaultCapabilities(), "t"),
    ).toThrow(MeshError);
    try {
      mesh.addDevice(nextDeviceId(), "overflow", "Browser", defaultCapabilities(), "t");
    } catch (e) {
      expect((e as MeshError).code).toBe("MAX_DEVICES_REACHED");
    }
  });

  it("removes a device successfully", () => {
    const mesh = createMesh();
    const id = addLaptop(mesh);
    mesh.removeDevice(id, true);
    expect(mesh.devices).toHaveLength(0);
  });

  it("clears sync states when removing a device", () => {
    const mesh = createMesh();
    const id1 = addLaptop(mesh);
    addHub(mesh);
    expect(mesh.syncStates).toHaveLength(1);
    mesh.removeDevice(id1, true);
    expect(mesh.syncStates).toHaveLength(0);
  });

  it("clears privacy anchor when removing the anchor device", () => {
    const mesh = createMesh();
    const hubId = addHub(mesh);
    mesh.setPrivacyAnchor(hubId);
    expect(mesh.privacyAnchor).toBe(hubId);
    mesh.removeDevice(hubId, false);
    expect(mesh.privacyAnchor).toBeUndefined();
  });

  it("throws when removing an unknown device", () => {
    const mesh = createMesh();
    expect(() => mesh.removeDevice("nonexistent", true)).toThrow(MeshError);
  });
});

// ---------------------------------------------------------------------------
// Zone assignment
// ---------------------------------------------------------------------------

describe("PersonalMesh — zone assignment", () => {
  it("reassigns a device zone", () => {
    const mesh = createMesh();
    const id = addLaptop(mesh);
    mesh.assignZone(id, "BCloud");
    expect(mesh.devices[0].zone).toBe("BCloud");
  });

  it("throws when assigning zone to unknown device", () => {
    const mesh = createMesh();
    expect(() => mesh.assignZone("nonexistent", "BCloud")).toThrow(MeshError);
  });
});

// ---------------------------------------------------------------------------
// Privacy anchor invariant
// ---------------------------------------------------------------------------

describe("PersonalMesh — privacy anchor", () => {
  it("sets privacy anchor on CEdge device", () => {
    const mesh = createMesh();
    const hubId = addHub(mesh);
    mesh.setPrivacyAnchor(hubId);
    expect(mesh.privacyAnchor).toBe(hubId);
  });

  it("rejects privacy anchor on non-CEdge device (INVARIANT)", () => {
    const mesh = createMesh();
    const laptopId = addLaptop(mesh);
    expect(() => mesh.setPrivacyAnchor(laptopId)).toThrow(MeshError);
    try {
      mesh.setPrivacyAnchor(laptopId);
    } catch (e) {
      expect((e as MeshError).code).toBe("INVALID_PRIVACY_ANCHOR_ZONE");
    }
  });

  it("rejects privacy anchor on Phone (AMobile)", () => {
    const mesh = createMesh();
    const phoneId = addPhone(mesh);
    expect(() => mesh.setPrivacyAnchor(phoneId)).toThrow(MeshError);
  });

  it("rejects privacy anchor on unknown device", () => {
    const mesh = createMesh();
    expect(() => mesh.setPrivacyAnchor("nonexistent")).toThrow(MeshError);
  });

  it("returns the privacy anchor device when set", () => {
    const mesh = createMesh();
    const hubId = addHub(mesh);
    mesh.setPrivacyAnchor(hubId);
    const dev = mesh.privacyAnchorDevice();
    expect(dev).toBeDefined();
    expect(dev!.deviceId).toBe(hubId);
  });

  it("returns undefined when no anchor set", () => {
    const mesh = createMesh();
    expect(mesh.privacyAnchorDevice()).toBeUndefined();
  });
});

// ---------------------------------------------------------------------------
// Agent management
// ---------------------------------------------------------------------------

describe("PersonalMesh — agent management", () => {
  it("places an agent on a device", () => {
    const mesh = createMesh();
    const devId = addLaptop(mesh);
    mesh.placeAgent("agent-1", "Worker", devId);
    const agents = mesh.agentsOnDevice(devId);
    expect(agents).toHaveLength(1);
    expect(agents[0].agentType).toBe("Worker");
  });

  it("throws when placing agent on unknown device", () => {
    const mesh = createMesh();
    expect(() => mesh.placeAgent("agent-1", "Worker", "nonexistent")).toThrow(
      MeshError,
    );
  });

  it("throws when placing duplicate agent", () => {
    const mesh = createMesh();
    const devId = addLaptop(mesh);
    mesh.placeAgent("agent-1", "Worker", devId);
    expect(() => mesh.placeAgent("agent-1", "Worker", devId)).toThrow(
      MeshError,
    );
    try {
      mesh.placeAgent("agent-1", "Router", devId);
    } catch (e) {
      expect((e as MeshError).code).toBe("AGENT_ALREADY_ASSIGNED");
    }
  });

  it("migrates an agent between devices", () => {
    const mesh = createMesh();
    const dev1 = addLaptop(mesh);
    const dev2 = addHub(mesh);
    mesh.placeAgent("agent-1", "Worker", dev1);
    mesh.migrateAgent("agent-1", dev2, "LoadBalancing");
    expect(mesh.agentsOnDevice(dev1)).toHaveLength(0);
    expect(mesh.agentsOnDevice(dev2)).toHaveLength(1);
  });

  it("throws when migrating to unknown device", () => {
    const mesh = createMesh();
    const dev = addLaptop(mesh);
    mesh.placeAgent("agent-1", "Worker", dev);
    expect(() =>
      mesh.migrateAgent("agent-1", "nonexistent", "LoadBalancing"),
    ).toThrow(MeshError);
  });

  it("throws when migrating unknown agent", () => {
    const mesh = createMesh();
    const dev = addLaptop(mesh);
    expect(() =>
      mesh.migrateAgent("nonexistent", dev, "LoadBalancing"),
    ).toThrow(MeshError);
  });
});

// ---------------------------------------------------------------------------
// Queries
// ---------------------------------------------------------------------------

describe("PersonalMesh — queries", () => {
  it("returns devices in a specific zone", () => {
    const mesh = createMesh();
    addLaptop(mesh);
    addHub(mesh);
    expect(mesh.devicesInZone("ADesktop")).toHaveLength(1);
    expect(mesh.devicesInZone("CEdge")).toHaveLength(1);
    expect(mesh.devicesInZone("BCloud")).toHaveLength(0);
  });

  it("returns online devices only", () => {
    const mesh = createMesh();
    const id = addLaptop(mesh);
    addHub(mesh);
    mesh.updateDeviceStatus(id, "Offline");
    expect(mesh.onlineDevices()).toHaveLength(1);
  });

  it("returns coordinator when ADesktop device is online", () => {
    const mesh = createMesh();
    const id = addLaptop(mesh);
    expect(mesh.coordinator()).toBeDefined();
    expect(mesh.coordinator()!.deviceId).toBe(id);
  });

  it("returns undefined coordinator when ADesktop is offline", () => {
    const mesh = createMesh();
    const id = addLaptop(mesh);
    mesh.updateDeviceStatus(id, "Offline");
    expect(mesh.coordinator()).toBeUndefined();
  });

  it("returns fleet status summary", () => {
    const mesh = createMesh();
    const dev1 = addLaptop(mesh);
    addHub(mesh);
    mesh.placeAgent("agent-1", "Worker", dev1);

    const status = mesh.getFleetStatus();
    expect(status.totalDevices).toBe(2);
    expect(status.onlineDevices).toBe(2);
    expect(status.totalAgents).toBe(1);
    expect(status.hasPrivacyAnchor).toBe(false);
    expect(status.hasCoordinator).toBe(true);
  });
});

// ---------------------------------------------------------------------------
// Device status updates
// ---------------------------------------------------------------------------

describe("PersonalMesh — device status", () => {
  it("updates device status", () => {
    const mesh = createMesh();
    const id = addLaptop(mesh);
    mesh.updateDeviceStatus(id, "Degraded");
    expect(mesh.devices[0].status).toBe("Degraded");
  });

  it("throws when updating unknown device status", () => {
    const mesh = createMesh();
    expect(() => mesh.updateDeviceStatus("nonexistent", "Online")).toThrow(
      MeshError,
    );
  });
});

// ---------------------------------------------------------------------------
// Domain events
// ---------------------------------------------------------------------------

describe("PersonalMesh — domain events", () => {
  it("emits DeviceJoined on addDevice", () => {
    const mesh = createMesh();
    addLaptop(mesh);
    const events = mesh.takeEvents();
    expect(events).toHaveLength(1);
    expect(events[0].type).toBe("DeviceJoined");
  });

  it("emits DeviceLeft on removeDevice", () => {
    const mesh = createMesh();
    const id = addLaptop(mesh);
    mesh.takeEvents(); // drain DeviceJoined
    mesh.removeDevice(id, true);
    const events = mesh.takeEvents();
    expect(events).toHaveLength(1);
    expect(events[0].type).toBe("DeviceLeft");
    if (events[0].type === "DeviceLeft") {
      expect(events[0].graceful).toBe(true);
    }
  });

  it("emits MeshReconfigured on assignZone", () => {
    const mesh = createMesh();
    const id = addLaptop(mesh);
    mesh.takeEvents(); // drain
    mesh.assignZone(id, "BCloud");
    const events = mesh.takeEvents();
    expect(events).toHaveLength(1);
    expect(events[0].type).toBe("MeshReconfigured");
  });

  it("emits AgentMigrated on migrateAgent", () => {
    const mesh = createMesh();
    const dev1 = addLaptop(mesh);
    const dev2 = addHub(mesh);
    mesh.placeAgent("agent-1", "Worker", dev1);
    mesh.takeEvents(); // drain
    mesh.migrateAgent("agent-1", dev2, "DeviceOffline");
    const events = mesh.takeEvents();
    expect(events).toHaveLength(1);
    expect(events[0].type).toBe("AgentMigrated");
    if (events[0].type === "AgentMigrated") {
      expect(events[0].fromDevice).toBe(dev1);
      expect(events[0].toDevice).toBe(dev2);
      expect(events[0].reason).toBe("DeviceOffline");
    }
  });

  it("takeEvents drains the buffer", () => {
    const mesh = createMesh();
    addLaptop(mesh);
    const events = mesh.takeEvents();
    expect(events).toHaveLength(1);
    // Second take returns empty.
    expect(mesh.takeEvents()).toHaveLength(0);
  });

  it("accumulates multiple events before draining", () => {
    const mesh = createMesh();
    addLaptop(mesh);
    addHub(mesh);
    addPhone(mesh);
    const events = mesh.takeEvents();
    expect(events).toHaveLength(3);
    expect(events.every((e) => e.type === "DeviceJoined")).toBe(true);
  });
});

// ---------------------------------------------------------------------------
// Invariant: sync states track pending ops on device removal
// ---------------------------------------------------------------------------

describe("PersonalMesh — sync invariants", () => {
  it("reports pending ops in DeviceLeft event", () => {
    const mesh = createMesh();
    const dev1 = addLaptop(mesh);
    addHub(mesh);
    mesh.takeEvents(); // drain

    // Manually there are no pending ops since createSyncState starts at 0.
    mesh.removeDevice(dev1, false);
    const events = mesh.takeEvents();
    expect(events).toHaveLength(1);
    if (events[0].type === "DeviceLeft") {
      expect(events[0].pendingOps).toBe(0);
    }
  });
});

// ---------------------------------------------------------------------------
// Transport selection via sync states
// ---------------------------------------------------------------------------

describe("PersonalMesh — transport selection", () => {
  it("uses QUIC for ADesktop <-> CEdge pair", () => {
    const mesh = createMesh();
    addLaptop(mesh); // ADesktop
    addHub(mesh);     // CEdge
    expect(mesh.syncStates[0].transport).toBe("Quic");
  });

  it("uses WebSocket for AMobile <-> any pair", () => {
    const mesh = createMesh();
    addPhone(mesh);   // AMobile
    addLaptop(mesh);  // ADesktop
    expect(mesh.syncStates[0].transport).toBe("WebSocket");
  });

  it("uses BroadcastChannel for DBrowser pair", () => {
    const mesh = createMesh();
    mesh.addDevice(nextDeviceId(), "browser", "Browser", defaultCapabilities(), "t");
    addLaptop(mesh);
    expect(mesh.syncStates[0].transport).toBe("BroadcastChannel");
  });
});
