/**
 * Base error class for all vLLM client errors.
 *
 * Maps to the Rust VllmError enum in rlmx-rlm/src/vllm.rs.
 */
export class VllmError extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'VllmError';
    Object.setPrototypeOf(this, new.target.prototype);
  }
}

/**
 * HTTP request failed (network error, DNS failure, etc.).
 * Maps to VllmError::RequestError in Rust.
 */
export class VllmRequestError extends VllmError {
  public readonly cause?: Error;

  constructor(message: string, cause?: Error) {
    super(`HTTP request failed: ${message}`);
    this.name = 'VllmRequestError';
    this.cause = cause;
  }
}

/**
 * The API returned a non-success status code.
 * Maps to VllmError::ApiError in Rust.
 */
export class VllmApiError extends VllmError {
  public readonly status: number;
  public readonly body: string;

  constructor(status: number, body: string) {
    super(`API returned error: ${status} - ${body}`);
    this.name = 'VllmApiError';
    this.status = status;
    this.body = body;
  }
}

/**
 * Failed to parse the API response body.
 * Maps to VllmError::ParseError in Rust.
 */
export class VllmParseError extends VllmError {
  constructor(message: string) {
    super(`Failed to parse API response: ${message}`);
    this.name = 'VllmParseError';
  }
}

/**
 * The API returned zero completion choices.
 * Maps to VllmError::EmptyResponse in Rust.
 */
export class VllmEmptyResponseError extends VllmError {
  constructor() {
    super('No completion choices returned');
    this.name = 'VllmEmptyResponseError';
  }
}

/**
 * Maximum retry attempts exceeded without success.
 * Maps to VllmError::MaxRetriesExceeded in Rust.
 */
export class VllmMaxRetriesError extends VllmError {
  public readonly maxRetries: number;

  constructor(maxRetries: number) {
    super(`Max retries (${maxRetries}) exceeded`);
    this.name = 'VllmMaxRetriesError';
    this.maxRetries = maxRetries;
  }
}

/**
 * Health check endpoint returned a non-success status or failed to connect.
 * Maps to VllmError::HealthCheckFailed in Rust.
 */
export class VllmHealthCheckError extends VllmError {
  constructor(message: string) {
    super(`Health check failed: ${message}`);
    this.name = 'VllmHealthCheckError';
  }
}

/**
 * Request timed out before receiving a response.
 */
export class VllmTimeoutError extends VllmError {
  public readonly timeoutMs: number;

  constructor(timeoutMs: number) {
    super(`Request timed out after ${timeoutMs}ms`);
    this.name = 'VllmTimeoutError';
    this.timeoutMs = timeoutMs;
  }
}
