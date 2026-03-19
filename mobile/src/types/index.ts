export interface Agent {
  id: string;
  name: string;
  domain: 'Finance' | 'Health' | 'Time' | 'Safety' | 'General';
  icon: string;
  level: number;
  xp: number;
  xpToNext: number;
  description: string;
  locked: boolean;
  tasksCompleted: number;
  successRate: number;
}

export interface DomainScore {
  domain: 'Finance' | 'Health' | 'Time' | 'Safety';
  score: number;
  trend: number[];
  change: number;
  metrics: DomainMetric[];
  suggestions: string[];
}

export interface DomainMetric {
  label: string;
  value: string;
  change?: number;
}

export interface BriefingItem {
  id: string;
  title: string;
  subtitle: string;
  domain: 'Finance' | 'Health' | 'Time' | 'Safety';
  icon: string;
  actionLabel?: string;
}

export interface Achievement {
  id: string;
  name: string;
  description: string;
  icon: string;
  unlocked: boolean;
  unlockedAt?: string;
  category: string;
}

export interface VoiceMessage {
  id: string;
  role: 'user' | 'assistant';
  text: string;
  timestamp: number;
}

export interface AgentProgress {
  agentId: string;
  agentName: string;
  domain: 'Finance' | 'Health' | 'Time' | 'Safety' | 'General';
  status: 'working' | 'complete' | 'waiting';
  progress: number;
  message: string;
}

export interface UserProfile {
  name: string;
  tier: 'Free' | 'Plus' | 'Pro';
  level: number;
  xp: number;
  xpToNext: number;
  streak: number;
  longestStreak: number;
  joinedDate: string;
  achievementsUnlocked: number;
  totalAchievements: number;
}

export interface AppState {
  user: UserProfile;
  agents: Agent[];
  lifeScore: number;
  domainScores: DomainScore[];
  moneySaved: number;
  briefing: BriefingItem[];
  achievements: Achievement[];
  voiceMessages: VoiceMessage[];
  agentProgress: AgentProgress[];
  isListening: boolean;
  isConnected: boolean;
}
