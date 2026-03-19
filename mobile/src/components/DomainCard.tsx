import React from 'react';
import {View, Text, StyleSheet} from 'react-native';
import Icon from 'react-native-vector-icons/MaterialIcons';
import {colors, spacing, radius, fontSize} from '../theme';
import {domainColors} from '../theme';
import SparklineChart from './SparklineChart';
import type {DomainScore} from '../types';

const domainIcons: Record<string, string> = {
  Finance: 'account-balance-wallet',
  Health: 'favorite',
  Time: 'schedule',
  Safety: 'shield',
};

interface Props {
  domain: DomainScore;
}

export default function DomainCard({domain}: Props) {
  const color = domainColors[domain.domain];
  const changePrefix = domain.change > 0 ? '+' : '';

  return (
    <View style={[styles.card, {borderColor: color + '30'}]}>
      <View style={styles.header}>
        <View style={styles.titleRow}>
          <View style={[styles.iconCircle, {backgroundColor: color + '20'}]}>
            <Icon name={domainIcons[domain.domain] || 'help'} size={20} color={color} />
          </View>
          <View>
            <Text style={styles.domainName}>{domain.domain}</Text>
            <Text style={[styles.change, {color: domain.change >= 0 ? colors.green : colors.red}]}>
              {changePrefix}{domain.change} pts
            </Text>
          </View>
        </View>
        <View style={styles.scoreCol}>
          <Text style={[styles.score, {color}]}>{domain.score}</Text>
          <SparklineChart data={domain.trend} color={color} width={60} height={20} />
        </View>
      </View>

      <View style={styles.metrics}>
        {domain.metrics.map((m, i) => (
          <View key={i} style={styles.metricRow}>
            <Text style={styles.metricLabel}>{m.label}</Text>
            <View style={styles.metricValueRow}>
              <Text style={styles.metricValue}>{m.value}</Text>
              {m.change !== undefined && (
                <Text style={[styles.metricChange, {color: m.change >= 0 ? colors.green : colors.red}]}>
                  {m.change > 0 ? '+' : ''}{m.change}%
                </Text>
              )}
            </View>
          </View>
        ))}
      </View>

      {domain.suggestions.length > 0 && (
        <View style={styles.suggestions}>
          {domain.suggestions.map((s, i) => (
            <View key={i} style={styles.suggestionRow}>
              <Icon name="lightbulb" size={14} color={colors.amber} />
              <Text style={styles.suggestionText}>{s}</Text>
            </View>
          ))}
        </View>
      )}
    </View>
  );
}

const styles = StyleSheet.create({
  card: {
    backgroundColor: colors.surface,
    borderRadius: radius.lg,
    padding: spacing.md,
    borderWidth: 1,
    gap: spacing.md,
  },
  header: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'flex-start',
  },
  titleRow: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: spacing.sm,
  },
  iconCircle: {
    width: 40,
    height: 40,
    borderRadius: 20,
    alignItems: 'center',
    justifyContent: 'center',
  },
  domainName: {
    color: colors.text,
    fontSize: fontSize.lg,
    fontWeight: '700',
  },
  change: {
    fontSize: fontSize.xs,
    fontWeight: '600',
  },
  scoreCol: {
    alignItems: 'flex-end',
    gap: spacing.xs,
  },
  score: {
    fontSize: fontSize.xxl,
    fontWeight: '800',
  },
  metrics: {
    gap: spacing.xs,
  },
  metricRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    paddingVertical: 2,
  },
  metricLabel: {
    color: colors.textSecondary,
    fontSize: fontSize.sm,
  },
  metricValueRow: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: spacing.xs,
  },
  metricValue: {
    color: colors.text,
    fontSize: fontSize.sm,
    fontWeight: '600',
  },
  metricChange: {
    fontSize: fontSize.xs,
    fontWeight: '600',
  },
  suggestions: {
    gap: spacing.xs,
    paddingTop: spacing.sm,
    borderTopWidth: 1,
    borderTopColor: colors.surfaceBorder,
  },
  suggestionRow: {
    flexDirection: 'row',
    gap: spacing.xs,
    alignItems: 'flex-start',
  },
  suggestionText: {
    color: colors.textSecondary,
    fontSize: fontSize.xs,
    flex: 1,
    lineHeight: 16,
  },
});
