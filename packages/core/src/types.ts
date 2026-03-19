// ---------------------------------------------------------------------------
// @aix/core — TypeScript type declarations for all NAPI bridge functions
//
// Groups:
//   Existing (6)  — kernel syscall dispatch, SONA, RVF, agents, swarm, routing
//   Voice (6)     — voice pipeline, VAD, STT, intent decomposition, TTS, sessions
//   Phone (8)     — phone runtime, engagement, savings, streaks, battery, notifications
//   Cognitive (5) — SONA recording/adaptation, voice patterns, anonymization, fatigue
//   RVF (1)       — proof verification
//   Inference (5) — model loading, generation, tiered routing, status, unload
// ---------------------------------------------------------------------------

// ---- Shared / Kernel types ------------------------------------------------

/** Result returned when native module is unavailable (graceful degradation). */
export interface UnavailableResult {
  status: "unavailable";
  reason: string;
}

/** Generic record for dynamic syscall params/results. */
export type JsonRecord = Record<string, unknown>;

/** Dispatch result from a kernel syscall. */
export interface DispatchResult {
  syscall_type: string;
  success: boolean;
  payload: JsonRecord;
}

/** SONA pattern match result. */
export interface PatternMatch {
  pattern_id: string;
  score: number;
  content: string;
  metadata: JsonRecord;
}

/** RVF proof seal result. */
export interface ProofResult {
  witness_id: string;
  valid: boolean;
  confidence: number;
  reason: string;
}

/** Handle to a spawned agent process. */
export interface AgentHandle {
  process_id: string;
  agent_type: string;
  permissions: string[];
  status: string;
}

/** Swarm health snapshot. */
export interface SwarmHealth {
  active_nodes: number;
  zone_health: Record<string, number>;
  consensus_protocol: string;
  avg_latency_ms: number;
  healthy: boolean;
}

/** TinyDancerRouter routing decision. */
export interface RoutingDecision {
  strategy: string;
  confidence: number;
  scores: Record<string, number>;
  latency_ns: number;
}

// ---- Voice types ----------------------------------------------------------

/** Configuration for creating a voice pipeline. */
export interface VoicePipelineConfig {
  /** Language code (e.g. "en", "fr"). */
  language: string;
  /** Whether to enable VAD. */
  vadEnabled: boolean;
  /** STT engine tier: "small" | "medium" | "remote". */
  sttTier: string;
  /** TTS persona name (e.g. "Finance", "Health"). */
  ttsPersona: string;
  /** Maximum number of intents to decompose from a single utterance. */
  maxIntents: number;
}

/** Voice activity detection decision. */
export interface VadDecision {
  /** Whether speech was detected. */
  isSpeech: boolean;
  /** Confidence of the decision [0.0, 1.0]. */
  confidence: number;
  /** Duration of the analyzed segment in milliseconds. */
  durationMs: number;
}

/** Speech-to-text transcription result. */
export interface TranscriptResult {
  /** Transcribed text. */
  text: string;
  /** Language detected or used. */
  language: string;
  /** Confidence of the transcription [0.0, 1.0]. */
  confidence: number;
  /** Processing latency in milliseconds. */
  latencyMs: number;
}

/** A single decomposed intent from a multi-intent utterance. */
export interface Intent {
  /** Life domain (Finance, Health, Legal, etc.). */
  domain: string;
  /** The action to take. */
  action: string;
  /** Confidence of this intent classification [0.0, 1.0]. */
  confidence: number;
  /** Original text span that produced this intent. */
  span: string;
}

/** Text-to-speech synthesis result. */
export interface TtsResult {
  /** Audio data length in bytes. */
  audioLenBytes: number;
  /** Sample rate of the output audio. */
  sampleRate: number;
  /** Duration of synthesized speech in milliseconds. */
  durationMs: number;
  /** Persona used for synthesis. */
  persona: string;
}

// ---- Phone types ----------------------------------------------------------

/** Device capabilities for phone runtime initialization. */
export interface DeviceCapabilities {
  /** Whether the device has a microphone. */
  hasMicrophone: boolean;
  /** Whether the device supports haptic feedback. */
  hasHaptics: boolean;
  /** Whether background processing is available. */
  backgroundProcessing: boolean;
  /** Total RAM in megabytes. */
  ramMb: number;
  /** Number of CPU cores. */
  cpuCores: number;
}

/** Engagement score (Life Score). */
export interface EngagementScore {
  /** Composite life score [30, 100]. Floor is 30. */
  lifeScore: number;
  /** Per-domain scores keyed by LifeDomain name. */
  domainScores: Record<string, number>;
  /** ISO 8601 timestamp of last update. */
  lastUpdated: string;
}

/** Result of recording a savings event. */
export interface SavingsResult {
  /** Total savings in cents. */
  totalCents: number;
  /** Annualized projected savings in cents. */
  annualizedCents: number;
  /** Number of verified savings events. */
  eventCount: number;
}

/** Streak state. */
export interface StreakState {
  /** Current streak length in days. */
  currentDays: number;
  /** Longest streak ever achieved. */
  longestDays: number;
  /** Whether a freeze is active. */
  freezeActive: boolean;
  /** Days remaining on freeze (0 if not active). */
  freezeDaysRemaining: number;
  /** ISO 8601 timestamp of last check-in. */
  lastCheckIn: string;
}

/** Battery-aware scheduling policy. */
export interface BatteryPolicy {
  /** Recommended scheduling mode: "normal" | "low_power" | "critical". */
  mode: string;
  /** Whether cloud burst is allowed. */
  cloudBurstAllowed: boolean;
  /** Whether background agents should be paused. */
  pauseBackground: boolean;
  /** Maximum concurrent agents under this policy. */
  maxConcurrentAgents: number;
}

/** Lightweight coordinator status. */
export interface CoordinatorStatus {
  /** Number of active free agents (max 5). */
  activeAgents: number;
  /** Agent names and their current states. */
  agents: Array<{ name: string; status: string }>;
  /** Whether the coordinator is in offline mode. */
  offlineMode: boolean;
  /** Number of items in the offline outbox. */
  outboxSize: number;
}

/** Notification delivery result. */
export interface NotificationResult {
  /** Whether the notification was delivered. */
  delivered: boolean;
  /** Notification ID. */
  notificationId: string;
  /** Priority tier used: "critical" | "actionable" | "informational". */
  priority: string;
  /** Whether fatigue model suppressed the notification. */
  suppressed: boolean;
}

// ---- Cognitive types ------------------------------------------------------

/** Result of SONA micro-LoRA adaptation. */
export interface AdaptResult {
  /** Whether adaptation was applied. */
  adapted: boolean;
  /** Number of parameters updated. */
  paramsUpdated: number;
  /** EWC++ regularization loss. */
  ewcLoss: number;
}

/** Voice pattern search result. */
export interface PatternResult {
  /** Pattern identifier. */
  patternId: string;
  /** Similarity score [0.0, 1.0]. */
  score: number;
  /** Pattern content summary. */
  content: string;
  /** Life domain of the pattern. */
  domain: string;
  /** Temporal weight applied. */
  temporalWeight: number;
}

/** Notification fatigue check result. */
export interface FatigueResult {
  /** Whether the user is fatigued. */
  fatigued: boolean;
  /** Current fatigue score [0.0, 1.0]. */
  fatigueScore: number;
  /** Recommended cooldown in seconds (0 if not fatigued). */
  cooldownSeconds: number;
}

// ---- RVF types ------------------------------------------------------------

/** Proof verification result. */
export interface VerifyResult {
  /** Whether the proof is valid. */
  valid: boolean;
  /** Witness ID that was verified. */
  witnessId: string;
  /** Chain position of the witness. */
  chainPosition: number;
  /** Verification confidence [0.0, 1.0]. */
  confidence: number;
}

// ---- Inference types ------------------------------------------------------

/** Inference generation result. */
export interface GenerateResult {
  /** Generated text. */
  text: string;
  /** Number of tokens generated. */
  tokensGenerated: number;
  /** Tokens per second. */
  tokensPerSecond: number;
  /** Model tier used: "small" | "medium" | "remote". */
  tier: string;
  /** Whether the result was escalated from a lower tier. */
  escalated: boolean;
}

/** Inference engine status. */
export interface InferenceStatus {
  /** Whether an engine is loaded and ready. */
  ready: boolean;
  /** Currently loaded model identifier, if any. */
  modelId: string | null;
  /** Model tier: "small" | "medium" | "remote" | null. */
  tier: string | null;
  /** Memory usage in megabytes. */
  memoryMb: number;
  /** Backend in use: "candle" | "mlx" | "vllm" | null. */
  backend: string | null;
}

/** Tiered routing decision for inference. */
export interface RouteDecision {
  /** Selected tier: "small" | "medium" | "remote". */
  tier: string;
  /** Confidence that the selected tier can handle the query. */
  confidence: number;
  /** Whether escalation from a lower tier is recommended. */
  shouldEscalate: boolean;
  /** Reason for the routing decision. */
  reason: string;
}

// ---- Native module interface ----------------------------------------------

/**
 * The full set of NAPI functions exposed by the native Rust module.
 * When the native binary is unavailable, a degraded proxy implementing
 * this interface returns `{ status: 'unavailable' }` for every call.
 */
export interface NativeBindings {
  // -- Existing (6) --
  napiDispatch(
    syscall: string,
    args: JsonRecord,
  ): Promise<DispatchResult | UnavailableResult>;
  napiSonaQuery(
    query: string,
  ): Promise<JsonRecord | UnavailableResult>;
  napiRvfSeal(
    data: Buffer,
  ): Promise<JsonRecord | UnavailableResult>;
  napiAgentSpawn(
    agentType: string,
    config: JsonRecord,
  ): Promise<JsonRecord | UnavailableResult>;
  napiSwarmStatus(): Promise<JsonRecord | UnavailableResult>;
  napiRoute(
    input: Float32Array,
  ): Promise<JsonRecord | UnavailableResult>;

  // -- Voice (6) --
  napiVoicePipelineCreate(
    config: VoicePipelineConfig,
  ): Promise<string | UnavailableResult>;
  napiVoiceVadProcess(
    energy: number,
    spectral: number,
    durationMs: number,
  ): Promise<VadDecision | UnavailableResult>;
  napiVoiceTranscribe(
    audioLen: number,
    language: string,
    tier: string,
  ): Promise<TranscriptResult | UnavailableResult>;
  napiVoiceDecomposeIntents(
    transcript: string,
    maxIntents: number,
  ): Promise<Intent[] | UnavailableResult>;
  napiVoiceSynthesize(
    text: string,
    persona: string,
    streaming: boolean,
  ): Promise<TtsResult | UnavailableResult>;
  napiVoiceSessionCreate(): Promise<string | UnavailableResult>;

  // -- Phone (8) --
  napiPhoneRuntimeCreate(
    deviceId: string,
    capabilities: DeviceCapabilities,
  ): Promise<string | UnavailableResult>;
  napiPhoneEngagementScore(): Promise<EngagementScore | UnavailableResult>;
  napiPhoneSavingsRecord(
    cents: number,
    domain: string,
    proof: string,
  ): Promise<SavingsResult | UnavailableResult>;
  napiPhoneStreakCheckIn(): Promise<StreakState | UnavailableResult>;
  napiPhoneStreakStatus(): Promise<StreakState | UnavailableResult>;
  napiPhoneBatteryPolicy(
    level: number,
    thermal: string,
  ): Promise<BatteryPolicy | UnavailableResult>;
  napiPhoneCoordinatorStatus(): Promise<CoordinatorStatus | UnavailableResult>;
  napiPhoneNotificationSend(
    title: string,
    body: string,
    priority: string,
  ): Promise<NotificationResult | UnavailableResult>;

  // -- Cognitive (5) --
  napiSonaRecord(
    query: string,
    actions: string[],
    quality: number,
  ): Promise<void | UnavailableResult>;
  napiSonaAdapt(
    embedding: Float32Array,
    quality: number,
  ): Promise<AdaptResult | UnavailableResult>;
  napiVoicePatternSearch(
    query: string,
    topK: number,
  ): Promise<PatternResult[] | UnavailableResult>;
  napiFederatedAnonymize(
    patternJson: string,
  ): Promise<string | UnavailableResult>;
  napiFatigueCheck(
    userId: string,
  ): Promise<FatigueResult | UnavailableResult>;

  // -- RVF (1) --
  napiRvfVerify(
    witnessId: string,
  ): Promise<VerifyResult | UnavailableResult>;

  // -- Inference (5) --
  napiInferenceLoad(
    modelPath: string,
    tier: string,
  ): Promise<string | UnavailableResult>;
  napiInferenceGenerate(
    prompt: string,
    maxTokens: number,
  ): Promise<GenerateResult | UnavailableResult>;
  napiInferenceStatus(): Promise<InferenceStatus | UnavailableResult>;
  napiInferenceTieredRoute(
    query: string,
    confidence: number,
  ): Promise<RouteDecision | UnavailableResult>;
  napiInferenceUnload(
    modelId: string,
  ): Promise<void | UnavailableResult>;
}
