import React from 'react';
import {View, Text, StyleSheet, TouchableOpacity} from 'react-native';
import Icon from 'react-native-vector-icons/MaterialIcons';
import {colors, spacing, radius, fontSize} from '../theme';
import {domainColors} from '../theme';
import type {Agent} from '../types';

interface Props {
  agent: Agent;
  onPress?: () => void;
  compact?: boolean;
}

export default function AgentCard({agent, onPress, compact}: Props) {
  const domainColor = agent.domain === 'General'
    ? colors.violet
    : domainColors[agent.domain as keyof typeof domainColors] || colors.cyan;

  if (agent.locked) {
    return (
      <TouchableOpacity style={[styles.card, styles.lockedCard, compact && styles.compact]} onPress={onPress}>
        <View style={styles.lockedIcon}>
          <Icon name="lock" size={24} color={colors.textMuted} />
        </View>
        <Text style={styles.lockedName}>{agent.name}</Text>
        <Text style={styles.lockedLevel}>Locked</Text>
      </TouchableOpacity>
    );
  }

  const xpPercent = (agent.xp / agent.xpToNext) * 100;

  return (
    <TouchableOpacity
      style={[styles.card, compact && styles.compact, {borderColor: domainColor + '40'}]}
      onPress={onPress}
      activeOpacity={0.7}>
      <View style={[styles.iconCircle, {backgroundColor: domainColor + '20'}]}>
        <Icon name={agent.icon} size={compact ? 20 : 24} color={domainColor} />
      </View>
      <Text style={styles.name} numberOfLines={1}>{agent.name}</Text>
      <View style={styles.levelRow}>
        <Text style={[styles.level, {color: domainColor}]}>Lv.{agent.level}</Text>
        <Text style={styles.domain}>{agent.domain}</Text>
      </View>
      <View style={styles.xpBar}>
        <View style={[styles.xpFill, {width: `${xpPercent}%`, backgroundColor: domainColor}]} />
      </View>
      {!compact && (
        <Text style={styles.tasks}>{agent.tasksCompleted} tasks</Text>
      )}
    </TouchableOpacity>
  );
}

const styles = StyleSheet.create({
  card: {
    backgroundColor: colors.surface,
    borderRadius: radius.md,
    padding: spacing.md,
    borderWidth: 1,
    borderColor: colors.surfaceBorder,
    alignItems: 'center',
    gap: spacing.xs,
  },
  compact: {
    padding: spacing.sm,
  },
  lockedCard: {
    opacity: 0.5,
  },
  iconCircle: {
    width: 44,
    height: 44,
    borderRadius: 22,
    alignItems: 'center',
    justifyContent: 'center',
  },
  lockedIcon: {
    width: 44,
    height: 44,
    borderRadius: 22,
    backgroundColor: colors.surfaceLight,
    alignItems: 'center',
    justifyContent: 'center',
  },
  name: {
    color: colors.text,
    fontSize: fontSize.md,
    fontWeight: '700',
  },
  lockedName: {
    color: colors.textMuted,
    fontSize: fontSize.md,
    fontWeight: '600',
  },
  lockedLevel: {
    color: colors.textMuted,
    fontSize: fontSize.xs,
  },
  levelRow: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: spacing.xs,
  },
  level: {
    fontSize: fontSize.sm,
    fontWeight: '800',
  },
  domain: {
    color: colors.textMuted,
    fontSize: fontSize.xs,
  },
  xpBar: {
    width: '100%',
    height: 3,
    backgroundColor: colors.surfaceLight,
    borderRadius: 2,
  },
  xpFill: {
    height: 3,
    borderRadius: 2,
  },
  tasks: {
    color: colors.textMuted,
    fontSize: fontSize.xs,
  },
});
