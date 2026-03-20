/**
 * UC1 — Agent type configurations and zone mappings.
 *
 * Illustrative only — not built or tested by CI.
 */
import { AgentType, LifeDomain } from '@aix/shared';

/** Device zone where an agent primarily runs. */
type DeviceZone = 'A-Mobile' | 'A-Desktop' | 'C-Hub' | 'D-Cloud';

interface AgentConfig {
  type: AgentType;
  zone: DeviceZone;
  domains: LifeDomain[];
  description: string;
}

/**
 * Core agent roster for UC1 voice-first swarm.
 *
 * Zone A-Mobile is the primary device (phone). Agents escalate to
 * A-Desktop, C-Hub, or D-Cloud when compute or data requirements exceed
 * on-device capacity.
 */
export const uc1Agents: AgentConfig[] = [
  {
    type: AgentType.VoiceCoordinator,
    zone: 'A-Mobile',
    domains: [],
    description: 'On-device STT → intent decomposition → dispatch',
  },
  {
    type: AgentType.Router,
    zone: 'A-Mobile',
    domains: [],
    description: 'Routes intents to domain-specific workers',
  },
  {
    type: AgentType.Worker,
    zone: 'A-Mobile',
    domains: [LifeDomain.Finance],
    description: 'Bill negotiation, savings detection, investment rebalancing',
  },
  {
    type: AgentType.Worker,
    zone: 'A-Mobile',
    domains: [LifeDomain.Health],
    description: 'Lab result analysis, health trajectory, medication reminders',
  },
  {
    type: AgentType.Worker,
    zone: 'A-Mobile',
    domains: [LifeDomain.Legal],
    description: 'TOS analysis, contract review, rights protection',
  },
  {
    type: AgentType.Worker,
    zone: 'A-Mobile',
    domains: [LifeDomain.Shopping],
    description: 'Price comparison, deal detection, purchase timing',
  },
  {
    type: AgentType.Researcher,
    zone: 'A-Desktop',
    domains: [LifeDomain.Career, LifeDomain.Education],
    description: 'Job market analysis, skill gap detection, course recommendations',
  },
  {
    type: AgentType.Worker,
    zone: 'A-Mobile',
    domains: [LifeDomain.Home, LifeDomain.Travel],
    description: 'Home automation, energy optimization, travel planning',
  },
  {
    type: AgentType.Monitor,
    zone: 'C-Hub',
    domains: [],
    description: 'Cross-zone health monitoring, anomaly detection',
  },
  {
    type: AgentType.Reviewer,
    zone: 'A-Desktop',
    domains: [],
    description: 'Agent output quality review, safety checks',
  },
  {
    type: AgentType.MarketplaceManager,
    zone: 'D-Cloud',
    domains: [],
    description: 'Third-party agent discovery, security review, revenue share',
  },
  {
    type: AgentType.Coordinator,
    zone: 'A-Mobile',
    domains: [],
    description: 'Swarm consensus coordination, result aggregation',
  },
];
