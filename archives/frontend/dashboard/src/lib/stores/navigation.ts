import { writable, derived } from 'svelte/store';

export type SectionId = 'swarm' | 'agents' | 'compute' | 'memory' | 'edge' | 'research' | 'sandbox' | 'events' | 'tools' | 'rpc' | 'log';

export interface NavItem {
  id: string;
  label: string;
  badge?: string;
}

export interface Section {
  id: SectionId;
  title: string;
  icon: string;
  nav: NavItem[];
}

export const sections: Section[] = [
  {
    id: 'swarm', title: 'Swarm', icon: 'S',
    nav: [
      { id: 'overview', label: 'Overview' },
      { id: 'topology', label: 'Topology' },
      { id: 'consensus', label: 'Consensus' },
    ],
  },
  {
    id: 'agents', title: 'Agents', icon: 'A',
    nav: [
      { id: 'manager', label: 'Manager' },
      { id: 'matrix', label: 'Permissions' },
      { id: 'hierarchy', label: 'Hierarchy' },
    ],
  },
  {
    id: 'compute', title: 'Compute', icon: 'C',
    nav: [
      { id: 'pool', label: 'WASM Pool' },
      { id: 'bench', label: 'Benchmark' },
    ],
  },
  {
    id: 'memory', title: 'Memory', icon: 'M',
    nav: [
      { id: 'query', label: 'Query' },
      { id: 'ingest', label: 'Ingest' },
      { id: 'stats', label: 'Stats' },
    ],
  },
  {
    id: 'edge', title: 'Edge AI', icon: 'E',
    nav: [
      { id: 'inference', label: 'Inference' },
      { id: 'models', label: 'Tiered Models' },
      { id: 'chat', label: 'Chat Format' },
    ],
  },
  {
    id: 'research', title: 'Research', icon: 'R',
    nav: [
      { id: 'tracker', label: 'Tracker' },
      { id: 'evolution', label: 'Evolution' },
    ],
  },
  {
    id: 'sandbox', title: 'Sandbox', icon: 'B',
    nav: [
      { id: 'manager', label: 'Manager' },
      { id: 'profiles', label: 'Profiles' },
      { id: 'fleet', label: 'Fleet Deploy' },
    ],
  },
  {
    id: 'events', title: 'Events', icon: 'W',
    nav: [{ id: 'stream', label: 'Live Stream', badge: 'LIVE' }],
  },
  {
    id: 'tools', title: 'Tools', icon: 'T',
    nav: [{ id: 'registry', label: 'MCP Tools' }],
  },
  {
    id: 'rpc', title: 'RPC', icon: 'J',
    nav: [{ id: 'console', label: 'Console' }],
  },
  {
    id: 'log', title: 'Log', icon: 'L',
    nav: [{ id: 'history', label: 'History' }],
  },
];

export const activeSection = writable<SectionId>('swarm');
export const activeView = writable<string>('overview');

export const currentSection = derived(activeSection, ($s) =>
  sections.find(sec => sec.id === $s)!
);

export function navigate(section: SectionId, view?: string) {
  activeSection.set(section);
  const sec = sections.find(s => s.id === section);
  activeView.set(view ?? sec?.nav[0]?.id ?? '');
}
