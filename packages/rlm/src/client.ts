import {
  VllmApiError,
  VllmEmptyResponseError,
  VllmHealthCheckError,
  VllmMaxRetriesError,
  VllmParseError,
  VllmRequestError,
  VllmTimeoutError,
} from './errors.js';
import type {
  BatchRequest,
  ChatMessage,
  CompletionChoice,
  CompletionRequest,
  CompletionResponse,
  ResponseFormat,
  StreamChunk,
  UsageInfo,
  VllmConfig,
  WireCompletionRequest,
  WireCompletionResponse,
  WireStreamChunk,
} from './types.js';

/** Default configuration values matching the Rust VllmConfig::default(). */
const DEFAULTS = {
  maxRetries: 3,
  baseRetryDelayMs: 500,
  timeoutMs: 30_000,
} as const;

/**
 * OpenAI-compatible HTTP client for vLLM inference servers.
 *
 * Port of the Rust VllmClient in rlmx-rlm/src/vllm.rs.
 * Uses native `fetch` for HTTP requests -- no external HTTP dependencies.
 *
 * Features:
 * - Chat completion API with structured JSON output support
 * - Streaming (SSE) completions via async generator
 * - Batch parallel completions for sub-agent execution
 * - Exponential backoff retry on transient failures (5xx / 429)
 * - Configurable timeout via AbortController
 * - Health check endpoint
 */
export class VllmClient {
  private readonly endpoint: string;
  private readonly apiKey: string | undefined;
  private readonly model: string;
  private readonly maxRetries: number;
  private readonly baseRetryDelayMs: number;
  private readonly timeoutMs: number;

  constructor(config: VllmConfig) {
    this.endpoint = config.endpoint.replace(/\/+$/, '');
    this.apiKey = config.apiKey;
    this.model = config.model;
    this.maxRetries = config.maxRetries ?? DEFAULTS.maxRetries;
    this.baseRetryDelayMs = config.baseRetryDelayMs ?? DEFAULTS.baseRetryDelayMs;
    this.timeoutMs = config.timeoutMs ?? DEFAULTS.timeoutMs;
  }

  /**
   * Create a client with just an endpoint URL and model name.
   * Convenience factory matching the Rust `VllmClient::with_endpoint`.
   */
  static withEndpoint(endpoint: string, model: string): VllmClient {
    return new VllmClient({ endpoint, model });
  }

  /** Get the configured endpoint URL (trailing slash stripped). */
  getEndpoint(): string {
    return this.endpoint;
  }

  /** Get the configured model name. */
  getModel(): string {
    return this.model;
  }

  // ---------------------------------------------------------------------------
  // URL & header helpers
  // ---------------------------------------------------------------------------

  /** Build the full URL for a given API path. */
  private apiUrl(path: string): string {
    return `${this.endpoint}${path}`;
  }

  /** Build common headers for requests. */
  private headers(): Record<string, string> {
    const h: Record<string, string> = { 'Content-Type': 'application/json' };
    if (this.apiKey) {
      h['Authorization'] = `Bearer ${this.apiKey}`;
    }
    return h;
  }

  // ---------------------------------------------------------------------------
  // Wire format conversion
  // ---------------------------------------------------------------------------

  /** Convert public CompletionRequest to wire format. */
  private toWireRequest(
    req: CompletionRequest,
    stream = false,
  ): WireCompletionRequest {
    const wire: WireCompletionRequest = {
      model: this.model,
      messages: req.messages,
    };
    if (req.temperature !== undefined) wire.temperature = req.temperature;
    if (req.maxTokens !== undefined) wire.max_tokens = req.maxTokens;
    if (req.responseFormat !== undefined) wire.response_format = req.responseFormat;
    if (req.stop !== undefined) wire.stop = req.stop;
    if (req.topP !== undefined) wire.top_p = req.topP;
    if (req.frequencyPenalty !== undefined) wire.frequency_penalty = req.frequencyPenalty;
    if (req.presencePenalty !== undefined) wire.presence_penalty = req.presencePenalty;
    if (stream) wire.stream = true;
    return wire;
  }

  /** Convert wire response to public CompletionResponse. */
  private fromWireResponse(wire: WireCompletionResponse): CompletionResponse {
    return {
      id: wire.id,
      object: wire.object,
      created: wire.created,
      model: wire.model,
      choices: wire.choices.map(
        (c): CompletionChoice => ({
          index: c.index,
          message: {
            role: c.message.role as ChatMessage['role'],
            content: c.message.content ?? '',
          },
          finishReason: c.finish_reason,
        }),
      ),
      usage: wire.usage
        ? {
            promptTokens: wire.usage.prompt_tokens,
            completionTokens: wire.usage.completion_tokens,
            totalTokens: wire.usage.total_tokens,
          }
        : undefined,
    };
  }

  /** Convert a wire stream chunk to the public StreamChunk shape. */
  private fromWireStreamChunk(wire: WireStreamChunk): StreamChunk {
    return {
      id: wire.id,
      object: wire.object,
      created: wire.created,
      model: wire.model,
      choices: wire.choices.map((c) => ({
        index: c.index,
        delta: { role: c.delta.role, content: c.delta.content },
        finishReason: c.finish_reason,
      })),
    };
  }

  // ---------------------------------------------------------------------------
  // Core request with retry
  // ---------------------------------------------------------------------------

  /**
   * Execute a fetch request with timeout and retry logic.
   * Retries on network errors, 5xx, and 429 (rate limit).
   * Does NOT retry on 4xx (except 429).
   */
  private async fetchWithRetry(
    url: string,
    init: RequestInit,
  ): Promise<Response> {
    let lastError: Error | undefined;

    for (let attempt = 0; attempt <= this.maxRetries; attempt++) {
      if (attempt > 0) {
        const delayMs = this.baseRetryDelayMs * Math.pow(2, attempt - 1);
        await sleep(delayMs);
      }

      const controller = new AbortController();
      const timeout = setTimeout(() => controller.abort(), this.timeoutMs);

      try {
        const response = await fetch(url, {
          ...init,
          signal: controller.signal,
        });

        clearTimeout(timeout);

        if (response.ok) {
          return response;
        }

        const statusCode = response.status;
        const errorBody = await response.text().catch(() => '');

        // Do not retry client errors (4xx) except 429 (rate limit)
        if (statusCode >= 400 && statusCode < 500 && statusCode !== 429) {
          throw new VllmApiError(statusCode, errorBody);
        }

        // Transient error -- retry
        lastError = new VllmApiError(statusCode, errorBody);
      } catch (err) {
        clearTimeout(timeout);

        // Propagate non-retryable API errors immediately
        if (err instanceof VllmApiError) {
          throw err;
        }

        if (err instanceof DOMException && err.name === 'AbortError') {
          lastError = new VllmTimeoutError(this.timeoutMs);
        } else {
          lastError = new VllmRequestError(
            (err as Error).message,
            err as Error,
          );
        }
      }
    }

    // All retries exhausted
    throw lastError ?? new VllmMaxRetriesError(this.maxRetries);
  }

  // ---------------------------------------------------------------------------
  // Public API
  // ---------------------------------------------------------------------------

  /**
   * Send a chat completion request to the vLLM server.
   *
   * Returns the full parsed CompletionResponse including usage info.
   * Implements retry logic with exponential backoff on transient failures.
   */
  async complete(request: CompletionRequest): Promise<CompletionResponse> {
    const url = this.apiUrl('/v1/chat/completions');
    const body = JSON.stringify(this.toWireRequest(request));

    const response = await this.fetchWithRetry(url, {
      method: 'POST',
      headers: this.headers(),
      body,
    });

    let wire: WireCompletionResponse;
    try {
      wire = (await response.json()) as WireCompletionResponse;
    } catch (err) {
      throw new VllmParseError((err as Error).message);
    }

    return this.fromWireResponse(wire);
  }

  /**
   * Send a chat completion request and return just the content string.
   *
   * Convenience method matching the Rust `chat_completion` which returns
   * `Result<String, VllmError>`. Throws VllmEmptyResponseError if no
   * choices are returned.
   */
  async chatCompletion(
    messages: ChatMessage[],
    temperature?: number,
    maxTokens?: number,
    responseFormat?: ResponseFormat,
  ): Promise<string> {
    const response = await this.complete({
      messages,
      temperature,
      maxTokens,
      responseFormat,
    });

    const firstChoice = response.choices[0];
    if (!firstChoice || !firstChoice.message.content) {
      throw new VllmEmptyResponseError();
    }

    return firstChoice.message.content;
  }

  /**
   * Send multiple chat completion requests in parallel for sub-agent execution.
   *
   * Returns results in the same order as the input requests.
   * Each request uses JSON response format by default (matching Rust behavior).
   */
  async batchCompletion(requests: BatchRequest[]): Promise<string[]> {
    const promises = requests.map(([messages, temperature, maxTokens]) =>
      this.chatCompletion(messages, temperature, maxTokens, { type: 'json_object' }),
    );
    return Promise.all(promises);
  }

  /**
   * Stream a chat completion response using Server-Sent Events (SSE).
   *
   * Yields StreamChunk objects as they arrive. Each chunk contains a delta
   * with partial content. Concatenate `chunk.choices[0].delta.content` to
   * build the full response.
   */
  async *streamCompletion(
    request: CompletionRequest,
  ): AsyncGenerator<StreamChunk, void, undefined> {
    const url = this.apiUrl('/v1/chat/completions');
    const body = JSON.stringify(this.toWireRequest(request, true));

    const response = await this.fetchWithRetry(url, {
      method: 'POST',
      headers: this.headers(),
      body,
    });

    if (!response.body) {
      throw new VllmParseError('Response body is null (streaming not supported)');
    }

    yield* this.parseSSEStream(response.body);
  }

  /**
   * Stream a completion and collect the full content string.
   *
   * Convenience wrapper around streamCompletion that concatenates all
   * content deltas into a single string.
   */
  async streamChatCompletion(
    messages: ChatMessage[],
    temperature?: number,
    maxTokens?: number,
    responseFormat?: ResponseFormat,
    onChunk?: (chunk: StreamChunk) => void,
  ): Promise<string> {
    const parts: string[] = [];

    for await (const chunk of this.streamCompletion({
      messages,
      temperature,
      maxTokens,
      responseFormat,
    })) {
      if (onChunk) onChunk(chunk);
      const content = chunk.choices[0]?.delta?.content;
      if (content) parts.push(content);
    }

    return parts.join('');
  }

  /**
   * Check if the vLLM server is healthy and responding.
   * Matches the Rust `health_check` method.
   */
  async healthCheck(): Promise<void> {
    const url = this.apiUrl('/health');
    const controller = new AbortController();
    const timeout = setTimeout(() => controller.abort(), this.timeoutMs);

    try {
      const response = await fetch(url, {
        method: 'GET',
        signal: controller.signal,
      });

      clearTimeout(timeout);

      if (!response.ok) {
        throw new VllmHealthCheckError(`server returned status ${response.status}`);
      }
    } catch (err) {
      clearTimeout(timeout);
      if (err instanceof VllmHealthCheckError) throw err;
      throw new VllmHealthCheckError((err as Error).message);
    }
  }

  // ---------------------------------------------------------------------------
  // Static message builder helpers (matching Rust convenience methods)
  // ---------------------------------------------------------------------------

  /** Build a messages list from system prompt and user query. */
  static buildMessages(systemPrompt: string, userQuery: string): ChatMessage[] {
    return [
      { role: 'system', content: systemPrompt },
      { role: 'user', content: userQuery },
    ];
  }

  /** Build a messages list with context segments included. */
  static buildMessagesWithContext(
    systemPrompt: string,
    context: string,
    userQuery: string,
  ): ChatMessage[] {
    return [
      { role: 'system', content: systemPrompt },
      { role: 'user', content: `Context:\n${context}\n\nQuery: ${userQuery}` },
    ];
  }

  // ---------------------------------------------------------------------------
  // SSE stream parser
  // ---------------------------------------------------------------------------

  /** Parse an SSE byte stream into StreamChunk objects. */
  private async *parseSSEStream(
    body: ReadableStream<Uint8Array>,
  ): AsyncGenerator<StreamChunk, void, undefined> {
    const reader = body.getReader();
    const decoder = new TextDecoder();
    let buffer = '';

    try {
      while (true) {
        const { done, value } = await reader.read();
        if (done) break;

        buffer += decoder.decode(value, { stream: true });
        const lines = buffer.split('\n');
        // Keep the last incomplete line in the buffer
        buffer = lines.pop() ?? '';

        for (const line of lines) {
          const trimmed = line.trim();
          if (!trimmed || trimmed.startsWith(':')) continue;
          if (trimmed === 'data: [DONE]') return;

          if (trimmed.startsWith('data: ')) {
            const jsonStr = trimmed.slice(6);
            try {
              const wire = JSON.parse(jsonStr) as WireStreamChunk;
              yield this.fromWireStreamChunk(wire);
            } catch {
              // Skip malformed SSE data lines
            }
          }
        }
      }
    } finally {
      reader.releaseLock();
    }
  }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}
