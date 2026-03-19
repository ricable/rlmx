/**
 * A2A agent card discovery — builds agent cards from RLMX agent types (ADR-034).
 */

import { AGENT_TYPES } from '@aix/shared';
import type { AgentCard, A2ASkill } from './types.js';

/** Descriptions for each of the 17 RLMX agent types. */
const AGENT_TYPE_DESCRIPTIONS: Record<string, { description: string; tags: string[] }> = {
  Coordinator: {
    description: 'Orchestrates multi-agent workflows, assigns tasks, and manages agent lifecycle',
    tags: ['orchestration', 'workflow', 'management'],
  },
  Researcher: {
    description: 'Performs deep research, gathers information, and synthesizes findings',
    tags: ['research', 'analysis', 'information-gathering'],
  },
  Router: {
    description: 'Routes queries to appropriate agents based on intent and domain classification',
    tags: ['routing', 'classification', 'intent'],
  },
  Experimenter: {
    description: 'Designs and runs experiments, A/B tests, and hypothesis validation',
    tags: ['experimentation', 'testing', 'hypothesis'],
  },
  Worker: {
    description: 'Executes general-purpose tasks and processes work items from the queue',
    tags: ['execution', 'task-processing', 'general'],
  },
  Monitor: {
    description: 'Monitors system health, resource usage, and performance metrics',
    tags: ['monitoring', 'health', 'metrics'],
  },
  Reviewer: {
    description: 'Reviews agent outputs, validates quality, and provides feedback',
    tags: ['review', 'quality', 'validation'],
  },
  Trainer: {
    description: 'Trains and fine-tunes models, manages training data and pipelines',
    tags: ['training', 'fine-tuning', 'ml'],
  },
  Validator: {
    description: 'Validates data integrity, schema compliance, and proof verification',
    tags: ['validation', 'verification', 'compliance'],
  },
  Replicator: {
    description: 'Replicates data and state across nodes for redundancy and availability',
    tags: ['replication', 'redundancy', 'sync'],
  },
  Embedder: {
    description: 'Generates vector embeddings for text, images, and structured data',
    tags: ['embeddings', 'vectors', 'semantic'],
  },
  Analyst: {
    description: 'Analyzes data patterns, generates reports, and extracts insights',
    tags: ['analysis', 'reporting', 'insights'],
  },
  VoiceCoordinator: {
    description: 'Manages voice sessions, transcription pipelines, and speech synthesis',
    tags: ['voice', 'speech', 'transcription'],
  },
  MarketplaceManager: {
    description: 'Manages marketplace listings, reviews, and agent distribution',
    tags: ['marketplace', 'distribution', 'publishing'],
  },
  MeshCoordinator: {
    description: 'Coordinates personal device mesh topology and cross-device sync',
    tags: ['mesh', 'devices', 'sync'],
  },
  FederationAgent: {
    description: 'Participates in federated learning cycles with differential privacy',
    tags: ['federation', 'privacy', 'distributed-learning'],
  },
  BillingManager: {
    description: 'Manages subscriptions, usage tracking, and billing enforcement',
    tags: ['billing', 'subscriptions', 'usage'],
  },
};

/**
 * Build an A2A agent card with all 17 RLMX agent types as skills.
 *
 * @param baseUrl - The base URL where this agent is reachable.
 * @param version - The version string for the agent card.
 */
export function buildAgentCard(baseUrl: string, version: string): AgentCard {
  const skills: A2ASkill[] = AGENT_TYPES.map((agentType) => {
    const info = AGENT_TYPE_DESCRIPTIONS[agentType];
    const id = agentType
      .replace(/([a-z])([A-Z])/g, '$1-$2')
      .toLowerCase();
    return {
      id,
      name: agentType,
      description: info.description,
      tags: info.tags,
    };
  });

  return {
    name: 'RLMX Agent',
    version,
    description:
      'RLMX cognition kernel agent with 17 specialized agent types',
    url: baseUrl,
    capabilities: {
      streaming: false,
      pushNotifications: false,
      stateTransitionHistory: true,
    },
    skills,
    authentication: {
      schemes: ['bearer'],
    },
  };
}
