/**
 * SONA adaptive weight provider.
 * Dynamically imports @aix/core — if unavailable, falls back to uniform weights.
 */
export async function getAdaptiveWeights(backends: string[]): Promise<Map<string, number>> {
  const uniform = new Map<string, number>();
  for (const b of backends) {
    uniform.set(b, 1.0);
  }

  try {
    const core = await import('@aix/core');
    if (!core || (core as Record<string, unknown>).status === 'unavailable') {
      return uniform;
    }
    // When SONA is available, it would return optimized weights per backend
    // For now, return uniform weights
    return uniform;
  } catch {
    return uniform;
  }
}
