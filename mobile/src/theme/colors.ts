export const colors = {
  background: '#0a0e1a',
  surface: '#141824',
  surfaceLight: '#1e2436',
  surfaceBorder: '#2a3040',
  cyan: '#00e5ff',
  cyanDim: '#00e5ff40',
  violet: '#a855f7',
  violetDim: '#a855f740',
  green: '#22c55e',
  greenDim: '#22c55e40',
  amber: '#f59e0b',
  amberDim: '#f59e0b40',
  red: '#ef4444',
  redDim: '#ef444440',
  pink: '#ec4899',
  pinkDim: '#ec489940',
  text: '#e2e8f0',
  textSecondary: '#94a3b8',
  textMuted: '#64748b',
  white: '#ffffff',
  black: '#000000',
  transparent: 'transparent',
} as const;

export const domainColors = {
  Finance: colors.green,
  Health: colors.pink,
  Time: colors.cyan,
  Safety: colors.amber,
  Legal: colors.amber,
  Career: '#3b82f6',
  Education: colors.violet,
  Home: colors.cyan,
  Shopping: '#f97316',
  Travel: '#14b8a6',
  Social: colors.violet,
  Government: '#64748b',
  Automotive: colors.red,
  Pet: '#84cc16',
} as const;

export type DomainKey = keyof typeof domainColors;

export const gradients = {
  primary: [colors.cyan, colors.violet],
  success: [colors.green, '#10b981'],
  surface: [colors.surface, colors.background],
  card: ['#1a2030', '#141824'],
} as const;
