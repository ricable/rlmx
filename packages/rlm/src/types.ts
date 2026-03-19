/**
 * Configuration for the vLLM client.
 *
 * Maps 1:1 to the Rust VllmConfig struct in rlmx-rlm/src/vllm.rs.
 */
export interface VllmConfig {
  /** Base URL of the vLLM server (e.g. "http://localhost:8000"). */
  endpoint: string;
  /** Optional Bearer token for API authentication. */
  apiKey?: string;
  /** Model name to use for inference requests. */
  model: string;
  /** Maximum number of retry attempts for transient failures. Default: 3. */
  maxRetries?: number;
  /** Base delay in milliseconds for exponential backoff retries. Default: 500. */
  baseRetryDelayMs?: number;
  /** Request timeout in milliseconds. Default: 30000. */
  timeoutMs?: number;
}

/**
 * A message in the OpenAI-compatible chat completion API.
 */
export interface ChatMessage {
  role: 'system' | 'user' | 'assistant';
  content: string;
}

/**
 * Response format specification for structured output.
 */
export interface ResponseFormat {
  type: 'json_object' | 'text';
}

/**
 * Parameters for a chat completion request.
 */
export interface CompletionRequest {
  /** Chat messages forming the conversation. */
  messages: ChatMessage[];
  /** Sampling temperature (0.0 - 2.0). */
  temperature?: number;
  /** Maximum tokens to generate. */
  maxTokens?: number;
  /** Response format for structured output. */
  responseFormat?: ResponseFormat;
  /** Stop sequences. */
  stop?: string[];
  /** Top-p nucleus sampling. */
  topP?: number;
  /** Frequency penalty (-2.0 to 2.0). */
  frequencyPenalty?: number;
  /** Presence penalty (-2.0 to 2.0). */
  presencePenalty?: number;
}

/**
 * Token usage information from a completion response.
 */
export interface UsageInfo {
  promptTokens: number;
  completionTokens: number;
  totalTokens: number;
}

/**
 * A single choice in a chat completion response.
 */
export interface CompletionChoice {
  index: number;
  message: ChatMessage;
  finishReason: string | null;
}

/**
 * Full chat completion response from the vLLM server.
 */
export interface CompletionResponse {
  id: string;
  object: string;
  created: number;
  model: string;
  choices: CompletionChoice[];
  usage?: UsageInfo;
}

/**
 * A single chunk in a streamed completion response (SSE).
 */
export interface StreamChunk {
  id: string;
  object: string;
  created: number;
  model: string;
  choices: StreamChoice[];
}

/**
 * A single choice delta in a streamed response chunk.
 */
export interface StreamChoice {
  index: number;
  delta: {
    role?: string;
    content?: string;
  };
  finishReason: string | null;
}

/**
 * Batch completion request tuple: messages, optional temperature, optional maxTokens.
 */
export type BatchRequest = [
  messages: ChatMessage[],
  temperature?: number,
  maxTokens?: number,
];

/**
 * Wire format types matching the vLLM OpenAI-compatible JSON schema.
 * These are used internally for serialization/deserialization.
 * @internal
 */
export interface WireCompletionRequest {
  model: string;
  messages: ChatMessage[];
  temperature?: number;
  max_tokens?: number;
  response_format?: ResponseFormat;
  stop?: string[];
  top_p?: number;
  frequency_penalty?: number;
  presence_penalty?: number;
  stream?: boolean;
}

/** @internal */
export interface WireCompletionResponse {
  id: string;
  object: string;
  created: number;
  model: string;
  choices: WireCompletionChoice[];
  usage?: WireUsageInfo;
}

/** @internal */
export interface WireCompletionChoice {
  index: number;
  message: { role: string; content: string | null };
  finish_reason: string | null;
}

/** @internal */
export interface WireUsageInfo {
  prompt_tokens: number;
  completion_tokens: number;
  total_tokens: number;
}

/** @internal */
export interface WireStreamChunk {
  id: string;
  object: string;
  created: number;
  model: string;
  choices: WireStreamChoice[];
}

/** @internal */
export interface WireStreamChoice {
  index: number;
  delta: { role?: string; content?: string };
  finish_reason: string | null;
}
