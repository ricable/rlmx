import type {
  Agent,
  DomainScore,
  BriefingItem,
  Achievement,
  UserProfile,
  AgentProgress,
} from '../types';

export const demoUser: UserProfile = {
  name: 'Cedric',
  tier: 'Pro',
  level: 14,
  xp: 2340,
  xpToNext: 3000,
  streak: 14,
  longestStreak: 21,
  joinedDate: '2025-12-01',
  achievementsUnlocked: 12,
  totalAchievements: 50,
};

export const demoAgents: Agent[] = [
  {
    id: 'a1',
    name: 'Penny',
    domain: 'Finance',
    icon: 'attach-money',
    level: 7,
    xp: 680,
    xpToNext: 800,
    description: 'Tracks spending patterns, finds subscriptions to cancel, and negotiates bills.',
    locked: false,
    tasksCompleted: 142,
    successRate: 94,
  },
  {
    id: 'a2',
    name: 'Vitalis',
    domain: 'Health',
    icon: 'favorite',
    level: 5,
    xp: 420,
    xpToNext: 600,
    description: 'Monitors health metrics, suggests exercise routines, and tracks nutrition.',
    locked: false,
    tasksCompleted: 87,
    successRate: 89,
  },
  {
    id: 'a3',
    name: 'Chronos',
    domain: 'Time',
    icon: 'schedule',
    level: 6,
    xp: 550,
    xpToNext: 700,
    description: 'Manages calendar, blocks focus time, and optimizes daily schedules.',
    locked: false,
    tasksCompleted: 203,
    successRate: 91,
  },
  {
    id: 'a4',
    name: 'Sentinel',
    domain: 'Safety',
    icon: 'shield',
    level: 8,
    xp: 750,
    xpToNext: 900,
    description: 'Monitors data breaches, secures accounts, and alerts on privacy risks.',
    locked: false,
    tasksCompleted: 56,
    successRate: 98,
  },
  {
    id: 'a5',
    name: 'Savvy',
    domain: 'Finance',
    icon: 'trending-up',
    level: 4,
    xp: 310,
    xpToNext: 500,
    description: 'Finds deals, compares prices, and applies coupons automatically.',
    locked: false,
    tasksCompleted: 94,
    successRate: 87,
  },
  {
    id: 'a6',
    name: 'Echo',
    domain: 'General',
    icon: 'psychology',
    level: 3,
    xp: 180,
    xpToNext: 400,
    description: 'General assistant for research, summaries, and creative tasks.',
    locked: false,
    tasksCompleted: 67,
    successRate: 92,
  },
  {
    id: 'a7',
    name: 'Flux',
    domain: 'Time',
    icon: 'bolt',
    level: 2,
    xp: 90,
    xpToNext: 300,
    description: 'Automates repetitive tasks and creates smart workflows.',
    locked: false,
    tasksCompleted: 31,
    successRate: 85,
  },
  {
    id: 'a8',
    name: 'Nexus',
    domain: 'General',
    icon: 'hub',
    level: 0,
    xp: 0,
    xpToNext: 200,
    description: 'Coordinates multiple agents for complex multi-step tasks.',
    locked: true,
    tasksCompleted: 0,
    successRate: 0,
  },
  {
    id: 'a9',
    name: 'Medica',
    domain: 'Health',
    icon: 'local-hospital',
    level: 0,
    xp: 0,
    xpToNext: 200,
    description: 'Tracks medications, appointments, and health records.',
    locked: true,
    tasksCompleted: 0,
    successRate: 0,
  },
  {
    id: 'a10',
    name: 'Fortress',
    domain: 'Safety',
    icon: 'lock',
    level: 0,
    xp: 0,
    xpToNext: 200,
    description: 'Advanced threat detection and identity protection.',
    locked: true,
    tasksCompleted: 0,
    successRate: 0,
  },
];

function generateSparkline(base: number, points: number = 14): number[] {
  const data: number[] = [];
  let val = base;
  for (let i = 0; i < points; i++) {
    val += (Math.random() - 0.45) * 5;
    val = Math.max(0, Math.min(100, val));
    data.push(Math.round(val));
  }
  return data;
}

export const demoDomainScores: DomainScore[] = [
  {
    domain: 'Finance',
    score: 82,
    trend: generateSparkline(78),
    change: 4,
    metrics: [
      {label: 'Monthly Savings', value: '$340', change: 12},
      {label: 'Bills Reduced', value: '3', change: 1},
      {label: 'Subscriptions Cut', value: '5'},
    ],
    suggestions: [
      'Switch to annual billing on Spotify to save $24/yr',
      'Your electric bill spiked 18% — check thermostat schedule',
    ],
  },
  {
    domain: 'Health',
    score: 61,
    trend: generateSparkline(58),
    change: -2,
    metrics: [
      {label: 'Steps Today', value: '6,240', change: -8},
      {label: 'Sleep Score', value: '72/100', change: 3},
      {label: 'Water Intake', value: '5/8 cups'},
    ],
    suggestions: [
      'You missed your morning walk 3 days this week',
      'Try a 10-min stretch before bed to improve sleep score',
    ],
  },
  {
    domain: 'Time',
    score: 78,
    trend: generateSparkline(75),
    change: 6,
    metrics: [
      {label: 'Focus Hours', value: '4.2h', change: 15},
      {label: 'Meetings Saved', value: '2.5h'},
      {label: 'Tasks Auto-done', value: '12'},
    ],
    suggestions: [
      'Block 9-11am as deep work — your peak productivity window',
      'Consolidate Thursday meetings into one 30-min block',
    ],
  },
  {
    domain: 'Safety',
    score: 71,
    trend: generateSparkline(70),
    change: 1,
    metrics: [
      {label: 'Breaches Checked', value: '24', change: 0},
      {label: 'Passwords Updated', value: '8'},
      {label: 'Privacy Score', value: '71/100'},
    ],
    suggestions: [
      'Update your LinkedIn password — found in recent breach',
      'Enable 2FA on your bank account',
    ],
  },
];

export const demoBriefing: BriefingItem[] = [
  {
    id: 'b1',
    title: 'Saved $23 on your phone bill',
    subtitle: 'Penny negotiated a loyalty discount with T-Mobile',
    domain: 'Finance',
    icon: 'savings',
    actionLabel: 'View Details',
  },
  {
    id: 'b2',
    title: 'Calendar optimized for focus',
    subtitle: 'Chronos moved 2 meetings to free up your morning block',
    domain: 'Time',
    icon: 'event-available',
    actionLabel: 'Review Changes',
  },
  {
    id: 'b3',
    title: 'New data breach detected',
    subtitle: 'Sentinel found your email in a recent Ticketmaster breach',
    domain: 'Safety',
    icon: 'warning',
    actionLabel: 'Secure Now',
  },
];

export const demoAchievements: Achievement[] = [
  {id: 'ach1', name: 'First Steps', description: 'Complete your first voice command', icon: 'mic', unlocked: true, unlockedAt: '2025-12-01', category: 'Basics'},
  {id: 'ach2', name: 'Money Saver', description: 'Save your first $100', icon: 'savings', unlocked: true, unlockedAt: '2025-12-15', category: 'Finance'},
  {id: 'ach3', name: 'Streak Starter', description: 'Maintain a 7-day streak', icon: 'local-fire-department', unlocked: true, unlockedAt: '2025-12-08', category: 'Engagement'},
  {id: 'ach4', name: 'Agent Whisperer', description: 'Level up an agent to level 5', icon: 'psychology', unlocked: true, unlockedAt: '2026-01-10', category: 'Agents'},
  {id: 'ach5', name: 'Privacy Pro', description: 'Secure all accounts against breaches', icon: 'verified-user', unlocked: true, unlockedAt: '2026-01-20', category: 'Safety'},
  {id: 'ach6', name: 'Focus Master', description: 'Complete 20 focus sessions', icon: 'center-focus-strong', unlocked: true, unlockedAt: '2026-02-01', category: 'Time'},
  {id: 'ach7', name: 'Bill Buster', description: 'Reduce 5 monthly bills', icon: 'receipt-long', unlocked: true, unlockedAt: '2026-02-10', category: 'Finance'},
  {id: 'ach8', name: 'Health Check', description: 'Track health for 30 days', icon: 'favorite', unlocked: true, unlockedAt: '2026-02-15', category: 'Health'},
  {id: 'ach9', name: 'Team Player', description: 'Use 5 different agents', icon: 'groups', unlocked: true, unlockedAt: '2026-02-20', category: 'Agents'},
  {id: 'ach10', name: 'Night Owl', description: 'Use the app after midnight', icon: 'nightlight', unlocked: true, unlockedAt: '2026-03-01', category: 'Engagement'},
  {id: 'ach11', name: 'Big Saver', description: 'Save $500 total', icon: 'account-balance', unlocked: true, unlockedAt: '2026-03-05', category: 'Finance'},
  {id: 'ach12', name: 'Two Weeks', description: 'Maintain a 14-day streak', icon: 'whatshot', unlocked: true, unlockedAt: '2026-03-19', category: 'Engagement'},
  {id: 'ach13', name: 'Grand Saver', description: 'Save $1,000 total', icon: 'diamond', unlocked: false, category: 'Finance'},
  {id: 'ach14', name: 'Marathon', description: 'Maintain a 30-day streak', icon: 'emoji-events', unlocked: false, category: 'Engagement'},
  {id: 'ach15', name: 'Agent Master', description: 'Level up an agent to level 10', icon: 'military-tech', unlocked: false, category: 'Agents'},
];

export const demoVoiceProgress: AgentProgress[] = [
  {agentId: 'a1', agentName: 'Penny', domain: 'Finance', status: 'complete', progress: 100, message: 'Found 2 subscriptions you can cancel'},
  {agentId: 'a3', agentName: 'Chronos', domain: 'Time', status: 'complete', progress: 100, message: 'Rescheduled 3 meetings for Thursday'},
  {agentId: 'a4', agentName: 'Sentinel', domain: 'Safety', status: 'working', progress: 67, message: 'Scanning breach databases...'},
  {agentId: 'a2', agentName: 'Vitalis', domain: 'Health', status: 'waiting', progress: 0, message: 'Waiting for Chronos to finish'},
];

export function computeLifeScore(scores: DomainScore[]): number {
  const weights = {Finance: 0.3, Health: 0.25, Time: 0.25, Safety: 0.2};
  let total = 0;
  for (const s of scores) {
    total += s.score * (weights[s.domain] || 0.25);
  }
  return Math.round(total);
}
