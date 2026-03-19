// ---------------------------------------------------------------------------
// @aix/mesh — Error types for the personal mesh bounded context
//
// Maps to: crates/rlmx-mesh/src/error.rs
// ---------------------------------------------------------------------------

import type { DeviceZone, DeviceType } from "./device.js";

/** Discriminated union of all mesh error codes. */
export type MeshErrorCode =
  | "DEVICE_NOT_FOUND"
  | "DEVICE_ALREADY_EXISTS"
  | "AGENT_NOT_FOUND"
  | "AGENT_ALREADY_ASSIGNED"
  | "INVALID_PRIVACY_ANCHOR_ZONE"
  | "PRIVACY_ANCHOR_ALREADY_SET"
  | "COORDINATOR_ALREADY_EXISTS"
  | "INCOMPATIBLE_ZONE"
  | "SYNC_ERROR"
  | "DISCOVERY_ERROR"
  | "FAILOVER_ERROR"
  | "MANIFEST_VERSION_MISMATCH"
  | "MAX_DEVICES_REACHED"
  | "INVALID_OPERATION";

/**
 * Structured error for mesh operations.
 *
 * Includes a machine-readable `code` and optional contextual fields
 * matching the Rust `MeshError` variants.
 */
export class MeshError extends Error {
  readonly code: MeshErrorCode;

  /** Device or agent UUID relevant to the error, if any. */
  readonly entityId?: string;

  /** Zone relevant to the error, if any. */
  readonly zone?: DeviceZone;

  /** Device type relevant to the error, if any. */
  readonly deviceType?: DeviceType;

  /** Additional detail fields (e.g. expected/actual versions). */
  readonly detail?: Record<string, unknown>;

  constructor(
    code: MeshErrorCode,
    message: string,
    opts?: {
      entityId?: string;
      zone?: DeviceZone;
      deviceType?: DeviceType;
      detail?: Record<string, unknown>;
    },
  ) {
    super(message);
    this.name = "MeshError";
    this.code = code;
    Object.setPrototypeOf(this, new.target.prototype);
    this.entityId = opts?.entityId;
    this.zone = opts?.zone;
    this.deviceType = opts?.deviceType;
    this.detail = opts?.detail;
  }
}

/** Convenience alias matching Rust's `MeshResult<T>`. */
export type MeshResult<T> = T; // Functions throw MeshError on failure
