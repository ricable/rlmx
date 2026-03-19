/**
 * useAgents — React hook for managing the RuVix agent marketplace.
 *
 * Provides agent install/uninstall, leveling, and XP tracking for the
 * mobile app's agent card UI.
 */

import { useCallback, useState } from 'react';

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

export type AgentDomain =
  | 'Finance'
  | 'Health'
  | 'Legal'
  | 'Shopping'
  | 'Calendar'
  | 'Emergency'
  | 'Productivity'
  | 'Travel';

export interface Agent {
  id: string;
  name: string;
  domain: AgentDomain;
  level: number;
  xp: number;
  xpToNextLevel: number;
  icon: string;
  description: string;
  installed: boolean;
  capabilities: string[];
}

export interface UseAgentsResult {
  agents: Agent[];
  installedAgents: Agent[];
  availableAgents: Agent[];
  installAgent: (id: string) => void;
  uninstallAgent: (id: string) => void;
  addXp: (id: string, amount: number) => void;
  levelUpAgent: (id: string) => void;
  getAgent: (id: string) => Agent | undefined;
}

// ---------------------------------------------------------------------------
// XP curve: each level requires more XP
// ---------------------------------------------------------------------------

function xpForLevel(level: number): number {
  return Math.floor(100 * Math.pow(1.5, level - 1));
}

// ---------------------------------------------------------------------------
// Default agent catalog
// ---------------------------------------------------------------------------

function createDefaultAgents(): Agent[] {
  return [
    {
      id: 'finance-agent',
      name: 'Finance Pro',
      domain: 'Finance',
      level: 1,
      xp: 0,
      xpToNextLevel: xpForLevel(1),
      icon: 'dollar-sign',
      description:
        'Tracks spending, finds savings, optimizes subscriptions, and manages budgets.',
      installed: true,
      capabilities: [
        'Subscription tracking',
        'Bill negotiation',
        'Budget analysis',
        'Savings detection',
      ],
    },
    {
      id: 'health-agent',
      name: 'Health Guardian',
      domain: 'Health',
      level: 1,
      xp: 0,
      xpToNextLevel: xpForLevel(1),
      icon: 'heart',
      description:
        'Monitors health metrics, schedules appointments, and provides wellness insights.',
      installed: true,
      capabilities: [
        'Activity tracking',
        'Appointment scheduling',
        'Sleep analysis',
        'Medication reminders',
      ],
    },
    {
      id: 'legal-agent',
      name: 'Legal Advisor',
      domain: 'Legal',
      level: 1,
      xp: 0,
      xpToNextLevel: xpForLevel(1),
      icon: 'shield',
      description:
        'Reviews contracts, tracks compliance deadlines, and summarizes legal documents.',
      installed: false,
      capabilities: [
        'Contract review',
        'Terms analysis',
        'Compliance tracking',
        'Document summarization',
      ],
    },
    {
      id: 'shopping-agent',
      name: 'Deal Finder',
      domain: 'Shopping',
      level: 1,
      xp: 0,
      xpToNextLevel: xpForLevel(1),
      icon: 'shopping-cart',
      description:
        'Compares prices, tracks deliveries, finds coupons, and monitors price drops.',
      installed: true,
      capabilities: [
        'Price comparison',
        'Coupon finding',
        'Delivery tracking',
        'Price drop alerts',
      ],
    },
    {
      id: 'calendar-agent',
      name: 'Time Keeper',
      domain: 'Calendar',
      level: 1,
      xp: 0,
      xpToNextLevel: xpForLevel(1),
      icon: 'calendar',
      description:
        'Manages your schedule, sets intelligent reminders, and optimizes your day.',
      installed: true,
      capabilities: [
        'Smart scheduling',
        'Meeting prep',
        'Focus time blocking',
        'Travel time estimation',
      ],
    },
    {
      id: 'emergency-agent',
      name: 'Safety Net',
      domain: 'Emergency',
      level: 1,
      xp: 0,
      xpToNextLevel: xpForLevel(1),
      icon: 'alert-triangle',
      description:
        'Handles urgent situations, contacts emergency services, and notifies trusted contacts.',
      installed: false,
      capabilities: [
        'Emergency contacts',
        'Location sharing',
        'Service dispatch',
        'Safety alerts',
      ],
    },
    {
      id: 'productivity-agent',
      name: 'Focus Engine',
      domain: 'Productivity',
      level: 1,
      xp: 0,
      xpToNextLevel: xpForLevel(1),
      icon: 'zap',
      description:
        'Tracks tasks, manages projects, reduces distractions, and boosts output.',
      installed: false,
      capabilities: [
        'Task prioritization',
        'Distraction blocking',
        'Progress tracking',
        'Daily planning',
      ],
    },
    {
      id: 'travel-agent',
      name: 'Trip Planner',
      domain: 'Travel',
      level: 1,
      xp: 0,
      xpToNextLevel: xpForLevel(1),
      icon: 'map-pin',
      description:
        'Plans trips, finds deals on flights and hotels, and manages itineraries.',
      installed: false,
      capabilities: [
        'Flight search',
        'Hotel comparison',
        'Itinerary building',
        'Travel alerts',
      ],
    },
  ];
}

// ---------------------------------------------------------------------------
// Hook
// ---------------------------------------------------------------------------

export function useAgents(): UseAgentsResult {
  const [agents, setAgents] = useState<Agent[]>(createDefaultAgents);

  const installedAgents = agents.filter((a) => a.installed);
  const availableAgents = agents.filter((a) => !a.installed);

  const installAgent = useCallback((id: string) => {
    setAgents((prev) =>
      prev.map((a) => (a.id === id ? { ...a, installed: true } : a)),
    );
  }, []);

  const uninstallAgent = useCallback((id: string) => {
    setAgents((prev) =>
      prev.map((a) => (a.id === id ? { ...a, installed: false } : a)),
    );
  }, []);

  const addXp = useCallback((id: string, amount: number) => {
    setAgents((prev) =>
      prev.map((a) => {
        if (a.id !== id) return a;

        let newXp = a.xp + amount;
        let newLevel = a.level;
        let xpNeeded = a.xpToNextLevel;

        // Auto level-up if XP exceeds threshold
        while (newXp >= xpNeeded) {
          newXp -= xpNeeded;
          newLevel += 1;
          xpNeeded = xpForLevel(newLevel);
        }

        return {
          ...a,
          xp: newXp,
          level: newLevel,
          xpToNextLevel: xpNeeded,
        };
      }),
    );
  }, []);

  const levelUpAgent = useCallback((id: string) => {
    setAgents((prev) =>
      prev.map((a) => {
        if (a.id !== id) return a;
        const newLevel = a.level + 1;
        return {
          ...a,
          level: newLevel,
          xp: 0,
          xpToNextLevel: xpForLevel(newLevel),
        };
      }),
    );
  }, []);

  const getAgent = useCallback(
    (id: string): Agent | undefined => {
      return agents.find((a) => a.id === id);
    },
    [agents],
  );

  return {
    agents,
    installedAgents,
    availableAgents,
    installAgent,
    uninstallAgent,
    addXp,
    levelUpAgent,
    getAgent,
  };
}
