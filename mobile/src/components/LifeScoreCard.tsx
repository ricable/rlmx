import React from 'react';
import {View, Text, StyleSheet} from 'react-native';
import {colors, spacing, radius, fontSize} from '../theme';
import type {DomainScore} from '../types';
import {domainColors} from '../theme';

interface Props {
  score: number;
  domainScores: DomainScore[];
}

export default function LifeScoreCard({score, domainScores}: Props) {
  return (
    <View style={styles.card}>
      <View style={styles.header}>
        <Text style={styles.label}>Life Score</Text>
        <View style={styles.scoreContainer}>
          <Text style={styles.score}>{score}</Text>
          <Text style={styles.scoreMax}>/100</Text>
        </View>
      </View>

      <View style={styles.ring}>
        <View style={styles.ringInner}>
          <Text style={styles.ringScore}>{score}</Text>
          <Text style={styles.ringLabel}>Overall</Text>
        </View>
      </View>

      <View style={styles.domains}>
        {domainScores.map(ds => (
          <View key={ds.domain} style={styles.domainRow}>
            <View style={styles.domainInfo}>
              <View style={[styles.domainDot, {backgroundColor: domainColors[ds.domain]}]} />
              <Text style={styles.domainName}>{ds.domain}</Text>
            </View>
            <View style={styles.barContainer}>
              <View
                style={[
                  styles.barFill,
                  {
                    width: `${ds.score}%`,
                    backgroundColor: domainColors[ds.domain],
                  },
                ]}
              />
            </View>
            <Text style={[styles.domainScore, {color: domainColors[ds.domain]}]}>
              {ds.score}
            </Text>
          </View>
        ))}
      </View>
    </View>
  );
}

const styles = StyleSheet.create({
  card: {
    backgroundColor: colors.surface,
    borderRadius: radius.lg,
    padding: spacing.lg,
    borderWidth: 1,
    borderColor: colors.surfaceBorder,
  },
  header: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: spacing.md,
  },
  label: {
    color: colors.textSecondary,
    fontSize: fontSize.md,
    fontWeight: '600',
  },
  scoreContainer: {
    flexDirection: 'row',
    alignItems: 'baseline',
  },
  score: {
    color: colors.cyan,
    fontSize: fontSize.xxl,
    fontWeight: '800',
  },
  scoreMax: {
    color: colors.textMuted,
    fontSize: fontSize.md,
  },
  ring: {
    alignItems: 'center',
    marginVertical: spacing.md,
  },
  ringInner: {
    width: 100,
    height: 100,
    borderRadius: 50,
    borderWidth: 4,
    borderColor: colors.cyan,
    alignItems: 'center',
    justifyContent: 'center',
    backgroundColor: colors.cyanDim,
  },
  ringScore: {
    color: colors.white,
    fontSize: fontSize.hero,
    fontWeight: '800',
  },
  ringLabel: {
    color: colors.textSecondary,
    fontSize: fontSize.xs,
    marginTop: -4,
  },
  domains: {
    gap: spacing.sm,
    marginTop: spacing.md,
  },
  domainRow: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: spacing.sm,
  },
  domainInfo: {
    flexDirection: 'row',
    alignItems: 'center',
    width: 80,
    gap: spacing.xs,
  },
  domainDot: {
    width: 8,
    height: 8,
    borderRadius: 4,
  },
  domainName: {
    color: colors.text,
    fontSize: fontSize.sm,
  },
  barContainer: {
    flex: 1,
    height: 6,
    backgroundColor: colors.surfaceLight,
    borderRadius: 3,
  },
  barFill: {
    height: 6,
    borderRadius: 3,
  },
  domainScore: {
    width: 28,
    textAlign: 'right',
    fontSize: fontSize.sm,
    fontWeight: '700',
  },
});
