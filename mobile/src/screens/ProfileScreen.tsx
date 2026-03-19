import React from 'react';
import {View, Text, ScrollView, StyleSheet, TouchableOpacity} from 'react-native';
import Icon from 'react-native-vector-icons/MaterialIcons';
import {colors, spacing, radius, fontSize, screen} from '../theme';
import {useApp} from '../context/AppContext';
import AchievementBadge from '../components/AchievementBadge';

export default function ProfileScreen() {
  const {state} = useApp();
  const {user, achievements} = state;
  const xpPercent = (user.xp / user.xpToNext) * 100;
  const cols = screen.isTablet ? 5 : 4;

  return (
    <View style={styles.container}>
      <ScrollView contentContainerStyle={styles.scrollContent} showsVerticalScrollIndicator={false}>
        {/* Profile Header */}
        <View style={styles.profileHeader}>
          <View style={styles.avatar}>
            <Text style={styles.avatarText}>{user.name.charAt(0)}</Text>
          </View>
          <Text style={styles.userName}>{user.name}</Text>
          <View style={styles.tierBadge}>
            <Icon name="diamond" size={14} color={colors.violet} />
            <Text style={styles.tierText}>{user.tier}</Text>
          </View>
        </View>

        {/* Level */}
        <View style={styles.levelCard}>
          <View style={styles.levelHeader}>
            <Text style={styles.levelLabel}>Level {user.level}</Text>
            <Text style={styles.xpText}>{user.xp} / {user.xpToNext} XP</Text>
          </View>
          <View style={styles.xpBar}>
            <View style={[styles.xpFill, {width: `${xpPercent}%`}]} />
          </View>
        </View>

        {/* Stats */}
        <View style={styles.statsRow}>
          <View style={styles.statCard}>
            <Icon name="local-fire-department" size={24} color={colors.amber} />
            <Text style={styles.statValue}>{user.streak}</Text>
            <Text style={styles.statLabel}>Day Streak</Text>
          </View>
          <View style={styles.statCard}>
            <Icon name="emoji-events" size={24} color={colors.violet} />
            <Text style={styles.statValue}>{user.achievementsUnlocked}/{user.totalAchievements}</Text>
            <Text style={styles.statLabel}>Achievements</Text>
          </View>
          <View style={styles.statCard}>
            <Icon name="calendar-today" size={24} color={colors.cyan} />
            <Text style={styles.statValue}>{Math.floor((Date.now() - new Date(user.joinedDate).getTime()) / 86400000)}</Text>
            <Text style={styles.statLabel}>Days Active</Text>
          </View>
        </View>

        {/* Streak Calendar */}
        <View style={styles.section}>
          <Text style={styles.sectionTitle}>Streak Calendar</Text>
          <View style={styles.calendarCard}>
            <View style={styles.calendarGrid}>
              {Array.from({length: 28}, (_, i) => {
                const active = i >= 28 - user.streak;
                return (
                  <View
                    key={i}
                    style={[
                      styles.calDay,
                      active && styles.calDayActive,
                    ]}
                  />
                );
              })}
            </View>
            <View style={styles.calLegend}>
              <View style={styles.calLegendItem}>
                <View style={[styles.calDay, styles.calDaySmall]} />
                <Text style={styles.calLegendText}>Inactive</Text>
              </View>
              <View style={styles.calLegendItem}>
                <View style={[styles.calDay, styles.calDayActive, styles.calDaySmall]} />
                <Text style={styles.calLegendText}>Active</Text>
              </View>
            </View>
          </View>
        </View>

        {/* Achievements */}
        <View style={styles.section}>
          <View style={styles.sectionHeader}>
            <Text style={styles.sectionTitle}>Achievements</Text>
            <Text style={styles.sectionCount}>
              {achievements.filter(a => a.unlocked).length}/{achievements.length}
            </Text>
          </View>
          <View style={styles.achievementGrid}>
            {achievements.map(ach => (
              <View key={ach.id} style={{width: (screen.width - spacing.md * 2 - spacing.xs * (cols - 1)) / cols}}>
                <AchievementBadge achievement={ach} />
              </View>
            ))}
          </View>
        </View>

        {/* Settings */}
        <View style={styles.section}>
          <Text style={styles.sectionTitle}>Settings</Text>
          <View style={styles.settingsCard}>
            {[
              {icon: 'notifications', label: 'Notifications', value: 'On'},
              {icon: 'mic', label: 'Voice Language', value: 'English'},
              {icon: 'volume-up', label: 'Voice Feedback', value: 'On'},
              {icon: 'dark-mode', label: 'Theme', value: 'Dark'},
              {icon: 'privacy-tip', label: 'Privacy', value: ''},
              {icon: 'info', label: 'About RuVix', value: 'v1.0.0'},
            ].map((item, i) => (
              <TouchableOpacity key={i} style={styles.settingRow}>
                <View style={styles.settingLeft}>
                  <Icon name={item.icon} size={20} color={colors.textSecondary} />
                  <Text style={styles.settingLabel}>{item.label}</Text>
                </View>
                <View style={styles.settingRight}>
                  <Text style={styles.settingValue}>{item.value}</Text>
                  <Icon name="chevron-right" size={20} color={colors.textMuted} />
                </View>
              </TouchableOpacity>
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
  scrollContent: {
    padding: spacing.md,
  },
  profileHeader: {
    alignItems: 'center',
    paddingTop: spacing.lg,
    gap: spacing.sm,
  },
  avatar: {
    width: 80,
    height: 80,
    borderRadius: 40,
    backgroundColor: colors.cyan,
    alignItems: 'center',
    justifyContent: 'center',
  },
  avatarText: {
    color: colors.white,
    fontSize: fontSize.hero,
    fontWeight: '800',
  },
  userName: {
    color: colors.text,
    fontSize: fontSize.xxl,
    fontWeight: '800',
  },
  tierBadge: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 4,
    backgroundColor: colors.violetDim,
    paddingHorizontal: spacing.sm,
    paddingVertical: spacing.xs,
    borderRadius: radius.full,
  },
  tierText: {
    color: colors.violet,
    fontSize: fontSize.sm,
    fontWeight: '700',
  },
  levelCard: {
    backgroundColor: colors.surface,
    borderRadius: radius.lg,
    padding: spacing.md,
    marginTop: spacing.lg,
    gap: spacing.sm,
    borderWidth: 1,
    borderColor: colors.surfaceBorder,
  },
  levelHeader: {
    flexDirection: 'row',
    justifyContent: 'space-between',
  },
  levelLabel: {
    color: colors.text,
    fontSize: fontSize.lg,
    fontWeight: '700',
  },
  xpText: {
    color: colors.textMuted,
    fontSize: fontSize.sm,
  },
  xpBar: {
    height: 8,
    backgroundColor: colors.surfaceLight,
    borderRadius: 4,
  },
  xpFill: {
    height: 8,
    borderRadius: 4,
    backgroundColor: colors.cyan,
  },
  statsRow: {
    flexDirection: 'row',
    gap: spacing.sm,
    marginTop: spacing.md,
  },
  statCard: {
    flex: 1,
    backgroundColor: colors.surface,
    borderRadius: radius.md,
    padding: spacing.md,
    alignItems: 'center',
    gap: 4,
    borderWidth: 1,
    borderColor: colors.surfaceBorder,
  },
  statValue: {
    color: colors.text,
    fontSize: fontSize.lg,
    fontWeight: '800',
  },
  statLabel: {
    color: colors.textMuted,
    fontSize: fontSize.xs,
    textAlign: 'center',
  },
  section: {
    marginTop: spacing.lg,
  },
  sectionHeader: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'baseline',
    marginBottom: spacing.sm,
  },
  sectionTitle: {
    color: colors.text,
    fontSize: fontSize.lg,
    fontWeight: '700',
    marginBottom: spacing.sm,
  },
  sectionCount: {
    color: colors.textMuted,
    fontSize: fontSize.sm,
    marginBottom: spacing.sm,
  },
  calendarCard: {
    backgroundColor: colors.surface,
    borderRadius: radius.lg,
    padding: spacing.md,
    borderWidth: 1,
    borderColor: colors.surfaceBorder,
    gap: spacing.sm,
  },
  calendarGrid: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    gap: 4,
    justifyContent: 'center',
  },
  calDay: {
    width: 32,
    height: 32,
    borderRadius: 6,
    backgroundColor: colors.surfaceLight,
  },
  calDayActive: {
    backgroundColor: colors.cyan,
  },
  calDaySmall: {
    width: 12,
    height: 12,
    borderRadius: 3,
  },
  calLegend: {
    flexDirection: 'row',
    justifyContent: 'center',
    gap: spacing.md,
  },
  calLegendItem: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 4,
  },
  calLegendText: {
    color: colors.textMuted,
    fontSize: fontSize.xs,
  },
  achievementGrid: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    gap: spacing.xs,
  },
  settingsCard: {
    backgroundColor: colors.surface,
    borderRadius: radius.lg,
    borderWidth: 1,
    borderColor: colors.surfaceBorder,
    overflow: 'hidden',
  },
  settingRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    padding: spacing.md,
    borderBottomWidth: 1,
    borderBottomColor: colors.surfaceBorder,
  },
  settingLeft: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: spacing.sm,
  },
  settingLabel: {
    color: colors.text,
    fontSize: fontSize.md,
  },
  settingRight: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 4,
  },
  settingValue: {
    color: colors.textMuted,
    fontSize: fontSize.sm,
  },
  bottomPad: {
    height: 100,
  },
});
