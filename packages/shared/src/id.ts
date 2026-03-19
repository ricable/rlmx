/**
 * Shared ID generation utility for the @aix ecosystem.
 * Uses crypto.randomUUID() which is available in Node 19+ and modern browsers.
 */

/** Generate a unique identifier (UUID v4). */
export function generateId(): string {
  return crypto.randomUUID();
}
