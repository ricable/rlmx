/**
 * UC2 — Personal Agent Cloud
 *
 * Example PluginManifest for the multi-device mesh use case.
 * Illustrative only — not built or tested by CI.
 */
import type { PluginManifest } from '@aix/plugin';

export const uc2Manifest: PluginManifest = {
  name: 'uc2-personal-cloud',
  version: '0.1.0',
  description:
    'Personal agent cloud — multi-device mesh with federated learning and tiered billing',
  author: 'RLMX',
  license: 'MIT',
  domains: [
    'Finance',
    'Health',
    'Legal',
    'Career',
    'Education',
    'Home',
    'Shopping',
    'Travel',
    'Social',
    'Government',
    'Automotive',
    'Pet',
  ],
  minHostVersion: '>=0.1.0',
};
