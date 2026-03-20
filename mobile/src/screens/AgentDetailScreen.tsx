import React from 'react';
import {View, Text, ScrollView, StyleSheet, TouchableOpacity} from 'react-native';
import Icon from 'react-native-vector-icons/MaterialIcons';
import {colors, spacing, radius, fontSize} from '../theme';
import {domainColors} from '../theme';
import {useNavigation, useRoute} from '@react-navigation/native';

export default function AgentDetailScreen() {
  const navigation = useNavigation<any>();
  const route = useRoute<any>();
  const agent = route.params?.agent;

  if (!agent) {
    return (
      <View style={styles.container}>
        <Text style={styles.errorText}>Agent not found</Text>
      </View>
    );
  }

  const domainColor = (domainColors as Record<string, string>)[agent.domain] || colors.cyan;

  return (
    <View style={styles.container}>
      <View style={styles.header}>
        <TouchableOpacity onPress={() => navigation.goBack()} style={styles.backBtn}>
          <Icon name="arrow-back" size={24} color={colors.text} />
        </TouchableOpacity>
        <Text style={styles.title}>Agent Details</Text>
        <View style={styles.placeholder} />
      </View>

      <ScrollView contentContainerStyle={styles.scrollContent}>
        <View style={styles.heroSection}>
          <View style={[styles.iconLarge, {backgroundColor: domainColor + '20'}]}>
            <Icon name={agent.locked ? 'lock' : agent.icon} size={48} color={domainColor} />
          </View>
          <Text style={styles.name}>{agent.name}</Text>
          <Text style={[styles.domain, {color: domainColor}]}>{agent.domain}</Text>
          {agent.rating && (
            <View style={styles.ratingRow}>
              {[1, 2, 3, 4, 5].map((i: number) => (
                <Icon key={i} name="star" size={18} color={i <= Math.floor(agent.rating) ? colors.amber : colors.surfaceLight} />
              ))}
              <Text style={styles.ratingText}>{agent.rating}</Text>
              {agent.installs && <Text style={styles.installsText}>{agent.installs} installs</Text>}
            </View>
          )}
        </View>

        <View style={styles.descCard}>
          <Text style={styles.descTitle}>About</Text>
          <Text style={styles.descText}>{agent.description}</Text>
        </View>

        {agent.level !== undefined && (
          <View style={styles.statsRow}>
            <View style={styles.statBox}>
              <Text style={styles.statValue}>Lv.{agent.level}</Text>
              <Text style={styles.statLabel}>Level</Text>
            </View>
            <View style={styles.statBox}>
              <Text style={styles.statValue}>{agent.tasksCompleted || 0}</Text>
              <Text style={styles.statLabel}>Tasks</Text>
            </View>
            <View style={styles.statBox}>
              <Text style={styles.statValue}>{agent.successRate || 0}%</Text>
              <Text style={styles.statLabel}>Success</Text>
            </View>
          </View>
        )}

        <View style={styles.capSection}>
          <Text style={styles.capTitle}>Capabilities</Text>
          {['Autonomous task execution', 'Natural language interaction', 'Cross-agent collaboration', 'Real-time progress reporting'].map((cap, i) => (
            <View key={i} style={styles.capRow}>
              <Icon name="check-circle" size={16} color={colors.green} />
              <Text style={styles.capText}>{cap}</Text>
            </View>
          ))}
        </View>

        <TouchableOpacity style={[styles.actionBtn, {backgroundColor: domainColor}]} activeOpacity={0.8}>
          <Text style={styles.actionBtnText}>
            {agent.locked ? 'Unlock Agent' : agent.price ? 'Install Agent' : 'View Activity'}
          </Text>
        </TouchableOpacity>

        <View style={styles.bottomPad} />
      </ScrollView>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {flex: 1, backgroundColor: colors.background},
  header: {
    flexDirection: 'row', alignItems: 'center', justifyContent: 'space-between',
    paddingHorizontal: spacing.md, paddingTop: spacing.lg, paddingBottom: spacing.sm,
  },
  backBtn: {width: 40, height: 40, borderRadius: 20, backgroundColor: colors.surface, alignItems: 'center', justifyContent: 'center'},
  title: {color: colors.text, fontSize: fontSize.xl, fontWeight: '800'},
  placeholder: {width: 40},
  errorText: {color: colors.textMuted, fontSize: fontSize.lg, textAlign: 'center', marginTop: 100},
  scrollContent: {padding: spacing.md},
  heroSection: {alignItems: 'center', paddingVertical: spacing.xl, gap: spacing.sm},
  iconLarge: {width: 96, height: 96, borderRadius: 48, alignItems: 'center', justifyContent: 'center'},
  name: {color: colors.text, fontSize: fontSize.xxl, fontWeight: '800'},
  domain: {fontSize: fontSize.md, fontWeight: '600'},
  ratingRow: {flexDirection: 'row', alignItems: 'center', gap: 4, marginTop: spacing.xs},
  ratingText: {color: colors.amber, fontSize: fontSize.sm, fontWeight: '700', marginLeft: 4},
  installsText: {color: colors.textMuted, fontSize: fontSize.sm, marginLeft: spacing.sm},
  descCard: {backgroundColor: colors.surface, borderRadius: radius.lg, padding: spacing.lg, gap: spacing.sm, borderWidth: 1, borderColor: colors.surfaceBorder},
  descTitle: {color: colors.text, fontSize: fontSize.lg, fontWeight: '700'},
  descText: {color: colors.textSecondary, fontSize: fontSize.md, lineHeight: 24},
  statsRow: {flexDirection: 'row', gap: spacing.sm, marginTop: spacing.md},
  statBox: {flex: 1, backgroundColor: colors.surface, borderRadius: radius.md, padding: spacing.md, alignItems: 'center', gap: 4, borderWidth: 1, borderColor: colors.surfaceBorder},
  statValue: {color: colors.text, fontSize: fontSize.xl, fontWeight: '800'},
  statLabel: {color: colors.textMuted, fontSize: fontSize.xs},
  capSection: {marginTop: spacing.md, backgroundColor: colors.surface, borderRadius: radius.lg, padding: spacing.lg, gap: spacing.sm, borderWidth: 1, borderColor: colors.surfaceBorder},
  capTitle: {color: colors.text, fontSize: fontSize.lg, fontWeight: '700'},
  capRow: {flexDirection: 'row', alignItems: 'center', gap: spacing.sm},
  capText: {color: colors.textSecondary, fontSize: fontSize.md},
  actionBtn: {marginTop: spacing.lg, paddingVertical: spacing.md, borderRadius: radius.xl, alignItems: 'center'},
  actionBtnText: {color: colors.white, fontSize: fontSize.lg, fontWeight: '700'},
  bottomPad: {height: 40},
});
