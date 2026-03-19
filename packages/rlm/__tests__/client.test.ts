import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { VllmClient } from '../src/client.js';
import {
  VllmApiError,
  VllmEmptyResponseError,
  VllmHealthCheckError,
  VllmParseError,
  VllmTimeoutError,
} from '../src/errors.js';
import type {
  ChatMessage,
  WireCompletionResponse,
  WireStreamChunk,
} from '../src/types.js';

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function makeWireResponse(
  content: string,
  finishReason = 'stop',
): WireCompletionResponse {
  return {
    id: 'chatcmpl-test',
    object: 'chat.completion',
    created: 1700000000,
    model: 'test-model',
    choices: [
      {
        index: 0,
        message: { role: 'assistant', content },
        finish_reason: finishReason,
      },
    ],
    usage: {
      prompt_tokens: 10,
      completion_tokens: 20,
      total_tokens: 30,
    },
  };
}

function jsonResponse(body: unknown, status = 200): Response {
  return new Response(JSON.stringify(body), {
    status,
    headers: { 'Content-Type': 'application/json' },
  });
}

function textResponse(body: string, status = 200): Response {
  return new Response(body, { status });
}

function sseResponse(chunks: WireStreamChunk[], includesDone = true): Response {
  let sseText = '';
  for (const chunk of chunks) {
    sseText += `data: ${JSON.stringify(chunk)}\n\n`;
  }
  if (includesDone) {
    sseText += 'data: [DONE]\n\n';
  }
  const encoder = new TextEncoder();
  const stream = new ReadableStream({
    start(controller) {
      controller.enqueue(encoder.encode(sseText));
      controller.close();
    },
  });
  return new Response(stream, {
    status: 200,
    headers: { 'Content-Type': 'text/event-stream' },
  });
}

function makeStreamChunk(
  content: string,
  finishReason: string | null = null,
): WireStreamChunk {
  return {
    id: 'chatcmpl-stream',
    object: 'chat.completion.chunk',
    created: 1700000000,
    model: 'test-model',
    choices: [
      {
        index: 0,
        delta: { content },
        finish_reason: finishReason,
      },
    ],
  };
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

describe('VllmClient', () => {
  let fetchSpy: ReturnType<typeof vi.spyOn>;

  beforeEach(() => {
    fetchSpy = vi.spyOn(globalThis, 'fetch');
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  // -------------------------------------------------------------------------
  // Construction
  // -------------------------------------------------------------------------

  describe('construction', () => {
    it('should construct with full config', () => {
      const client = new VllmClient({
        endpoint: 'http://localhost:8000',
        model: 'llama-3-70b',
        apiKey: 'sk-test',
        maxRetries: 5,
        baseRetryDelayMs: 100,
        timeoutMs: 10_000,
      });

      expect(client.getEndpoint()).toBe('http://localhost:8000');
      expect(client.getModel()).toBe('llama-3-70b');
    });

    it('should strip trailing slashes from endpoint', () => {
      const client = new VllmClient({
        endpoint: 'http://localhost:8000/',
        model: 'model',
      });
      expect(client.getEndpoint()).toBe('http://localhost:8000');
    });

    it('should create with withEndpoint factory', () => {
      const client = VllmClient.withEndpoint('http://gpu:8080', 'mixtral');
      expect(client.getEndpoint()).toBe('http://gpu:8080');
      expect(client.getModel()).toBe('mixtral');
    });
  });

  // -------------------------------------------------------------------------
  // Static message builders
  // -------------------------------------------------------------------------

  describe('message builders', () => {
    it('should build system + user messages', () => {
      const msgs = VllmClient.buildMessages('You are helpful.', 'Hello');
      expect(msgs).toEqual([
        { role: 'system', content: 'You are helpful.' },
        { role: 'user', content: 'Hello' },
      ]);
    });

    it('should build messages with context', () => {
      const msgs = VllmClient.buildMessagesWithContext(
        'System prompt',
        'Some context',
        'Analyze this',
      );
      expect(msgs).toHaveLength(2);
      expect(msgs[0].role).toBe('system');
      expect(msgs[1].content).toContain('Context:\nSome context');
      expect(msgs[1].content).toContain('Query: Analyze this');
    });
  });

  // -------------------------------------------------------------------------
  // chatCompletion
  // -------------------------------------------------------------------------

  describe('chatCompletion', () => {
    it('should return content from a successful response', async () => {
      fetchSpy.mockResolvedValueOnce(
        jsonResponse(makeWireResponse('Hello world')),
      );

      const client = VllmClient.withEndpoint('http://localhost:8000', 'test');
      const result = await client.chatCompletion(
        [{ role: 'user', content: 'Hi' }],
        0.7,
        100,
      );

      expect(result).toBe('Hello world');
      expect(fetchSpy).toHaveBeenCalledOnce();

      const [url, init] = fetchSpy.mock.calls[0];
      expect(url).toBe('http://localhost:8000/v1/chat/completions');
      expect(init?.method).toBe('POST');
      const body = JSON.parse(init?.body as string);
      expect(body.model).toBe('test');
      expect(body.messages).toEqual([{ role: 'user', content: 'Hi' }]);
      expect(body.temperature).toBe(0.7);
      expect(body.max_tokens).toBe(100);
    });

    it('should include Authorization header when apiKey is set', async () => {
      fetchSpy.mockResolvedValueOnce(
        jsonResponse(makeWireResponse('ok')),
      );

      const client = new VllmClient({
        endpoint: 'http://localhost:8000',
        model: 'test',
        apiKey: 'sk-secret',
      });
      await client.chatCompletion([{ role: 'user', content: 'Hi' }]);

      const [, init] = fetchSpy.mock.calls[0];
      const headers = init?.headers as Record<string, string>;
      expect(headers['Authorization']).toBe('Bearer sk-secret');
    });

    it('should send response_format when provided', async () => {
      fetchSpy.mockResolvedValueOnce(
        jsonResponse(makeWireResponse('{"key":"value"}')),
      );

      const client = VllmClient.withEndpoint('http://localhost:8000', 'test');
      await client.chatCompletion(
        [{ role: 'user', content: 'json' }],
        undefined,
        undefined,
        { type: 'json_object' },
      );

      const body = JSON.parse(fetchSpy.mock.calls[0][1]?.body as string);
      expect(body.response_format).toEqual({ type: 'json_object' });
    });

    it('should throw VllmEmptyResponseError when no choices', async () => {
      const wireResp: WireCompletionResponse = {
        id: 'test',
        object: 'chat.completion',
        created: 1,
        model: 'test',
        choices: [],
      };
      fetchSpy.mockResolvedValueOnce(jsonResponse(wireResp));

      const client = VllmClient.withEndpoint('http://localhost:8000', 'test');
      await expect(
        client.chatCompletion([{ role: 'user', content: 'Hi' }]),
      ).rejects.toThrow(VllmEmptyResponseError);
    });

    it('should throw VllmEmptyResponseError when content is null', async () => {
      const wireResp: WireCompletionResponse = {
        id: 'test',
        object: 'chat.completion',
        created: 1,
        model: 'test',
        choices: [
          { index: 0, message: { role: 'assistant', content: null }, finish_reason: 'stop' },
        ],
      };
      fetchSpy.mockResolvedValueOnce(jsonResponse(wireResp));

      const client = VllmClient.withEndpoint('http://localhost:8000', 'test');
      await expect(
        client.chatCompletion([{ role: 'user', content: 'Hi' }]),
      ).rejects.toThrow(VllmEmptyResponseError);
    });
  });

  // -------------------------------------------------------------------------
  // complete (full response)
  // -------------------------------------------------------------------------

  describe('complete', () => {
    it('should return full CompletionResponse with usage info', async () => {
      fetchSpy.mockResolvedValueOnce(
        jsonResponse(makeWireResponse('The answer')),
      );

      const client = VllmClient.withEndpoint('http://localhost:8000', 'test');
      const response = await client.complete({
        messages: [{ role: 'user', content: 'question' }],
        temperature: 0.5,
        maxTokens: 256,
      });

      expect(response.id).toBe('chatcmpl-test');
      expect(response.model).toBe('test-model');
      expect(response.choices).toHaveLength(1);
      expect(response.choices[0].message.content).toBe('The answer');
      expect(response.choices[0].message.role).toBe('assistant');
      expect(response.choices[0].finishReason).toBe('stop');
      expect(response.usage).toEqual({
        promptTokens: 10,
        completionTokens: 20,
        totalTokens: 30,
      });
    });

    it('should handle response without usage field', async () => {
      const wire: WireCompletionResponse = {
        id: 'test',
        object: 'chat.completion',
        created: 1,
        model: 'test',
        choices: [
          { index: 0, message: { role: 'assistant', content: 'ok' }, finish_reason: 'stop' },
        ],
      };
      fetchSpy.mockResolvedValueOnce(jsonResponse(wire));

      const client = VllmClient.withEndpoint('http://localhost:8000', 'test');
      const response = await client.complete({
        messages: [{ role: 'user', content: 'q' }],
      });

      expect(response.usage).toBeUndefined();
    });

    it('should throw VllmParseError on invalid JSON response', async () => {
      fetchSpy.mockResolvedValueOnce(textResponse('not json', 200));

      const client = VllmClient.withEndpoint('http://localhost:8000', 'test');
      await expect(
        client.complete({ messages: [{ role: 'user', content: 'q' }] }),
      ).rejects.toThrow(VllmParseError);
    });

    it('should pass optional parameters in wire format', async () => {
      fetchSpy.mockResolvedValueOnce(
        jsonResponse(makeWireResponse('ok')),
      );

      const client = VllmClient.withEndpoint('http://localhost:8000', 'test');
      await client.complete({
        messages: [{ role: 'user', content: 'q' }],
        topP: 0.9,
        frequencyPenalty: 0.5,
        presencePenalty: 0.3,
        stop: ['\n'],
      });

      const body = JSON.parse(fetchSpy.mock.calls[0][1]?.body as string);
      expect(body.top_p).toBe(0.9);
      expect(body.frequency_penalty).toBe(0.5);
      expect(body.presence_penalty).toBe(0.3);
      expect(body.stop).toEqual(['\n']);
    });
  });

  // -------------------------------------------------------------------------
  // Retry logic
  // -------------------------------------------------------------------------

  describe('retry logic', () => {
    it('should retry on 500 errors and succeed', async () => {
      fetchSpy
        .mockResolvedValueOnce(textResponse('Internal Server Error', 500))
        .mockResolvedValueOnce(jsonResponse(makeWireResponse('recovered')));

      const client = new VllmClient({
        endpoint: 'http://localhost:8000',
        model: 'test',
        maxRetries: 3,
        baseRetryDelayMs: 1, // fast for tests
      });

      const result = await client.chatCompletion([
        { role: 'user', content: 'Hi' },
      ]);
      expect(result).toBe('recovered');
      expect(fetchSpy).toHaveBeenCalledTimes(2);
    });

    it('should retry on 429 rate limit errors', async () => {
      fetchSpy
        .mockResolvedValueOnce(textResponse('Rate limited', 429))
        .mockResolvedValueOnce(jsonResponse(makeWireResponse('ok')));

      const client = new VllmClient({
        endpoint: 'http://localhost:8000',
        model: 'test',
        maxRetries: 3,
        baseRetryDelayMs: 1,
      });

      const result = await client.chatCompletion([
        { role: 'user', content: 'Hi' },
      ]);
      expect(result).toBe('ok');
    });

    it('should NOT retry on 4xx client errors (except 429)', async () => {
      fetchSpy.mockResolvedValueOnce(textResponse('Bad Request', 400));

      const client = new VllmClient({
        endpoint: 'http://localhost:8000',
        model: 'test',
        maxRetries: 3,
        baseRetryDelayMs: 1,
      });

      await expect(
        client.chatCompletion([{ role: 'user', content: 'Hi' }]),
      ).rejects.toThrow(VllmApiError);
      expect(fetchSpy).toHaveBeenCalledOnce();
    });

    it('should NOT retry on 401 unauthorized', async () => {
      fetchSpy.mockResolvedValueOnce(textResponse('Unauthorized', 401));

      const client = new VllmClient({
        endpoint: 'http://localhost:8000',
        model: 'test',
        maxRetries: 3,
        baseRetryDelayMs: 1,
      });

      const err = await client
        .chatCompletion([{ role: 'user', content: 'Hi' }])
        .catch((e: Error) => e);
      expect(err).toBeInstanceOf(VllmApiError);
      expect((err as VllmApiError).status).toBe(401);
    });

    it('should NOT retry on 404 not found', async () => {
      fetchSpy.mockResolvedValueOnce(textResponse('Not Found', 404));

      const client = new VllmClient({
        endpoint: 'http://localhost:8000',
        model: 'test',
        maxRetries: 3,
        baseRetryDelayMs: 1,
      });

      await expect(
        client.chatCompletion([{ role: 'user', content: 'Hi' }]),
      ).rejects.toThrow(VllmApiError);
      expect(fetchSpy).toHaveBeenCalledOnce();
    });

    it('should retry on network errors', async () => {
      fetchSpy
        .mockRejectedValueOnce(new TypeError('fetch failed'))
        .mockResolvedValueOnce(jsonResponse(makeWireResponse('ok')));

      const client = new VllmClient({
        endpoint: 'http://localhost:8000',
        model: 'test',
        maxRetries: 3,
        baseRetryDelayMs: 1,
      });

      const result = await client.chatCompletion([
        { role: 'user', content: 'Hi' },
      ]);
      expect(result).toBe('ok');
      expect(fetchSpy).toHaveBeenCalledTimes(2);
    });

    it('should throw after exhausting all retries on 500', async () => {
      fetchSpy.mockResolvedValue(textResponse('Server Error', 500));

      const client = new VllmClient({
        endpoint: 'http://localhost:8000',
        model: 'test',
        maxRetries: 2,
        baseRetryDelayMs: 1,
      });

      const err = await client
        .chatCompletion([{ role: 'user', content: 'Hi' }])
        .catch((e: Error) => e);
      expect(err).toBeInstanceOf(VllmApiError);
      expect((err as VllmApiError).status).toBe(500);
      // 1 initial + 2 retries = 3 calls
      expect(fetchSpy).toHaveBeenCalledTimes(3);
    });
  });

  // -------------------------------------------------------------------------
  // batchCompletion
  // -------------------------------------------------------------------------

  describe('batchCompletion', () => {
    it('should execute multiple requests in parallel', async () => {
      fetchSpy
        .mockResolvedValueOnce(jsonResponse(makeWireResponse('answer1')))
        .mockResolvedValueOnce(jsonResponse(makeWireResponse('answer2')))
        .mockResolvedValueOnce(jsonResponse(makeWireResponse('answer3')));

      const client = VllmClient.withEndpoint('http://localhost:8000', 'test');
      const results = await client.batchCompletion([
        [[{ role: 'user', content: 'q1' }], 0.5, 100],
        [[{ role: 'user', content: 'q2' }], 0.7],
        [[{ role: 'user', content: 'q3' }]],
      ]);

      expect(results).toEqual(['answer1', 'answer2', 'answer3']);
      expect(fetchSpy).toHaveBeenCalledTimes(3);

      // Verify JSON response format is set for each request
      for (const call of fetchSpy.mock.calls) {
        const body = JSON.parse(call[1]?.body as string);
        expect(body.response_format).toEqual({ type: 'json_object' });
      }
    });

    it('should reject all if one request fails', async () => {
      fetchSpy
        .mockResolvedValueOnce(jsonResponse(makeWireResponse('ok')))
        .mockResolvedValueOnce(textResponse('Bad Request', 400));

      const client = new VllmClient({
        endpoint: 'http://localhost:8000',
        model: 'test',
        maxRetries: 0,
        baseRetryDelayMs: 1,
      });

      await expect(
        client.batchCompletion([
          [[{ role: 'user', content: 'q1' }]],
          [[{ role: 'user', content: 'q2' }]],
        ]),
      ).rejects.toThrow(VllmApiError);
    });
  });

  // -------------------------------------------------------------------------
  // healthCheck
  // -------------------------------------------------------------------------

  describe('healthCheck', () => {
    it('should succeed when server returns 200', async () => {
      fetchSpy.mockResolvedValueOnce(textResponse('OK', 200));

      const client = VllmClient.withEndpoint('http://localhost:8000', 'test');
      await expect(client.healthCheck()).resolves.toBeUndefined();

      const [url] = fetchSpy.mock.calls[0];
      expect(url).toBe('http://localhost:8000/health');
    });

    it('should throw VllmHealthCheckError on non-200 status', async () => {
      fetchSpy.mockResolvedValueOnce(textResponse('Bad Gateway', 502));

      const client = VllmClient.withEndpoint('http://localhost:8000', 'test');
      await expect(client.healthCheck()).rejects.toThrow(VllmHealthCheckError);
    });

    it('should throw VllmHealthCheckError on network failure', async () => {
      fetchSpy.mockRejectedValueOnce(new TypeError('fetch failed'));

      const client = VllmClient.withEndpoint('http://localhost:8000', 'test');
      const err = await client.healthCheck().catch((e: Error) => e);
      expect(err).toBeInstanceOf(VllmHealthCheckError);
      expect(err.message).toContain('fetch failed');
    });
  });

  // -------------------------------------------------------------------------
  // Streaming
  // -------------------------------------------------------------------------

  describe('streamCompletion', () => {
    it('should yield chunks from SSE stream', async () => {
      const chunks: WireStreamChunk[] = [
        makeStreamChunk('Hello'),
        makeStreamChunk(' world'),
        makeStreamChunk('!', 'stop'),
      ];
      fetchSpy.mockResolvedValueOnce(sseResponse(chunks));

      const client = VllmClient.withEndpoint('http://localhost:8000', 'test');
      const received: string[] = [];

      for await (const chunk of client.streamCompletion({
        messages: [{ role: 'user', content: 'Hi' }],
      })) {
        const content = chunk.choices[0]?.delta?.content;
        if (content) received.push(content);
      }

      expect(received).toEqual(['Hello', ' world', '!']);
    });

    it('should set stream=true in wire request', async () => {
      fetchSpy.mockResolvedValueOnce(sseResponse([]));

      const client = VllmClient.withEndpoint('http://localhost:8000', 'test');
      // consume the generator
      for await (const _chunk of client.streamCompletion({
        messages: [{ role: 'user', content: 'Hi' }],
      })) {
        // no-op
      }

      const body = JSON.parse(fetchSpy.mock.calls[0][1]?.body as string);
      expect(body.stream).toBe(true);
    });

    it('should throw VllmParseError when response body is null', async () => {
      fetchSpy.mockResolvedValueOnce(
        new Response(null, { status: 200 }),
      );

      const client = VllmClient.withEndpoint('http://localhost:8000', 'test');
      await expect(async () => {
        for await (const _chunk of client.streamCompletion({
          messages: [{ role: 'user', content: 'Hi' }],
        })) {
          // no-op
        }
      }).rejects.toThrow(VllmParseError);
    });
  });

  // -------------------------------------------------------------------------
  // streamChatCompletion (convenience)
  // -------------------------------------------------------------------------

  describe('streamChatCompletion', () => {
    it('should collect all content into a single string', async () => {
      const chunks: WireStreamChunk[] = [
        makeStreamChunk('Hello'),
        makeStreamChunk(' world'),
      ];
      fetchSpy.mockResolvedValueOnce(sseResponse(chunks));

      const client = VllmClient.withEndpoint('http://localhost:8000', 'test');
      const result = await client.streamChatCompletion(
        [{ role: 'user', content: 'Hi' }],
        0.7,
        100,
      );

      expect(result).toBe('Hello world');
    });

    it('should invoke onChunk callback for each chunk', async () => {
      const chunks: WireStreamChunk[] = [
        makeStreamChunk('a'),
        makeStreamChunk('b'),
      ];
      fetchSpy.mockResolvedValueOnce(sseResponse(chunks));

      const client = VllmClient.withEndpoint('http://localhost:8000', 'test');
      const seen: string[] = [];

      await client.streamChatCompletion(
        [{ role: 'user', content: 'Hi' }],
        undefined,
        undefined,
        undefined,
        (chunk) => {
          const c = chunk.choices[0]?.delta?.content;
          if (c) seen.push(c);
        },
      );

      expect(seen).toEqual(['a', 'b']);
    });
  });

  // -------------------------------------------------------------------------
  // Timeout
  // -------------------------------------------------------------------------

  describe('timeout', () => {
    it('should abort requests that exceed timeoutMs', async () => {
      fetchSpy.mockImplementation(
        (_url: string | URL | Request, init?: RequestInit) =>
          new Promise((_resolve, reject) => {
            // Simulate the signal aborting the request
            init?.signal?.addEventListener('abort', () => {
              const err = new DOMException('The operation was aborted.', 'AbortError');
              reject(err);
            });
          }),
      );

      const client = new VllmClient({
        endpoint: 'http://localhost:8000',
        model: 'test',
        maxRetries: 0,
        timeoutMs: 50,
        baseRetryDelayMs: 1,
      });

      await expect(
        client.chatCompletion([{ role: 'user', content: 'Hi' }]),
      ).rejects.toThrow(VllmTimeoutError);
    });
  });

  // -------------------------------------------------------------------------
  // URL construction edge cases
  // -------------------------------------------------------------------------

  describe('URL construction', () => {
    it('should handle endpoint without trailing slash', async () => {
      fetchSpy.mockResolvedValueOnce(
        jsonResponse(makeWireResponse('ok')),
      );
      const client = VllmClient.withEndpoint('http://localhost:8000', 'test');
      await client.chatCompletion([{ role: 'user', content: 'Hi' }]);
      expect(fetchSpy.mock.calls[0][0]).toBe(
        'http://localhost:8000/v1/chat/completions',
      );
    });

    it('should handle endpoint with trailing slash', async () => {
      fetchSpy.mockResolvedValueOnce(
        jsonResponse(makeWireResponse('ok')),
      );
      const client = VllmClient.withEndpoint('http://localhost:8000/', 'test');
      await client.chatCompletion([{ role: 'user', content: 'Hi' }]);
      expect(fetchSpy.mock.calls[0][0]).toBe(
        'http://localhost:8000/v1/chat/completions',
      );
    });

    it('should handle endpoint with multiple trailing slashes', async () => {
      fetchSpy.mockResolvedValueOnce(
        jsonResponse(makeWireResponse('ok')),
      );
      const client = VllmClient.withEndpoint('http://localhost:8000///', 'test');
      await client.chatCompletion([{ role: 'user', content: 'Hi' }]);
      expect(fetchSpy.mock.calls[0][0]).toBe(
        'http://localhost:8000/v1/chat/completions',
      );
    });
  });
});
