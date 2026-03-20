/**
 * Result<T, E> — a discriminated union for success/failure, inspired by Rust.
 * Enables graceful degradation without throwing exceptions.
 */

export interface Ok<T> {
  readonly ok: true;
  readonly value: T;
}

export interface Err<E> {
  readonly ok: false;
  readonly error: E;
}

/** A Result is either Ok(value) or Err(error). */
export type Result<T, E = Error> = Ok<T> | Err<E>;

// ---------------------------------------------------------------------------
// Constructors
// ---------------------------------------------------------------------------

/** Create a successful Result. */
export function ok<T>(value: T): Ok<T> {
  return { ok: true, value };
}

/** Create a failed Result. */
export function err<E>(error: E): Err<E> {
  return { ok: false, error };
}

// ---------------------------------------------------------------------------
// Type guards
// ---------------------------------------------------------------------------

/** Check if a Result is Ok. */
export function isOk<T, E>(result: Result<T, E>): result is Ok<T> {
  return result.ok === true;
}

/** Check if a Result is Err. */
export function isErr<T, E>(result: Result<T, E>): result is Err<E> {
  return result.ok === false;
}

// ---------------------------------------------------------------------------
// Combinators
// ---------------------------------------------------------------------------

/** Map the success value, leaving errors untouched. */
export function mapResult<T, U, E>(
  result: Result<T, E>,
  fn: (value: T) => U,
): Result<U, E> {
  return result.ok ? ok(fn(result.value)) : result;
}

/** Map the error value, leaving successes untouched. */
export function mapErr<T, E, F>(
  result: Result<T, E>,
  fn: (error: E) => F,
): Result<T, F> {
  return result.ok ? result : err(fn(result.error));
}

/** Flat-map (chain) on the success value. */
export function flatMap<T, U, E>(
  result: Result<T, E>,
  fn: (value: T) => Result<U, E>,
): Result<U, E> {
  return result.ok ? fn(result.value) : result;
}

/** Unwrap the success value, or return a default on error. */
export function unwrapOr<T, E>(result: Result<T, E>, defaultValue: T): T {
  return result.ok ? result.value : defaultValue;
}

/** Unwrap the success value, or throw the error. */
export function unwrap<T, E>(result: Result<T, E>): T {
  if (result.ok) return result.value;
  throw result.error instanceof Error
    ? result.error
    : new Error(String(result.error));
}

// ---------------------------------------------------------------------------
// Async helpers
// ---------------------------------------------------------------------------

/**
 * Wrap an async function call in a Result, catching thrown errors.
 * Useful for converting promise-based APIs to Result-based flows.
 */
export async function tryAsync<T>(
  fn: () => Promise<T>,
): Promise<Result<T, Error>> {
  try {
    return ok(await fn());
  } catch (e) {
    return err(e instanceof Error ? e : new Error(String(e)));
  }
}

/**
 * Wrap a synchronous function call in a Result, catching thrown errors.
 */
export function trySync<T>(fn: () => T): Result<T, Error> {
  try {
    return ok(fn());
  } catch (e) {
    return err(e instanceof Error ? e : new Error(String(e)));
  }
}

// ---------------------------------------------------------------------------
// AixResult alias
// ---------------------------------------------------------------------------

import type { AixError } from './errors.js';

/** Convenience alias: a Result whose error type is AixError. */
export type AixResult<T> = Result<T, AixError>;
