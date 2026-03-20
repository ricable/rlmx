/**
 * UC2 — Mesh topology agent configurations.
 *
 * Illustrative only — not built or tested by CI.
 */
import { AgentType, LifeDomain } from '@aix/shared';

type DeviceZone = 'A-Mobile' | 'A-Desktop' | 'C-Hub' | 'D-Cloud';

interface MeshAgentConfig {
  type: AgentType;
  zone: DeviceZone;
  domains: LifeDomain[];
  description: string;
  meshRole: 'primary' | 'compute' | 'anchor' | 'burst';
}

/** Agent roster for UC2 personal agent cloud mesh. */
export const uc2Agents: MeshAgentConfig[] = [
  {
    type: AgentType.MeshCoordinator,
    zone: 'C-Hub',
    domains: [],
    description: 'Single mesh coordinator — privacy anchor, device discovery',
    meshRole: 'anchor',
  },
  {
    type: AgentType.Worker,
    zone: 'A-Mobile',
    domains: [LifeDomain.Shopping, LifeDomain.Finance],
    description: 'Shopping sentinel + finance pilot on primary device',
    meshRole: 'primary',
  },
  {
    type: AgentType.Worker,
    zone: 'A-Mobile',
    domains: [LifeDomain.Health],
    description: 'Health guardian — lab results, meal planning, activity nudges',
    meshRole: 'primary',
  },
  {
    type: AgentType.Researcher,
    zone: 'A-Desktop',
    domains: [LifeDomain.Career, LifeDomain.Legal],
    description: 'Heavy research on laptop — job market, legal document analysis',
    meshRole: 'compute',
  },
  {
    type: AgentType.FederationAgent,
    zone: 'C-Hub',
    domains: [],
    description: 'Federated learning — RVF container exchange, DP aggregation',
    meshRole: 'anchor',
  },
  {
    type: AgentType.BillingManager,
    zone: 'C-Hub',
    domains: [],
    description: 'Tier enforcement, capability tokens, family group management',
    meshRole: 'anchor',
  },
  {
    type: AgentType.Monitor,
    zone: 'C-Hub',
    domains: [LifeDomain.Home],
    description: 'Home IoT monitoring, energy optimization, security',
    meshRole: 'anchor',
  },
  {
    type: AgentType.Trainer,
    zone: 'A-Desktop',
    domains: [],
    description: 'Micro-LoRA training, EWC++ consolidation on laptop GPU',
    meshRole: 'compute',
  },
  {
    type: AgentType.Coordinator,
    zone: 'D-Cloud',
    domains: [],
    description: 'Cloud burst coordinator for heavy inference escalation',
    meshRole: 'burst',
  },
];
