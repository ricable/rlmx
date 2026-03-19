import React from 'react';
import {View, Text, StyleSheet} from 'react-native';
import Icon from 'react-native-vector-icons/MaterialIcons';
import {colors, spacing, radius, fontSize} from '../theme';
import type {Achievement} from '../types';

interface Props {
  achievement: Achievement;
}

export default function AchievementBadge({achievement}: Props) {
  return (
    <View style={[styles.badge, !achievement.unlocked && styles.locked]}>
      <View style={[styles.iconCircle, {backgroundColor: achievement.unlocked ? colors.violetDim : colors.surfaceLight}]}>
        <Icon
          name={achievement.icon}
          size={20}
          color={achievement.unlocked ? colors.violet : colors.textMuted}
        />
      </View>
      <Text style={[styles.name, !achievement.unlocked && styles.lockedText]} numberOfLines={1}>
        {achievement.name}
      </Text>
      <Text style={styles.desc} numberOfLines={2}>
        {achievement.description}
      </Text>
    </View>
  );
}

const styles = StyleSheet.create({
  badge: {
    alignItems: 'center',
    padding: spacing.sm,
    gap: 4,
  },
  locked: {
    opacity: 0.4,
  },
  iconCircle: {
    width: 44,
    height: 44,
    borderRadius: 22,
    alignItems: 'center',
    justifyContent: 'center',
  },
  name: {
    color: colors.text,
    fontSize: fontSize.xs,
    fontWeight: '700',
    textAlign: 'center',
  },
  lockedText: {
    color: colors.textMuted,
  },
  desc: {
    color: colors.textMuted,
    fontSize: 9,
    textAlign: 'center',
    lineHeight: 12,
  },
});
