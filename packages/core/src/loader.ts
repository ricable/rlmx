// ---------------------------------------------------------------------------
// @aix/core — Platform detection and native binary loading
//
// Detects the current platform/arch combination and attempts to load the
// corresponding platform-specific npm package (e.g. @aix/core-darwin-arm64).
// If loading fails, returns a degraded proxy that resolves every call with
// { status: 'unavailable', reason: '...' }.
// ---------------------------------------------------------------------------

import { createRequire } from "node:module";
import type { NativeBindings, UnavailableResult } from "./types.js";

/** Supported platform triples mapped to their npm package names. */
const PLATFORM_PACKAGES: Record<string, string> = {
  "darwin-arm64": "@aix/core-darwin-arm64",
  "darwin-x64": "@aix/core-darwin-x64",
  "linux-x64": "@aix/core-linux-x64-gnu",
  "linux-arm64": "@aix/core-linux-arm64-gnu",
  "win32-x64": "@aix/core-win32-x64-msvc",
};

/**
 * Detect the current platform triple string (e.g. "darwin-arm64").
 * Returns `null` if the platform is unsupported.
 */
export function detectPlatform(): string | null {
  const key = `${process.platform}-${process.arch}`;
  return key in PLATFORM_PACKAGES ? key : null;
}

/**
 * Get the npm package name for the current platform.
 * Returns `null` if the platform is unsupported.
 */
export function getPackageName(): string | null {
  const platform = detectPlatform();
  return platform ? PLATFORM_PACKAGES[platform] ?? null : null;
}

/**
 * Build the unavailable result constant. Every degraded proxy call resolves
 * to this value so callers can check `result.status === 'unavailable'`.
 */
function unavailable(reason: string): UnavailableResult {
  return { status: "unavailable", reason };
}

/**
 * All function names on the NativeBindings interface. Used to build the
 * degraded proxy dynamically so we don't have to maintain a manual list
 * that can drift.
 */
const BINDING_NAMES: ReadonlyArray<keyof NativeBindings> = [
  // Existing (6)
  "napiDispatch",
  "napiSonaQuery",
  "napiRvfSeal",
  "napiAgentSpawn",
  "napiSwarmStatus",
  "napiRoute",
  // Voice (6)
  "napiVoicePipelineCreate",
  "napiVoiceVadProcess",
  "napiVoiceTranscribe",
  "napiVoiceDecomposeIntents",
  "napiVoiceSynthesize",
  "napiVoiceSessionCreate",
  // Phone (8)
  "napiPhoneRuntimeCreate",
  "napiPhoneEngagementScore",
  "napiPhoneSavingsRecord",
  "napiPhoneStreakCheckIn",
  "napiPhoneStreakStatus",
  "napiPhoneBatteryPolicy",
  "napiPhoneCoordinatorStatus",
  "napiPhoneNotificationSend",
  // Cognitive (5)
  "napiSonaRecord",
  "napiSonaAdapt",
  "napiVoicePatternSearch",
  "napiFederatedAnonymize",
  "napiFatigueCheck",
  // RVF (1)
  "napiRvfVerify",
  // Inference (5)
  "napiInferenceLoad",
  "napiInferenceGenerate",
  "napiInferenceStatus",
  "napiInferenceTieredRoute",
  "napiInferenceUnload",
] as const;

/**
 * Create a degraded proxy that returns `{ status: 'unavailable', reason }` for
 * every function call. This allows consumer code to operate in degraded mode
 * without throwing.
 */
export function createDegradedBindings(reason: string): NativeBindings {
  const result = unavailable(reason);
  const proxy = {} as Record<string, unknown>;

  for (const name of BINDING_NAMES) {
    proxy[name] = (..._args: unknown[]) => Promise.resolve(result);
  }

  return proxy as unknown as NativeBindings;
}

/**
 * Attempt to load the native binary for the current platform.
 *
 * Loading order:
 * 1. Detect platform triple.
 * 2. Try `require()` on the platform-specific package.
 * 3. On any failure, fall back to the degraded proxy.
 *
 * Returns `{ bindings, nativeAvailable }`.
 */
export function loadNativeBindings(): {
  bindings: NativeBindings;
  nativeAvailable: boolean;
} {
  const platform = detectPlatform();

  if (!platform) {
    return {
      bindings: createDegradedBindings(
        `unsupported platform: ${process.platform}-${process.arch}`,
      ),
      nativeAvailable: false,
    };
  }

  const packageName = PLATFORM_PACKAGES[platform];
  if (!packageName) {
    return {
      bindings: createDegradedBindings(`no package mapping for ${platform}`),
      nativeAvailable: false,
    };
  }

  try {
    // Use createRequire to support both ESM and CJS contexts.
    const req = createRequire(import.meta.url);
    const native = req(packageName) as NativeBindings;

    // Verify the module actually exported the expected functions by
    // checking for at least one known function.
    if (typeof native.napiDispatch !== "function") {
      return {
        bindings: createDegradedBindings(
          `${packageName} loaded but napiDispatch is not a function`,
        ),
        nativeAvailable: false,
      };
    }

    return { bindings: native, nativeAvailable: true };
  } catch (err: unknown) {
    const message =
      err instanceof Error ? err.message : String(err);
    return {
      bindings: createDegradedBindings(
        `failed to load ${packageName}: ${message}`,
      ),
      nativeAvailable: false,
    };
  }
}
