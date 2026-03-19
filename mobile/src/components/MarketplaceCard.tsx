import React from 'react';
import {StyleSheet, Text, TouchableOpacity, View} from 'react-native';
import Icon from 'react-native-vector-icons/MaterialIcons';
import {colors, spacing, radius, fontSize} from '../theme';
import {
  type MarketplaceAgent,
  type PriceTier,
  DOMAIN_COLORS,
  formatInstalls,
} from '../data/agents';

interface Props {
  agent: MarketplaceAgent;
  onPress?: () => void;
  onInstall?: () => void;
  installed?: boolean;
}

const PRICE_COLORS: Record<PriceTier, string> = {
  Free: '#22c55e',
  Plus: '#a855f7',
  Pro: '#f59e0b',
};

function StarRating({rating, size = 12}: {rating: number; size?: number}) {
  const stars = [];
  for (let i = 1; i <= 5; i++) {
    if (i <= Math.floor(rating)) {
      stars.push(
        <Icon key={i} name="star" size={size} color="#f59e0b" />,
      );
    } else if (i - 0.5 <= rating) {
      stars.push(
        <Icon key={i} name="star-half" size={size} color="#f59e0b" />,
      );
    } else {
      stars.push(
        <Icon key={i} name="star-border" size={size} color="#64748b" />,
      );
    }
  }
  return <View style={starStyles.container}>{stars}</View>;
}

const starStyles = StyleSheet.create({
  container: {
    flexDirection: 'row',
    gap: 1,
  },
});

export {StarRating};

export default function MarketplaceCard({
  agent,
  onPress,
  onInstall,
  installed,
}: Props) {
  const domainColor = DOMAIN_COLORS[agent.domain];
  const priceColor = PRICE_COLORS[agent.price];

  return (
    <TouchableOpacity
      style={styles.card}
      onPress={onPress}
      activeOpacity={0.7}>
      <View style={styles.row}>
        <View
          style={[
            styles.iconCircle,
            {backgroundColor: `${agent.iconColor}20`},
          ]}>
          <Icon name={agent.icon} size={28} color={agent.iconColor} />
        </View>

        <View style={styles.info}>
          <View style={styles.nameRow}>
            <Text style={styles.name} numberOfLines={1}>
              {agent.name}
            </Text>
            <View
              style={[styles.priceBadge, {backgroundColor: `${priceColor}20`}]}>
              <Text style={[styles.priceText, {color: priceColor}]}>
                {agent.price}
              </Text>
            </View>
          </View>

          <Text style={styles.publisher}>{agent.publisher}</Text>

          <Text style={styles.description} numberOfLines={2}>
            {agent.description}
          </Text>

          <View style={styles.statsRow}>
            <View style={styles.ratingRow}>
              <StarRating rating={agent.rating} />
              <Text style={styles.ratingText}>{agent.rating}</Text>
            </View>
            <View style={styles.installRow}>
              <Icon
                name="download"
                size={12}
                color={colors.textMuted}
              />
              <Text style={styles.installText}>
                {formatInstalls(agent.installs)}
              </Text>
            </View>
            <View
              style={[
                styles.domainChip,
                {backgroundColor: `${domainColor}15`},
              ]}>
              <Text style={[styles.domainText, {color: domainColor}]}>
                {agent.domain}
              </Text>
            </View>
          </View>
        </View>

        <TouchableOpacity
          style={[
            styles.installButton,
            installed && styles.installedButton,
          ]}
          onPress={e => {
            e.stopPropagation();
            onInstall?.();
          }}
          activeOpacity={0.7}>
          <Text
            style={[
              styles.installButtonText,
              installed && styles.installedButtonText,
            ]}>
            {installed ? 'Open' : 'Get'}
          </Text>
        </TouchableOpacity>
      </View>
    </TouchableOpacity>
  );
}

const styles = StyleSheet.create({
  card: {
    backgroundColor: colors.surface,
    borderRadius: radius.md,
    padding: spacing.md,
    borderWidth: 1,
    borderColor: colors.surfaceBorder,
  },
  row: {
    flexDirection: 'row',
    gap: spacing.sm,
    alignItems: 'flex-start',
  },
  iconCircle: {
    width: 52,
    height: 52,
    borderRadius: radius.md,
    alignItems: 'center',
    justifyContent: 'center',
  },
  info: {
    flex: 1,
    gap: 3,
  },
  nameRow: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: spacing.xs,
  },
  name: {
    color: colors.text,
    fontSize: fontSize.lg,
    fontWeight: '700',
    flex: 1,
  },
  publisher: {
    color: colors.textMuted,
    fontSize: fontSize.xs,
  },
  description: {
    color: colors.textSecondary,
    fontSize: fontSize.sm,
    lineHeight: 18,
    marginTop: 2,
  },
  statsRow: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: spacing.sm,
    marginTop: 4,
  },
  ratingRow: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 3,
  },
  ratingText: {
    color: colors.textSecondary,
    fontSize: fontSize.xs,
    fontWeight: '600',
  },
  installRow: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 2,
  },
  installText: {
    color: colors.textMuted,
    fontSize: fontSize.xs,
  },
  domainChip: {
    paddingHorizontal: 6,
    paddingVertical: 2,
    borderRadius: radius.sm,
  },
  domainText: {
    fontSize: 10,
    fontWeight: '600',
  },
  priceBadge: {
    paddingHorizontal: 6,
    paddingVertical: 2,
    borderRadius: radius.sm,
  },
  priceText: {
    fontSize: 10,
    fontWeight: '700',
  },
  installButton: {
    backgroundColor: colors.cyan,
    paddingHorizontal: spacing.md,
    paddingVertical: spacing.xs + 2,
    borderRadius: radius.full,
    alignSelf: 'center',
    minWidth: 56,
    alignItems: 'center',
  },
  installedButton: {
    backgroundColor: 'transparent',
    borderWidth: 1,
    borderColor: colors.cyan,
  },
  installButtonText: {
    color: colors.background,
    fontSize: fontSize.sm,
    fontWeight: '700',
  },
  installedButtonText: {
    color: colors.cyan,
  },
});
