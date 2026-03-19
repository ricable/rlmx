import React from 'react';
import {View, Text, StyleSheet} from 'react-native';
import Icon from 'react-native-vector-icons/MaterialIcons';
import {colors, spacing, radius, fontSize} from '../theme';
import {domainColors} from '../theme';
import type {AgentProgress} from '../types';

interface Props {
  progress: AgentProgress;
}

export default function ProgressCard({progress}: Props) {
  const domainColor = progress.domain === 'General'
    ? colors.violet
    : domainColors[progress.domain as keyof typeof domainColors] || colors.cyan;

  const statusIcon =
    progress.status === 'complete' ? 'check-circle' :
    progress.status === 'working' ? 'sync' : 'hourglass-empty';

  const statusColor =
    progress.status === 'complete' ? colors.green :
    progress.status === 'working' ? colors.cyan : colors.textMuted;

  return (
    <View style={[styles.card, {borderLeftColor: domainColor}]}>
      <View style={styles.header}>
        <View style={styles.nameRow}>
          <View style={[styles.dot, {backgroundColor: domainColor}]} />
          <Text style={styles.name}>{progress.agentName}</Text>
        </View>
        <Icon name={statusIcon} size={16} color={statusColor} />
      </View>
      <Text style={styles.message} numberOfLines={1}>{progress.message}</Text>
      <View style={styles.barContainer}>
        <View
          style={[styles.barFill, {width: `${progress.progress}%`, backgroundColor: domainColor}]}
        />
      </View>
    </View>
  );
}

const styles = StyleSheet.create({
  card: {
    backgroundColor: colors.surface,
    borderRadius: radius.sm,
    padding: spacing.sm,
    borderLeftWidth: 3,
    gap: 4,
  },
  header: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
  },
  nameRow: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: spacing.xs,
  },
  dot: {
    width: 6,
    height: 6,
    borderRadius: 3,
  },
  name: {
    color: colors.text,
    fontSize: fontSize.sm,
    fontWeight: '700',
  },
  message: {
    color: colors.textSecondary,
    fontSize: fontSize.xs,
  },
  barContainer: {
    height: 3,
    backgroundColor: colors.surfaceLight,
    borderRadius: 2,
  },
  barFill: {
    height: 3,
    borderRadius: 2,
  },
});
