import { describe, it, expect } from "vitest";
import {
  detectPlatform,
  getPackageName,
  createDegradedBindings,
  loadNativeBindings,
} from "../src/loader.js";
import type { UnavailableResult, NativeBindings } from "../src/types.js";

// ---------------------------------------------------------------------------
// Helper to check that a value is the standard unavailable result.
// ---------------------------------------------------------------------------

function expectUnavailable(value: unknown): void {
  const result = value as UnavailableResult;
  expect(result).toBeDefined();
  expect(result.status).toBe("unavailable");
  expect(typeof result.reason).toBe("string");
  expect(result.reason.length).toBeGreaterThan(0);
}

// ---------------------------------------------------------------------------
// detectPlatform
// ---------------------------------------------------------------------------

describe("detectPlatform", () => {
  it("returns a non-null string on supported platforms", () => {
    const platform = detectPlatform();
    // We can't know the exact value in CI, but if running on a supported
    // platform it should be a known triple.
    const supported = [
      "darwin-arm64",
      "darwin-x64",
      "linux-x64",
      "linux-arm64",
      "win32-x64",
    ];
    if (platform !== null) {
      expect(supported).toContain(platform);
    }
  });

  it("returns a string matching process.platform-process.arch", () => {
    const platform = detectPlatform();
    if (platform !== null) {
      expect(platform).toBe(`${process.platform}-${process.arch}`);
    }
  });
});

// ---------------------------------------------------------------------------
// getPackageName
// ---------------------------------------------------------------------------

describe("getPackageName", () => {
  it("returns a scoped package name on supported platforms", () => {
    const pkg = getPackageName();
    if (pkg !== null) {
      expect(pkg).toMatch(/^@aix\/core-/);
    }
  });
});

// ---------------------------------------------------------------------------
// createDegradedBindings
// ---------------------------------------------------------------------------

describe("createDegradedBindings", () => {
  const reason = "test: binary not found";
  const bindings: NativeBindings = createDegradedBindings(reason);

  it("returns an object with all 31 NAPI function names", () => {
    const expectedFunctions = [
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

    for (const name of expectedFunctions) {
      expect(typeof bindings[name]).toBe("function");
    }

    // Verify we have exactly 31 functions.
    expect(expectedFunctions.length).toBe(31);
  });

  it("napiDispatch returns unavailable result", async () => {
    const result = await bindings.napiDispatch("HaltCheck", {});
    expectUnavailable(result);
  });

  it("napiSwarmStatus returns unavailable result", async () => {
    const result = await bindings.napiSwarmStatus();
    expectUnavailable(result);
  });

  it("napiVoicePipelineCreate returns unavailable result", async () => {
    const result = await bindings.napiVoicePipelineCreate({
      language: "en",
      vadEnabled: true,
      sttTier: "small",
      ttsPersona: "Finance",
      maxIntents: 5,
    });
    expectUnavailable(result);
  });

  it("napiPhoneEngagementScore returns unavailable result", async () => {
    const result = await bindings.napiPhoneEngagementScore();
    expectUnavailable(result);
  });

  it("napiSonaRecord returns unavailable result", async () => {
    const result = await bindings.napiSonaRecord("query", ["a"], 0.9);
    expectUnavailable(result);
  });

  it("napiRvfVerify returns unavailable result", async () => {
    const result = await bindings.napiRvfVerify("witness-123");
    expectUnavailable(result);
  });

  it("napiInferenceGenerate returns unavailable result", async () => {
    const result = await bindings.napiInferenceGenerate("hello", 100);
    expectUnavailable(result);
  });

  it("napiInferenceUnload returns unavailable result", async () => {
    const result = await bindings.napiInferenceUnload("model-1");
    expectUnavailable(result);
  });

  it("all functions include the reason in their response", async () => {
    const result = (await bindings.napiRoute(
      new Float32Array([1, 2, 3]),
    )) as UnavailableResult;
    expect(result.reason).toContain(reason);
  });

  it("every function returns a resolved (not rejected) promise", async () => {
    // Ensure no function throws — degraded mode must be safe.
    const fns = Object.values(bindings).filter(
      (v) => typeof v === "function",
    ) as Array<(...args: unknown[]) => Promise<unknown>>;

    const results = await Promise.all(fns.map((fn) => fn()));
    for (const result of results) {
      expectUnavailable(result);
    }
  });
});

// ---------------------------------------------------------------------------
// loadNativeBindings — integration (will use degraded path in test env)
// ---------------------------------------------------------------------------

describe("loadNativeBindings", () => {
  it("returns bindings and nativeAvailable flag", () => {
    const { bindings, nativeAvailable } = loadNativeBindings();
    expect(bindings).toBeDefined();
    expect(typeof nativeAvailable).toBe("boolean");
  });

  it("returns working bindings even when native is unavailable", async () => {
    const { bindings } = loadNativeBindings();
    // Since @aix/core-* packages are not installed in the test env, this
    // should fall back to degraded mode.
    const result = await bindings.napiDispatch("HaltCheck", {});
    // Either a real DispatchResult or an UnavailableResult is fine.
    expect(result).toBeDefined();
    expect(typeof result).toBe("object");
  });

  it("degraded bindings do not throw on any function call", async () => {
    const { bindings, nativeAvailable } = loadNativeBindings();
    if (!nativeAvailable) {
      // Verify every single function resolves without throwing.
      const calls = [
        bindings.napiDispatch("HaltCheck", {}),
        bindings.napiSonaQuery("test"),
        bindings.napiRvfSeal(Buffer.from("test")),
        bindings.napiAgentSpawn("worker", {}),
        bindings.napiSwarmStatus(),
        bindings.napiRoute(new Float32Array([0.1])),
        bindings.napiVoicePipelineCreate({
          language: "en",
          vadEnabled: true,
          sttTier: "small",
          ttsPersona: "Finance",
          maxIntents: 3,
        }),
        bindings.napiVoiceVadProcess(0.5, 0.3, 100),
        bindings.napiVoiceTranscribe(16000, "en", "small"),
        bindings.napiVoiceDecomposeIntents("pay bills and see doctor", 5),
        bindings.napiVoiceSynthesize("hello", "Finance", false),
        bindings.napiVoiceSessionCreate(),
        bindings.napiPhoneRuntimeCreate("device-1", {
          hasMicrophone: true,
          hasHaptics: true,
          backgroundProcessing: true,
          ramMb: 4096,
          cpuCores: 8,
        }),
        bindings.napiPhoneEngagementScore(),
        bindings.napiPhoneSavingsRecord(500, "Finance", "proof-abc"),
        bindings.napiPhoneStreakCheckIn(),
        bindings.napiPhoneStreakStatus(),
        bindings.napiPhoneBatteryPolicy(85, "nominal"),
        bindings.napiPhoneCoordinatorStatus(),
        bindings.napiPhoneNotificationSend("Title", "Body", "actionable"),
        bindings.napiSonaRecord("q", ["act"], 0.8),
        bindings.napiSonaAdapt(new Float32Array([0.1, 0.2]), 0.9),
        bindings.napiVoicePatternSearch("pattern", 5),
        bindings.napiFederatedAnonymize('{"field":"value"}'),
        bindings.napiFatigueCheck("user-1"),
        bindings.napiRvfVerify("witness-1"),
        bindings.napiInferenceLoad("/models/test.gguf", "small"),
        bindings.napiInferenceGenerate("Hello world", 50),
        bindings.napiInferenceStatus(),
        bindings.napiInferenceTieredRoute("complex query", 0.4),
        bindings.napiInferenceUnload("model-1"),
      ];

      const results = await Promise.all(calls);
      expect(results).toHaveLength(31);
      for (const r of results) {
        expectUnavailable(r);
      }
    }
  });
});
