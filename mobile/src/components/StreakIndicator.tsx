import React from 'react';
import {View, Text, StyleSheet} from 'react-native';
import Icon from 'react-native-vector-icons/MaterialIcons';
import {colors, spacing, radius, fontSize} from '../theme';

interface Props {
  streak: number;
  longestStreak: number;
}

export default function StreakIndicator({streak, longestStreak}: Props) {
  const flameColor = streak >= 14 ? colors.amber : streak >= 7 ? '#fb923c' : colors.textMuted;

  return (
    <View style={styles.card}>
      <View style={styles.row}>
        <Icon name="local-fire-department" size={28} color={flameColor} />
        <View>
          <Text style={styles.count}>{streak} day streak</Text>
          <Text style={styles.best}>Best: {longestStreak} days</Text>
        </View>
      </View>
      <View style={styles.dotsRow}>
        {Array.from({length: 14}, (_, i) => (
          <View
            key={i}
            style={[
              styles.dot,
              {
                backgroundColor: i < streak % 14 || streak >= 14
                  ? flameColor
                  : colors.surfaceLight,
              },
            ]}
          />
        ))}
      </View>
      <Text style={styles.reward}>
        {streak >= 14 ? 'Reward unlocked! Tap to claim' : `${14 - streak} days to next reward`}
      </Text>
    </View>
  );
}

const styles = StyleSheet.create({
  card: {
    backgroundColor: colors.surface,
    borderRadius: radius.lg,
    padding: spacing.md,
    borderWidth: 1,
    borderColor: colors.amberDim,
    gap: spacing.sm,
  },
  row: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: spacing.sm,
  },
  count: {
    color: colors.text,
    fontSize: fontSize.lg,
    fontWeight: '700',
  },
  best: {
    color: colors.textMuted,
    fontSize: fontSize.xs,
  },
  dotsRow: {
    flexDirection: 'row',
    gap: 4,
    justifyContent: 'center',
  },
  dot: {
    width: 8,
    height: 8,
    borderRadius: 4,
  },
  reward: {
    color: colors.amber,
    fontSize: fontSize.xs,
    textAlign: 'center',
  },
});
