/**
 * UC1 — One Voice, Millions of Agents
 *
 * Example PluginManifest for the voice-first multi-agent use case.
 * Illustrative only — not built or tested by CI.
 */
import type { PluginManifest } from '@aix/plugin';

export const uc1Manifest: PluginManifest = {
  name: 'uc1-voice-agents',
  version: '0.1.0',
  description:
    'Voice-first cognition kernel — one spoken command activates agents across 12 life domains',
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
