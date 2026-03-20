import React from 'react';
import {View, Text, ScrollView, StyleSheet, TouchableOpacity, StatusBar} from 'react-native';
import Icon from 'react-native-vector-icons/MaterialIcons';
import {colors, spacing, radius, fontSize} from '../theme';
import {useApp} from '../context/AppContext';
import LifeScoreCard from '../components/LifeScoreCard';
import MoneySavedCounter from '../components/MoneySavedCounter';
import StreakIndicator from '../components/StreakIndicator';
import BriefingCard from '../components/BriefingCard';

export default function HomeScreen({navigation}: {navigation: any}) {
  const {state} = useApp();

  const greeting = () => {
    const h = new Date().getHours();
    if (h < 12) {return 'Good morning';}
    if (h < 17) {return 'Good afternoon';}
    return 'Good evening';
  };

  return (
    <View style={styles.container}>
      <StatusBar barStyle="light-content" backgroundColor={colors.background} />
      <ScrollView
        style={styles.scroll}
        contentContainerStyle={styles.scrollContent}
        showsVerticalScrollIndicator={false}>
        {/* Header */}
        <View style={styles.header}>
          <View>
            <Text style={styles.greeting}>{greeting()},</Text>
            <Text style={styles.name}>{state.user.name}</Text>
          </View>
          <View style={styles.headerRight}>
            <View style={[styles.connectionDot, {backgroundColor: state.isConnected ? colors.green : colors.textMuted}]} />
            <TouchableOpacity style={styles.profileBtn}>
              <Icon name="person" size={22} color={colors.text} />
            </TouchableOpacity>
          </View>
        </View>

        {/* Quick Actions */}
        <View style={styles.quickActions}>
          <TouchableOpacity
            style={styles.voiceQuick}
            onPress={() => navigation.navigate('Voice')}
            activeOpacity={0.7}>
            <Icon name="mic" size={20} color={colors.white} />
            <Text style={styles.voiceQuickText}>Talk to RuVix</Text>
          </TouchableOpacity>
        </View>

        {/* Morning Briefing */}
        <View style={styles.section}>
          <Text style={styles.sectionTitle}>Today's Briefing</Text>
          <View style={styles.briefingList}>
            {state.briefing.map(item => (
              <BriefingCard key={item.id} item={item} />
            ))}
          </View>
        </View>

        {/* Life Score */}
        <View style={styles.section}>
          <Text style={styles.sectionTitle}>Life Score</Text>
          <LifeScoreCard score={state.lifeScore} domainScores={state.domainScores} />
        </View>

        {/* Money Saved */}
        <View style={styles.section}>
          <MoneySavedCounter amount={state.moneySaved} />
        </View>

        {/* Streak */}
        <View style={styles.section}>
          <StreakIndicator streak={state.user.streak} longestStreak={state.user.longestStreak} />
        </View>

        {/* Active Agents */}
        <View style={styles.section}>
          <View style={styles.sectionHeader}>
            <Text style={styles.sectionTitle}>Active Agents</Text>
            <TouchableOpacity onPress={() => navigation.navigate('Agents')}>
              <Text style={styles.seeAll}>See all</Text>
            </TouchableOpacity>
          </View>
          <View style={styles.agentChips}>
            {state.agents.filter(a => !a.locked).slice(0, 4).map(a => (
              <View key={a.id} style={styles.agentChip}>
                <Icon name={a.icon} size={16} color={colors.cyan} />
                <Text style={styles.agentChipText}>{a.name}</Text>
                <Text style={styles.agentChipLevel}>Lv.{a.level}</Text>
              </View>
            ))}
          </View>
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
  scroll: {
    flex: 1,
  },
  scrollContent: {
    padding: spacing.md,
  },
  header: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: spacing.lg,
    paddingTop: spacing.md,
  },
  greeting: {
    color: colors.textSecondary,
    fontSize: fontSize.md,
  },
  name: {
    color: colors.text,
    fontSize: fontSize.xxl,
    fontWeight: '800',
  },
  headerRight: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: spacing.sm,
  },
  connectionDot: {
    width: 8,
    height: 8,
    borderRadius: 4,
  },
  profileBtn: {
    width: 40,
    height: 40,
    borderRadius: 20,
    backgroundColor: colors.surface,
    alignItems: 'center',
    justifyContent: 'center',
    borderWidth: 1,
    borderColor: colors.surfaceBorder,
  },
  quickActions: {
    marginBottom: spacing.lg,
  },
  voiceQuick: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'center',
    gap: spacing.sm,
    backgroundColor: colors.cyan,
    paddingVertical: spacing.md,
    borderRadius: radius.xl,
    elevation: 4,
    shadowColor: colors.cyan,
    shadowOffset: {width: 0, height: 2},
    shadowOpacity: 0.3,
    shadowRadius: 8,
  },
  voiceQuickText: {
    color: colors.white,
    fontSize: fontSize.lg,
    fontWeight: '700',
  },
  section: {
    marginBottom: spacing.lg,
  },
  sectionHeader: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: spacing.sm,
  },
  sectionTitle: {
    color: colors.text,
    fontSize: fontSize.lg,
    fontWeight: '700',
    marginBottom: spacing.sm,
  },
  seeAll: {
    color: colors.cyan,
    fontSize: fontSize.sm,
    fontWeight: '600',
    marginBottom: spacing.sm,
  },
  briefingList: {
    gap: spacing.sm,
  },
  agentChips: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    gap: spacing.sm,
  },
  agentChip: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: spacing.xs,
    backgroundColor: colors.surface,
    paddingHorizontal: spacing.sm,
    paddingVertical: spacing.xs,
    borderRadius: radius.full,
    borderWidth: 1,
    borderColor: colors.surfaceBorder,
  },
  agentChipText: {
    color: colors.text,
    fontSize: fontSize.sm,
    fontWeight: '600',
  },
  agentChipLevel: {
    color: colors.textMuted,
    fontSize: fontSize.xs,
  },
  bottomPad: {
    height: 80,
  },
});
