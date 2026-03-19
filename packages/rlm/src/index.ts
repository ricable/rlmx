/**
 * @aix/rlm - vLLM HTTP client for the @aix ecosystem.
 *
 * Port of the Rust rlmx-rlm crate's vLLM client module.
 * Provides a fetch-based OpenAI-compatible HTTP client for vLLM inference servers.
 *
 * @example
 * ```ts
 * import { VllmClient } from '@aix/rlm';
 *
 * const client = VllmClient.withEndpoint('http://localhost:8000', 'llama-3-70b');
 * const answer = await client.chatCompletion(
 *   VllmClient.buildMessages('You are helpful.', 'What is Rust?'),
 *   0.7,
 *   1024,
 * );
 * ```
 */

// Client
export { VllmClient } from './client.js';

// Types
export type {
  VllmConfig,
  ChatMessage,
  ResponseFormat,
  CompletionRequest,
  CompletionResponse,
  CompletionChoice,
  UsageInfo,
  StreamChunk,
  StreamChoice,
  BatchRequest,
} from './types.js';

// Errors
export {
  VllmError,
  VllmRequestError,
  VllmApiError,
  VllmParseError,
  VllmEmptyResponseError,
  VllmMaxRetriesError,
  VllmHealthCheckError,
  VllmTimeoutError,
} from './errors.js';
