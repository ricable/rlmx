import { describe, it, expect } from 'vitest';
import {
  DEPLOYMENT_PROFILES,
  PROFILE_NAMES,
  getProfile,
  getProfileOrThrow,
  listProfiles,
  fitsProfile,
} from '../src/profiles.js';

describe('profiles', () => {
  it('PROFILE_NAMES has 7 entries', () => {
    expect(PROFILE_NAMES).toHaveLength(7);
  });

  it('listProfiles returns 7 profiles', () => {
    expect(listProfiles()).toHaveLength(7);
  });

  it('getProfile("seed-compatible") returns correct profile', () => {
    const profile = getProfile('seed-compatible');
    expect(profile).toBeDefined();
    expect(profile!.name).toBe('seed-compatible');
    expect(profile!.target).toBe('rpi-zero-2w');
  });

  it('getProfile("unknown") returns undefined', () => {
    expect(getProfile('unknown')).toBeUndefined();
  });

  it('getProfileOrThrow("rlmx-edge") returns profile', () => {
    const profile = getProfileOrThrow('rlmx-edge');
    expect(profile.name).toBe('rlmx-edge');
  });

  it('getProfileOrThrow("unknown") throws', () => {
    expect(() => getProfileOrThrow('unknown')).toThrow('Deployment profile not found: unknown');
  });

  it('seed-compatible profile has maxMemoryMb 256', () => {
    const profile = getProfile('seed-compatible');
    expect(profile!.maxMemoryMb).toBe(256);
  });

  it('rlmx-cloud profile has zone BCloud', () => {
    const profile = getProfile('rlmx-cloud');
    expect(profile!.zone).toBe('BCloud');
  });

  it('fitsProfile: envelope within limits returns true', () => {
    const profile = getProfile('rlmx-edge')!;
    expect(fitsProfile({ memoryMb: 2048 }, profile)).toBe(true);
  });

  it('fitsProfile: envelope exceeding memory returns false', () => {
    const profile = getProfile('seed-compatible')!;
    expect(fitsProfile({ memoryMb: 512 }, profile)).toBe(false);
  });
});
