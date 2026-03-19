import React from 'react';
import {View, Text, ScrollView, StyleSheet} from 'react-native';
import {colors, spacing, fontSize} from '../theme';
import {useApp} from '../context/AppContext';
import DomainCard from '../components/DomainCard';

export default function InsightsScreen() {
  const {state} = useApp();

  return (
    <View style={styles.container}>
      <View style={styles.header}>
        <Text style={styles.title}>Insights</Text>
        <Text style={styles.subtitle}>How your agents are improving your life</Text>
      </View>

      <ScrollView contentContainerStyle={styles.scrollContent} showsVerticalScrollIndicator={false}>
        <View style={styles.overviewRow}>
          <View style={styles.overviewCard}>
            <Text style={styles.overviewValue}>{state.lifeScore}</Text>
            <Text style={styles.overviewLabel}>Life Score</Text>
          </View>
          <View style={styles.overviewCard}>
            <Text style={[styles.overviewValue, {color: colors.green}]}>${state.moneySaved}</Text>
            <Text style={styles.overviewLabel}>Total Saved</Text>
          </View>
          <View style={styles.overviewCard}>
            <Text style={[styles.overviewValue, {color: colors.amber}]}>{state.user.streak}</Text>
            <Text style={styles.overviewLabel}>Day Streak</Text>
          </View>
        </View>

        <Text style={styles.sectionTitle}>Domain Scores</Text>
        <View style={styles.domainList}>
          {state.domainScores.map(ds => (
            <DomainCard key={ds.domain} domain={ds} />
          ))}
        </View>

        <Text style={styles.sectionTitle}>Recent Activity</Text>
        <View style={styles.activityList}>
          {[
            {time: '2h ago', text: 'Penny found a $23 discount on phone bill', domain: 'Finance'},
            {time: '4h ago', text: 'Chronos rescheduled 2 meetings for focus time', domain: 'Time'},
            {time: '6h ago', text: 'Sentinel scanned 3 breach databases', domain: 'Safety'},
            {time: '1d ago', text: 'Vitalis logged 7,200 steps and 7h sleep', domain: 'Health'},
            {time: '1d ago', text: 'Savvy applied coupon code saving $12', domain: 'Finance'},
          ].map((item, i) => (
            <View key={i} style={styles.activityItem}>
              <Text style={styles.activityTime}>{item.time}</Text>
              <Text style={styles.activityText}>{item.text}</Text>
            </View>
          ))}
        </View>
        <View style={styles.bottomPad} />
      </ScrollView>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: colors.background,
  },
  header: {
    paddingHorizontal: spacing.md,
    paddingTop: spacing.lg,
    paddingBottom: spacing.sm,
  },
  title: {
    color: colors.text,
    fontSize: fontSize.xxl,
    fontWeight: '800',
  },
  subtitle: {
    color: colors.textSecondary,
    fontSize: fontSize.md,
    marginTop: 2,
  },
  scrollContent: {
    padding: spacing.md,
  },
  overviewRow: {
    flexDirection: 'row',
    gap: spacing.sm,
    marginBottom: spacing.lg,
  },
  overviewCard: {
    flex: 1,
    backgroundColor: colors.surface,
    borderRadius: 12,
    padding: spacing.md,
    alignItems: 'center',
    gap: 4,
    borderWidth: 1,
    borderColor: colors.surfaceBorder,
  },
  overviewValue: {
    color: colors.cyan,
    fontSize: fontSize.xl,
    fontWeight: '800',
  },
  overviewLabel: {
    color: colors.textMuted,
    fontSize: fontSize.xs,
  },
  sectionTitle: {
    color: colors.text,
    fontSize: fontSize.lg,
    fontWeight: '700',
    marginBottom: spacing.sm,
    marginTop: spacing.md,
  },
  domainList: {
    gap: spacing.md,
  },
  activityList: {
    backgroundColor: colors.surface,
    borderRadius: 12,
    borderWidth: 1,
    borderColor: colors.surfaceBorder,
    overflow: 'hidden',
  },
  activityItem: {
    flexDirection: 'row',
    padding: spacing.md,
    gap: spacing.sm,
    borderBottomWidth: 1,
    borderBottomColor: colors.surfaceBorder,
  },
  activityTime: {
    color: colors.textMuted,
    fontSize: fontSize.xs,
    width: 48,
  },
  activityText: {
    color: colors.text,
    fontSize: fontSize.sm,
    flex: 1,
  },
  bottomPad: {
    height: 100,
  },
});
