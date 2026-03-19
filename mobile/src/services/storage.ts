import AsyncStorage from '@react-native-async-storage/async-storage';

const KEYS = {
  USER_PROFILE: '@rlmx_user_profile',
  SAVED_AGENTS: '@rlmx_saved_agents',
  LIFE_SCORE: '@rlmx_life_score',
  STREAK: '@rlmx_streak',
  MONEY_SAVED: '@rlmx_money_saved',
} as const;

export async function save<T>(key: string, value: T): Promise<void> {
  try {
    await AsyncStorage.setItem(key, JSON.stringify(value));
  } catch {
    // silently fail
  }
}

export async function load<T>(key: string): Promise<T | null> {
  try {
    const val = await AsyncStorage.getItem(key);
    return val ? JSON.parse(val) : null;
  } catch {
    return null;
  }
}

export async function remove(key: string): Promise<void> {
  try {
    await AsyncStorage.removeItem(key);
  } catch {
    // silently fail
  }
}

export {KEYS};
