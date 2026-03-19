import React, {useState} from 'react';
import {View, Text, ScrollView, StyleSheet, TouchableOpacity, Modal} from 'react-native';
import Icon from 'react-native-vector-icons/MaterialIcons';
import {colors, spacing, radius, fontSize, screen} from '../theme';
import {domainColors} from '../theme';
import {useApp} from '../context/AppContext';
import AgentCard from '../components/AgentCard';
import type {Agent} from '../types';
import {useNavigation} from '@react-navigation/native';

export default function AgentsScreen() {
  const {state} = useApp();
  const navigation = useNavigation<any>();
  const [tab, setTab] = useState<'collected' | 'marketplace'>('collected');
  const [selectedAgent, setSelectedAgent] = useState<Agent | null>(null);

  const collected = state.agents.filter(a => !a.locked);
  const locked = state.agents.filter(a => a.locked);
  const cols = screen.isTablet ? 4 : 3;

  return (
    <View style={styles.container}>
      <View style={styles.header}>
        <Text style={styles.title}>Agents</Text>
        <Text style={styles.count}>{collected.length} collected</Text>
      </View>

      <View style={styles.tabs}>
        <TouchableOpacity
          style={[styles.tab, tab === 'collected' && styles.tabActive]}
          onPress={() => setTab('collected')}>
          <Text style={[styles.tabText, tab === 'collected' && styles.tabTextActive]}>
            My Agents
          </Text>
        </TouchableOpacity>
        <TouchableOpacity
          style={[styles.tab, tab === 'marketplace' && styles.tabActive]}
          onPress={() => setTab('marketplace')}>
          <Text style={[styles.tabText, tab === 'marketplace' && styles.tabTextActive]}>
            Marketplace
          </Text>
        </TouchableOpacity>
      </View>

      <ScrollView contentContainerStyle={styles.scrollContent} showsVerticalScrollIndicator={false}>
        {tab === 'collected' ? (
          <>
            <View style={styles.grid}>
              {collected.map(agent => (
                <View key={agent.id} style={{width: (screen.width - spacing.md * 2 - spacing.sm * (cols - 1)) / cols}}>
                  <AgentCard agent={agent} compact onPress={() => setSelectedAgent(agent)} />
                </View>
              ))}
            </View>
            {locked.length > 0 && (
              <>
                <Text style={styles.sectionLabel}>Locked</Text>
                <View style={styles.grid}>
                  {locked.map(agent => (
                    <View key={agent.id} style={{width: (screen.width - spacing.md * 2 - spacing.sm * (cols - 1)) / cols}}>
                      <AgentCard agent={agent} compact onPress={() => setSelectedAgent(agent)} />
                    </View>
                  ))}
                </View>
              </>
            )}
          </>
        ) : (
          <View style={styles.marketplaceContent}>
            <TouchableOpacity
              style={styles.marketCard}
              onPress={() => navigation.navigate('Marketplace')}
              activeOpacity={0.7}>
              <Icon name="storefront" size={48} color={colors.violet} />
              <Text style={styles.marketTitle}>Agent Marketplace</Text>
              <Text style={styles.marketSubtitle}>
                Browse and install specialized agents to expand your AI team.
                New agents are released weekly.
              </Text>
              <View style={styles.marketCategories}>
                {['Finance', 'Health', 'Legal', 'Career', 'Education', 'Shopping'].map(cat => (
                  <View key={cat} style={styles.categoryChip}>
                    <Text style={styles.categoryText}>{cat}</Text>
                  </View>
                ))}
              </View>
              <View style={styles.browseButton}>
                <Text style={styles.browseButtonText}>Browse Marketplace</Text>
                <Icon name="arrow-forward" size={18} color={colors.background} />
              </View>
            </TouchableOpacity>

            <TouchableOpacity
              style={styles.collectionCard}
              onPress={() => navigation.navigate('AgentCollection')}
              activeOpacity={0.7}>
              <Icon name="grid-view" size={36} color={colors.cyan} />
              <View style={styles.collectionInfo}>
                <Text style={styles.collectionTitle}>My Collection</Text>
                <Text style={styles.collectionSubtitle}>7/52 agents collected</Text>
              </View>
              <Icon name="chevron-right" size={24} color={colors.textMuted} />
            </TouchableOpacity>
          </View>
        )}
        <View style={styles.bottomPad} />
      </ScrollView>

      {/* Agent Detail Modal */}
      <Modal visible={selectedAgent !== null} animationType="slide" transparent>
        <View style={styles.modalOverlay}>
          <View style={styles.modalContent}>
            {selectedAgent && (
              <>
                <View style={styles.modalHeader}>
                  <View style={styles.modalClose}>
                    <TouchableOpacity onPress={() => setSelectedAgent(null)}>
                      <Icon name="close" size={24} color={colors.text} />
                    </TouchableOpacity>
                  </View>
                  <View style={[styles.modalIcon, {backgroundColor: (selectedAgent.domain === 'General' ? colors.violet : domainColors[selectedAgent.domain as keyof typeof domainColors] || colors.cyan) + '20'}]}>
                    <Icon
                      name={selectedAgent.locked ? 'lock' : selectedAgent.icon}
                      size={36}
                      color={selectedAgent.domain === 'General' ? colors.violet : domainColors[selectedAgent.domain as keyof typeof domainColors] || colors.cyan}
                    />
                  </View>
                  <Text style={styles.modalName}>{selectedAgent.name}</Text>
                  <Text style={styles.modalDomain}>{selectedAgent.domain}</Text>
                </View>

                {selectedAgent.locked ? (
                  <View style={styles.modalBody}>
                    <Text style={styles.modalDesc}>{selectedAgent.description}</Text>
                    <View style={styles.unlockInfo}>
                      <Icon name="lock-open" size={20} color={colors.amber} />
                      <Text style={styles.unlockText}>Complete 5 more tasks to unlock</Text>
                    </View>
                  </View>
                ) : (
                  <View style={styles.modalBody}>
                    <Text style={styles.modalDesc}>{selectedAgent.description}</Text>
                    <View style={styles.statsGrid}>
                      <View style={styles.statItem}>
                        <Text style={styles.statValue}>Lv.{selectedAgent.level}</Text>
                        <Text style={styles.statLabel}>Level</Text>
                      </View>
                      <View style={styles.statItem}>
                        <Text style={styles.statValue}>{selectedAgent.tasksCompleted}</Text>
                        <Text style={styles.statLabel}>Tasks Done</Text>
                      </View>
                      <View style={styles.statItem}>
                        <Text style={styles.statValue}>{selectedAgent.successRate}%</Text>
                        <Text style={styles.statLabel}>Success</Text>
                      </View>
                    </View>
                    <View style={styles.xpSection}>
                      <View style={styles.xpHeader}>
                        <Text style={styles.xpLabel}>XP Progress</Text>
                        <Text style={styles.xpCount}>{selectedAgent.xp}/{selectedAgent.xpToNext}</Text>
                      </View>
                      <View style={styles.xpBar}>
                        <View style={[styles.xpFill, {width: `${(selectedAgent.xp / selectedAgent.xpToNext) * 100}%`}]} />
                      </View>
                    </View>
                  </View>
                )}
              </>
            )}
          </View>
        </View>
      </Modal>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: colors.background,
  },
  header: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'baseline',
    paddingHorizontal: spacing.md,
    paddingTop: spacing.lg,
    paddingBottom: spacing.sm,
  },
  title: {
    color: colors.text,
    fontSize: fontSize.xxl,
    fontWeight: '800',
  },
  count: {
    color: colors.textSecondary,
    fontSize: fontSize.md,
  },
  tabs: {
    flexDirection: 'row',
    paddingHorizontal: spacing.md,
    gap: spacing.sm,
    marginBottom: spacing.md,
  },
  tab: {
    paddingHorizontal: spacing.md,
    paddingVertical: spacing.sm,
    borderRadius: radius.full,
    backgroundColor: colors.surface,
  },
  tabActive: {
    backgroundColor: colors.cyan,
  },
  tabText: {
    color: colors.textSecondary,
    fontSize: fontSize.md,
    fontWeight: '600',
  },
  tabTextActive: {
    color: colors.white,
  },
  scrollContent: {
    paddingHorizontal: spacing.md,
  },
  grid: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    gap: spacing.sm,
  },
  sectionLabel: {
    color: colors.textMuted,
    fontSize: fontSize.sm,
    fontWeight: '600',
    marginTop: spacing.lg,
    marginBottom: spacing.sm,
    textTransform: 'uppercase',
    letterSpacing: 1,
  },
  marketplaceContent: {
    alignItems: 'center',
    paddingTop: spacing.xl,
  },
  marketCard: {
    backgroundColor: colors.surface,
    borderRadius: radius.lg,
    padding: spacing.xl,
    alignItems: 'center',
    gap: spacing.md,
    borderWidth: 1,
    borderColor: colors.violetDim,
    width: '100%',
  },
  marketTitle: {
    color: colors.text,
    fontSize: fontSize.xl,
    fontWeight: '700',
  },
  marketSubtitle: {
    color: colors.textSecondary,
    fontSize: fontSize.md,
    textAlign: 'center',
    lineHeight: 22,
  },
  marketCategories: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    gap: spacing.xs,
    justifyContent: 'center',
  },
  categoryChip: {
    paddingHorizontal: spacing.sm,
    paddingVertical: spacing.xs,
    borderRadius: radius.full,
    backgroundColor: colors.surfaceLight,
  },
  categoryText: {
    color: colors.textSecondary,
    fontSize: fontSize.sm,
  },
  browseButton: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: spacing.xs,
    backgroundColor: colors.cyan,
    paddingHorizontal: spacing.lg,
    paddingVertical: spacing.sm + 2,
    borderRadius: radius.full,
    marginTop: spacing.sm,
  },
  browseButtonText: {
    color: colors.background,
    fontSize: fontSize.md,
    fontWeight: '700',
  },
  collectionCard: {
    flexDirection: 'row',
    alignItems: 'center',
    backgroundColor: colors.surface,
    borderRadius: radius.lg,
    padding: spacing.lg,
    gap: spacing.md,
    borderWidth: 1,
    borderColor: colors.surfaceBorder,
    width: '100%',
    marginTop: spacing.md,
  },
  collectionInfo: {
    flex: 1,
    gap: 2,
  },
  collectionTitle: {
    color: colors.text,
    fontSize: fontSize.lg,
    fontWeight: '700',
  },
  collectionSubtitle: {
    color: colors.textSecondary,
    fontSize: fontSize.sm,
  },
  bottomPad: {
    height: 100,
  },
  // Modal
  modalOverlay: {
    flex: 1,
    backgroundColor: 'rgba(0,0,0,0.7)',
    justifyContent: 'flex-end',
  },
  modalContent: {
    backgroundColor: colors.background,
    borderTopLeftRadius: radius.xl,
    borderTopRightRadius: radius.xl,
    paddingBottom: spacing.xl,
    maxHeight: '80%',
  },
  modalHeader: {
    alignItems: 'center',
    padding: spacing.lg,
    gap: spacing.sm,
    borderBottomWidth: 1,
    borderBottomColor: colors.surfaceBorder,
  },
  modalClose: {
    position: 'absolute',
    top: spacing.md,
    right: spacing.md,
    zIndex: 1,
  },
  modalIcon: {
    width: 72,
    height: 72,
    borderRadius: 36,
    alignItems: 'center',
    justifyContent: 'center',
  },
  modalName: {
    color: colors.text,
    fontSize: fontSize.xxl,
    fontWeight: '800',
  },
  modalDomain: {
    color: colors.textSecondary,
    fontSize: fontSize.md,
  },
  modalBody: {
    padding: spacing.lg,
    gap: spacing.lg,
  },
  modalDesc: {
    color: colors.textSecondary,
    fontSize: fontSize.md,
    lineHeight: 24,
  },
  statsGrid: {
    flexDirection: 'row',
    justifyContent: 'space-around',
  },
  statItem: {
    alignItems: 'center',
    gap: 4,
  },
  statValue: {
    color: colors.text,
    fontSize: fontSize.xl,
    fontWeight: '800',
  },
  statLabel: {
    color: colors.textMuted,
    fontSize: fontSize.xs,
  },
  xpSection: {
    gap: spacing.xs,
  },
  xpHeader: {
    flexDirection: 'row',
    justifyContent: 'space-between',
  },
  xpLabel: {
    color: colors.textSecondary,
    fontSize: fontSize.sm,
  },
  xpCount: {
    color: colors.textMuted,
    fontSize: fontSize.sm,
  },
  xpBar: {
    height: 6,
    backgroundColor: colors.surfaceLight,
    borderRadius: 3,
  },
  xpFill: {
    height: 6,
    borderRadius: 3,
    backgroundColor: colors.cyan,
  },
  unlockInfo: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: spacing.sm,
    backgroundColor: colors.amberDim,
    padding: spacing.md,
    borderRadius: radius.md,
  },
  unlockText: {
    color: colors.amber,
    fontSize: fontSize.md,
    fontWeight: '600',
  },
});
