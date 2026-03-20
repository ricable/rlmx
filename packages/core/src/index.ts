// ---------------------------------------------------------------------------
// @aix/core — NAPI bridge to RLMX Rust kernel
//
// Platform detection + dynamic loading of platform-specific .node binaries.
// When the native module is unavailable (wrong platform, binary not built,
// optional dependency not installed), all exports resolve with
// { status: 'unavailable', reason: '...' } so consumers can operate in
// degraded mode without try/catch.
// ---------------------------------------------------------------------------

export type {
  // Shared / Kernel
  UnavailableResult,
  JsonRecord,
  DispatchResult,
  PatternMatch,
  ProofResult,
  AgentHandle,
  SwarmHealth,
  RoutingDecision,
  // Voice
  VoicePipelineConfig,
  VadDecision,
  TranscriptResult,
  Intent,
  TtsResult,
  // Phone
  DeviceCapabilities,
  EngagementScore,
  SavingsResult,
  StreakState,
  BatteryPolicy,
  CoordinatorStatus,
  NotificationResult,
  // Cognitive
  AdaptResult,
  PatternResult,
  FatigueResult,
  // RVF
  VerifyResult,
  // Inference
  GenerateResult,
  InferenceStatus,
  RouteDecision,
  // Binding interface
  NativeBindings,
} from "./types.js";

export {
  detectPlatform,
  getPackageName,
  loadNativeBindings,
  createDegradedBindings,
} from "./loader.js";

import { loadNativeBindings } from "./loader.js";

// ---------------------------------------------------------------------------
// Eagerly load bindings at module evaluation time. The result is cached so
// subsequent imports reuse the same instance.
// ---------------------------------------------------------------------------

const { bindings, nativeAvailable } = loadNativeBindings();

/** Whether the native Rust binary was successfully loaded. */
export const isNativeAvailable: boolean = nativeAvailable;

// ---------------------------------------------------------------------------
// Re-export all 31 NAPI functions as top-level named exports.
// Callers can `import { napiDispatch } from '@aix/core'` directly.
// ---------------------------------------------------------------------------

// -- Existing (6) -----------------------------------------------------------

export const napiDispatch = bindings.napiDispatch.bind(bindings);
export const napiSonaQuery = bindings.napiSonaQuery.bind(bindings);
export const napiRvfSeal = bindings.napiRvfSeal.bind(bindings);
export const napiAgentSpawn = bindings.napiAgentSpawn.bind(bindings);
export const napiSwarmStatus = bindings.napiSwarmStatus.bind(bindings);
export const napiRoute = bindings.napiRoute.bind(bindings);

// -- Voice (6) --------------------------------------------------------------

export const napiVoicePipelineCreate =
  bindings.napiVoicePipelineCreate.bind(bindings);
export const napiVoiceVadProcess =
  bindings.napiVoiceVadProcess.bind(bindings);
export const napiVoiceTranscribe =
  bindings.napiVoiceTranscribe.bind(bindings);
export const napiVoiceDecomposeIntents =
  bindings.napiVoiceDecomposeIntents.bind(bindings);
export const napiVoiceSynthesize =
  bindings.napiVoiceSynthesize.bind(bindings);
export const napiVoiceSessionCreate =
  bindings.napiVoiceSessionCreate.bind(bindings);

// -- Phone (8) --------------------------------------------------------------

export const napiPhoneRuntimeCreate =
  bindings.napiPhoneRuntimeCreate.bind(bindings);
export const napiPhoneEngagementScore =
  bindings.napiPhoneEngagementScore.bind(bindings);
export const napiPhoneSavingsRecord =
  bindings.napiPhoneSavingsRecord.bind(bindings);
export const napiPhoneStreakCheckIn =
  bindings.napiPhoneStreakCheckIn.bind(bindings);
export const napiPhoneStreakStatus =
  bindings.napiPhoneStreakStatus.bind(bindings);
export const napiPhoneBatteryPolicy =
  bindings.napiPhoneBatteryPolicy.bind(bindings);
export const napiPhoneCoordinatorStatus =
  bindings.napiPhoneCoordinatorStatus.bind(bindings);
export const napiPhoneNotificationSend =
  bindings.napiPhoneNotificationSend.bind(bindings);

// -- Cognitive (5) ----------------------------------------------------------

export const napiSonaRecord = bindings.napiSonaRecord.bind(bindings);
export const napiSonaAdapt = bindings.napiSonaAdapt.bind(bindings);
export const napiVoicePatternSearch =
  bindings.napiVoicePatternSearch.bind(bindings);
export const napiFederatedAnonymize =
  bindings.napiFederatedAnonymize.bind(bindings);
export const napiFatigueCheck = bindings.napiFatigueCheck.bind(bindings);

// -- RVF (1) ----------------------------------------------------------------

export const napiRvfVerify = bindings.napiRvfVerify.bind(bindings);

// -- Inference (5) ----------------------------------------------------------

export const napiInferenceLoad =
  bindings.napiInferenceLoad.bind(bindings);
export const napiInferenceGenerate =
  bindings.napiInferenceGenerate.bind(bindings);
export const napiInferenceStatus =
  bindings.napiInferenceStatus.bind(bindings);
export const napiInferenceTieredRoute =
  bindings.napiInferenceTieredRoute.bind(bindings);
export const napiInferenceUnload =
  bindings.napiInferenceUnload.bind(bindings);
