/**
 * useEngagement — React hook for gamification and engagement features.
 *
 * Tracks life score, savings, streaks, and achievements to drive
 * user retention in the RuVix mobile app.
 */

import { useCallback, useEffect, useRef, useState } from 'react';

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

export interface Achievement {
  id: string;
  title: string;
  description: string;
  icon: string;
  unlockedAt: number | null;
}

export interface SavingsEntry {
  amount: number;
  source: string;
  timestamp: number;
}

export interface UseEngagementResult {
  lifeScore: number;
  moneySaved: number;
  displayedMoneySaved: number; // animated counter value
  streak: number;
  achievements: Achievement[];
  savingsHistory: SavingsEntry[];
  updateScore: (domain: string, delta: number) => void;
  addSavings: (amount: number, source: string) => void;
  checkStreak: () => void;
}

// ---------------------------------------------------------------------------
// Default achievements
// ---------------------------------------------------------------------------

const DEFAULT_ACHIEVEMENTS: Achievement[] = [
  {
    id: 'first_query',
    title: 'First Words',
    description: 'Made your first voice query',
    icon: 'mic',
    unlockedAt: null,
  },
  {
    id: 'savings_100',
    title: 'Penny Pincher',
    description: 'Saved $100 with agent recommendations',
    icon: 'piggy-bank',
    unlockedAt: null,
  },
  {
    id: 'savings_500',
    title: 'Smart Saver',
    description: 'Saved $500 with agent recommendations',
    icon: 'trending-up',
    unlockedAt: null,
  },
  {
    id: 'streak_7',
    title: 'Week Warrior',
    description: 'Used RuVix for 7 consecutive days',
    icon: 'flame',
    unlockedAt: null,
  },
  {
    id: 'streak_30',
    title: 'Monthly Master',
    description: 'Used RuVix for 30 consecutive days',
    icon: 'trophy',
    unlockedAt: null,
  },
  {
    id: 'all_domains',
    title: 'Life Manager',
    description: 'Interacted with all 6 life domains',
    icon: 'grid',
    unlockedAt: null,
  },
  {
    id: 'score_80',
    title: 'High Performer',
    description: 'Reached a Life Score of 80+',
    icon: 'star',
    unlockedAt: null,
  },
];

// ---------------------------------------------------------------------------
// Persistence helpers (AsyncStorage stubs for demo)
// ---------------------------------------------------------------------------

const STORAGE_KEY = 'ruvix_engagement';

interface PersistedState {
  lifeScore: number;
  moneySaved: number;
  streak: number;
  lastActiveDate: string;
  achievements: Achievement[];
  savingsHistory: SavingsEntry[];
  domainInteractions: Set<string>;
}

function todayString(): string {
  return new Date().toISOString().slice(0, 10);
}

// ---------------------------------------------------------------------------
// Hook
// ---------------------------------------------------------------------------

export function useEngagement(): UseEngagementResult {
  const [lifeScore, setLifeScore] = useState(42);
  const [moneySaved, setMoneySaved] = useState(0);
  const [displayedMoneySaved, setDisplayedMoneySaved] = useState(0);
  const [streak, setStreak] = useState(1);
  const [achievements, setAchievements] = useState<Achievement[]>(
    DEFAULT_ACHIEVEMENTS.map((a) => ({ ...a })),
  );
  const [savingsHistory, setSavingsHistory] = useState<SavingsEntry[]>([]);
  const domainInteractions = useRef(new Set<string>());
  const lastActiveDate = useRef(todayString());
  const animationRef = useRef<number | null>(null);

  // -----------------------------------------------------------------------
  // Animated counter for moneySaved
  // -----------------------------------------------------------------------

  useEffect(() => {
    if (displayedMoneySaved === moneySaved) return;

    const startValue = displayedMoneySaved;
    const diff = moneySaved - startValue;
    const duration = 800; // ms
    const startTime = Date.now();

    const animate = () => {
      const elapsed = Date.now() - startTime;
      const progress = Math.min(elapsed / duration, 1);
      // Ease-out cubic
      const eased = 1 - Math.pow(1 - progress, 3);
      const current = startValue + diff * eased;
      setDisplayedMoneySaved(Math.round(current * 100) / 100);

      if (progress < 1) {
        animationRef.current = requestAnimationFrame(animate);
      } else {
        setDisplayedMoneySaved(moneySaved);
      }
    };

    animationRef.current = requestAnimationFrame(animate);

    return () => {
      if (animationRef.current !== null) {
        cancelAnimationFrame(animationRef.current);
      }
    };
    // We intentionally only trigger on moneySaved changes
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [moneySaved]);

  // -----------------------------------------------------------------------
  // Achievement checking
  // -----------------------------------------------------------------------

  const unlockAchievement = useCallback(
    (id: string) => {
      setAchievements((prev) =>
        prev.map((a) =>
          a.id === id && a.unlockedAt === null
            ? { ...a, unlockedAt: Date.now() }
            : a,
        ),
      );
    },
    [],
  );

  const checkAchievements = useCallback(
    (
      newScore: number,
      newSaved: number,
      newStreak: number,
      domains: Set<string>,
    ) => {
      if (newSaved >= 100) unlockAchievement('savings_100');
      if (newSaved >= 500) unlockAchievement('savings_500');
      if (newStreak >= 7) unlockAchievement('streak_7');
      if (newStreak >= 30) unlockAchievement('streak_30');
      if (newScore >= 80) unlockAchievement('score_80');
      if (domains.size >= 6) unlockAchievement('all_domains');
    },
    [unlockAchievement],
  );

  // -----------------------------------------------------------------------
  // Public API
  // -----------------------------------------------------------------------

  const updateScore = useCallback(
    (domain: string, delta: number) => {
      domainInteractions.current.add(domain);
      unlockAchievement('first_query');

      setLifeScore((prev) => {
        const next = Math.max(0, Math.min(100, prev + delta));
        checkAchievements(
          next,
          moneySaved,
          streak,
          domainInteractions.current,
        );
        return next;
      });
    },
    [moneySaved, streak, checkAchievements, unlockAchievement],
  );

  const addSavings = useCallback(
    (amount: number, source: string) => {
      const entry: SavingsEntry = {
        amount,
        source,
        timestamp: Date.now(),
      };
      setSavingsHistory((prev) => [...prev, entry]);
      setMoneySaved((prev) => {
        const next = prev + amount;
        checkAchievements(
          lifeScore,
          next,
          streak,
          domainInteractions.current,
        );
        return next;
      });
    },
    [lifeScore, streak, checkAchievements],
  );

  const checkStreak = useCallback(() => {
    const today = todayString();
    if (today === lastActiveDate.current) return;

    const lastDate = new Date(lastActiveDate.current);
    const todayDate = new Date(today);
    const diffDays = Math.floor(
      (todayDate.getTime() - lastDate.getTime()) / (1000 * 60 * 60 * 24),
    );

    if (diffDays === 1) {
      setStreak((prev) => {
        const next = prev + 1;
        checkAchievements(
          lifeScore,
          moneySaved,
          next,
          domainInteractions.current,
        );
        return next;
      });
    } else if (diffDays > 1) {
      setStreak(1);
    }

    lastActiveDate.current = today;
  }, [lifeScore, moneySaved, checkAchievements]);

  return {
    lifeScore,
    moneySaved,
    displayedMoneySaved,
    streak,
    achievements,
    savingsHistory,
    updateScore,
    addSavings,
    checkStreak,
  };
}
