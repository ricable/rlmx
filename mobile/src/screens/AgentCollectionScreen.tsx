import React from 'react';
import {View, Text, ScrollView, StyleSheet, TouchableOpacity} from 'react-native';
import Icon from 'react-native-vector-icons/MaterialIcons';
import {colors, spacing, fontSize, screen} from '../theme';
import {useApp} from '../context/AppContext';
import AgentCard from '../components/AgentCard';
import {useNavigation} from '@react-navigation/native';

export default function AgentCollectionScreen() {
  const navigation = useNavigation<any>();
  const {state} = useApp();
  const cols = screen.isTablet ? 4 : 3;

  const collected = state.agents.filter(a => !a.locked);
  const locked = state.agents.filter(a => a.locked);

  return (
    <View style={styles.container}>
      <View style={styles.header}>
        <TouchableOpacity onPress={() => navigation.goBack()} style={styles.backBtn}>
          <Icon name="arrow-back" size={24} color={colors.text} />
        </TouchableOpacity>
        <Text style={styles.title}>My Collection</Text>
        <Text style={styles.count}>{collected.length}/{state.agents.length}</Text>
      </View>

      <ScrollView contentContainerStyle={styles.scrollContent} showsVerticalScrollIndicator={false}>
        <Text style={styles.sectionLabel}>Active Agents</Text>
        <View style={styles.grid}>
          {collected.map(agent => (
            <View key={agent.id} style={{width: (screen.width - spacing.md * 2 - spacing.sm * (cols - 1)) / cols}}>
              <AgentCard
                agent={agent}
                onPress={() => navigation.navigate('AgentDetail', {agent})}
              />
            </View>
          ))}
        </View>

        {locked.length > 0 && (
          <>
            <Text style={styles.sectionLabel}>Locked</Text>
            <View style={styles.grid}>
              {locked.map(agent => (
                <View key={agent.id} style={{width: (screen.width - spacing.md * 2 - spacing.sm * (cols - 1)) / cols}}>
                  <AgentCard
                    agent={agent}
                    onPress={() => navigation.navigate('AgentDetail', {agent})}
                  />
                </View>
              ))}
            </View>
          </>
        )}
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
  count: {color: colors.textSecondary, fontSize: fontSize.md},
  scrollContent: {padding: spacing.md},
  sectionLabel: {
    color: colors.textMuted, fontSize: fontSize.sm, fontWeight: '600',
    textTransform: 'uppercase', letterSpacing: 1, marginBottom: spacing.sm, marginTop: spacing.md,
  },
  grid: {flexDirection: 'row', flexWrap: 'wrap', gap: spacing.sm},
  bottomPad: {height: 40},
});
