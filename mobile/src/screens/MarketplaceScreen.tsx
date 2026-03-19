import React, {useState} from 'react';
import {View, Text, ScrollView, StyleSheet, TouchableOpacity, TextInput} from 'react-native';
import Icon from 'react-native-vector-icons/MaterialIcons';
import {colors, spacing, radius, fontSize} from '../theme';
import {domainColors} from '../theme';
import {useNavigation} from '@react-navigation/native';

interface MarketAgent {
  id: string;
  name: string;
  domain: string;
  icon: string;
  description: string;
  rating: number;
  installs: string;
  price: string;
}

const marketplaceAgents: MarketAgent[] = [
  {id: 'm1', name: 'TaxBot', domain: 'Finance', icon: 'receipt', description: 'Automatic tax deduction tracking and filing prep', rating: 4.8, installs: '12K', price: 'Free'},
  {id: 'm2', name: 'FitCoach', domain: 'Health', icon: 'fitness-center', description: 'Personalized workout plans based on your goals', rating: 4.6, installs: '8K', price: 'Plus'},
  {id: 'm3', name: 'LegalEye', domain: 'Legal', icon: 'gavel', description: 'Contract review and legal document analysis', rating: 4.9, installs: '5K', price: 'Pro'},
  {id: 'm4', name: 'CareerPilot', domain: 'Career', icon: 'work', description: 'Resume optimization and job matching', rating: 4.7, installs: '15K', price: 'Free'},
  {id: 'm5', name: 'StudyBuddy', domain: 'Education', icon: 'school', description: 'Flashcards, quizzes, and learning optimization', rating: 4.5, installs: '20K', price: 'Free'},
  {id: 'm6', name: 'HomeGuard', domain: 'Home', icon: 'home', description: 'Smart home automation and energy savings', rating: 4.4, installs: '6K', price: 'Plus'},
  {id: 'm7', name: 'DealFinder', domain: 'Shopping', icon: 'local-offer', description: 'Real-time price comparison and deal alerts', rating: 4.7, installs: '25K', price: 'Free'},
  {id: 'm8', name: 'TripWise', domain: 'Travel', icon: 'flight', description: 'Travel planning, booking optimization, itineraries', rating: 4.6, installs: '9K', price: 'Plus'},
  {id: 'm9', name: 'AutoCare', domain: 'Automotive', icon: 'directions-car', description: 'Vehicle maintenance tracking and cost optimization', rating: 4.3, installs: '4K', price: 'Free'},
  {id: 'm10', name: 'PetPal', domain: 'Pet', icon: 'pets', description: 'Pet health tracking, vet reminders, nutrition', rating: 4.8, installs: '7K', price: 'Free'},
  {id: 'm11', name: 'GovHelper', domain: 'Government', icon: 'account-balance', description: 'Benefits finder, form assistance, deadline tracking', rating: 4.5, installs: '11K', price: 'Free'},
  {id: 'm12', name: 'SocialSync', domain: 'Social', icon: 'people', description: 'Relationship management and social calendar', rating: 4.2, installs: '3K', price: 'Plus'},
];

const categories = ['All', 'Finance', 'Health', 'Legal', 'Career', 'Education', 'Shopping', 'Home', 'Travel'];

export default function MarketplaceScreen() {
  const navigation = useNavigation<any>();
  const [selectedCategory, setSelectedCategory] = useState('All');
  const [searchQuery, setSearchQuery] = useState('');

  const filtered = marketplaceAgents.filter(a => {
    const matchesCat = selectedCategory === 'All' || a.domain === selectedCategory;
    const matchesSearch = !searchQuery || a.name.toLowerCase().includes(searchQuery.toLowerCase()) || a.description.toLowerCase().includes(searchQuery.toLowerCase());
    return matchesCat && matchesSearch;
  });

  return (
    <View style={styles.container}>
      <View style={styles.header}>
        <TouchableOpacity onPress={() => navigation.goBack()} style={styles.backBtn}>
          <Icon name="arrow-back" size={24} color={colors.text} />
        </TouchableOpacity>
        <Text style={styles.title}>Marketplace</Text>
        <View style={styles.placeholder} />
      </View>

      <View style={styles.searchRow}>
        <View style={styles.searchBox}>
          <Icon name="search" size={20} color={colors.textMuted} />
          <TextInput
            style={styles.searchInput}
            placeholder="Search agents..."
            placeholderTextColor={colors.textMuted}
            value={searchQuery}
            onChangeText={setSearchQuery}
          />
        </View>
      </View>

      <ScrollView horizontal showsHorizontalScrollIndicator={false} style={styles.categoryScroll} contentContainerStyle={styles.categoryContent}>
        {categories.map(cat => (
          <TouchableOpacity
            key={cat}
            style={[styles.categoryChip, selectedCategory === cat && styles.categoryChipActive]}
            onPress={() => setSelectedCategory(cat)}>
            <Text style={[styles.categoryText, selectedCategory === cat && styles.categoryTextActive]}>{cat}</Text>
          </TouchableOpacity>
        ))}
      </ScrollView>

      <ScrollView contentContainerStyle={styles.scrollContent} showsVerticalScrollIndicator={false}>
        {filtered.map(agent => {
          const domainColor = (domainColors as Record<string, string>)[agent.domain] || colors.cyan;
          return (
            <TouchableOpacity
              key={agent.id}
              style={styles.agentRow}
              activeOpacity={0.7}
              onPress={() => navigation.navigate('AgentDetail', {agent})}>
              <View style={[styles.agentIcon, {backgroundColor: domainColor + '20'}]}>
                <Icon name={agent.icon} size={24} color={domainColor} />
              </View>
              <View style={styles.agentInfo}>
                <View style={styles.agentNameRow}>
                  <Text style={styles.agentName}>{agent.name}</Text>
                  <View style={[styles.priceBadge, agent.price === 'Free' ? styles.freeBadge : agent.price === 'Plus' ? styles.plusBadge : styles.proBadge]}>
                    <Text style={[styles.priceText, agent.price === 'Free' ? styles.freeText : agent.price === 'Plus' ? styles.plusText : styles.proText]}>
                      {agent.price}
                    </Text>
                  </View>
                </View>
                <Text style={styles.agentDesc} numberOfLines={1}>{agent.description}</Text>
                <View style={styles.agentMeta}>
                  <View style={styles.ratingRow}>
                    <Icon name="star" size={12} color={colors.amber} />
                    <Text style={styles.rating}>{agent.rating}</Text>
                  </View>
                  <Text style={styles.installs}>{agent.installs} installs</Text>
                </View>
              </View>
              <Icon name="chevron-right" size={20} color={colors.textMuted} />
            </TouchableOpacity>
          );
        })}
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
  searchRow: {paddingHorizontal: spacing.md, marginBottom: spacing.sm},
  searchBox: {
    flexDirection: 'row', alignItems: 'center', gap: spacing.sm,
    backgroundColor: colors.surface, borderRadius: radius.full,
    paddingHorizontal: spacing.md, borderWidth: 1, borderColor: colors.surfaceBorder,
  },
  searchInput: {flex: 1, color: colors.text, fontSize: fontSize.md, paddingVertical: spacing.sm},
  categoryScroll: {maxHeight: 44, marginBottom: spacing.sm},
  categoryContent: {paddingHorizontal: spacing.md, gap: spacing.xs},
  categoryChip: {
    paddingHorizontal: spacing.md, paddingVertical: spacing.xs,
    borderRadius: radius.full, backgroundColor: colors.surface,
    borderWidth: 1, borderColor: colors.surfaceBorder,
  },
  categoryChipActive: {backgroundColor: colors.cyan, borderColor: colors.cyan},
  categoryText: {color: colors.textSecondary, fontSize: fontSize.sm, fontWeight: '600'},
  categoryTextActive: {color: colors.white},
  scrollContent: {paddingHorizontal: spacing.md},
  agentRow: {
    flexDirection: 'row', alignItems: 'center', gap: spacing.sm,
    backgroundColor: colors.surface, borderRadius: radius.md, padding: spacing.md,
    marginBottom: spacing.sm, borderWidth: 1, borderColor: colors.surfaceBorder,
  },
  agentIcon: {width: 48, height: 48, borderRadius: 24, alignItems: 'center', justifyContent: 'center'},
  agentInfo: {flex: 1, gap: 2},
  agentNameRow: {flexDirection: 'row', alignItems: 'center', gap: spacing.xs},
  agentName: {color: colors.text, fontSize: fontSize.md, fontWeight: '700'},
  priceBadge: {paddingHorizontal: 6, paddingVertical: 1, borderRadius: 4},
  freeBadge: {backgroundColor: colors.greenDim},
  plusBadge: {backgroundColor: colors.cyanDim},
  proBadge: {backgroundColor: colors.violetDim},
  priceText: {fontSize: 10, fontWeight: '700'},
  freeText: {color: colors.green},
  plusText: {color: colors.cyan},
  proText: {color: colors.violet},
  agentDesc: {color: colors.textSecondary, fontSize: fontSize.sm},
  agentMeta: {flexDirection: 'row', alignItems: 'center', gap: spacing.sm, marginTop: 2},
  ratingRow: {flexDirection: 'row', alignItems: 'center', gap: 2},
  rating: {color: colors.amber, fontSize: fontSize.xs, fontWeight: '600'},
  installs: {color: colors.textMuted, fontSize: fontSize.xs},
  bottomPad: {height: 40},
});
