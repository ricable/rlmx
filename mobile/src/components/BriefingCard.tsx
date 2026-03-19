import React from 'react';
import {View, Text, StyleSheet, TouchableOpacity} from 'react-native';
import Icon from 'react-native-vector-icons/MaterialIcons';
import {colors, spacing, radius, fontSize} from '../theme';
import {domainColors} from '../theme';
import type {BriefingItem} from '../types';

interface Props {
  item: BriefingItem;
  onAction?: () => void;
}

export default function BriefingCard({item, onAction}: Props) {
  const domainColor = domainColors[item.domain];

  return (
    <View style={[styles.card, {borderLeftColor: domainColor}]}>
      <View style={styles.row}>
        <View style={[styles.iconCircle, {backgroundColor: domainColor + '20'}]}>
          <Icon name={item.icon} size={20} color={domainColor} />
        </View>
        <View style={styles.content}>
          <Text style={styles.title}>{item.title}</Text>
          <Text style={styles.subtitle} numberOfLines={2}>{item.subtitle}</Text>
        </View>
      </View>
      {item.actionLabel && (
        <TouchableOpacity style={[styles.action, {borderColor: domainColor + '40'}]} onPress={onAction}>
          <Text style={[styles.actionText, {color: domainColor}]}>{item.actionLabel}</Text>
          <Icon name="chevron-right" size={16} color={domainColor} />
        </TouchableOpacity>
      )}
    </View>
  );
}

const styles = StyleSheet.create({
  card: {
    backgroundColor: colors.surface,
    borderRadius: radius.md,
    padding: spacing.md,
    borderLeftWidth: 3,
    gap: spacing.sm,
  },
  row: {
    flexDirection: 'row',
    gap: spacing.sm,
  },
  iconCircle: {
    width: 40,
    height: 40,
    borderRadius: 20,
    alignItems: 'center',
    justifyContent: 'center',
  },
  content: {
    flex: 1,
    gap: 2,
  },
  title: {
    color: colors.text,
    fontSize: fontSize.md,
    fontWeight: '700',
  },
  subtitle: {
    color: colors.textSecondary,
    fontSize: fontSize.sm,
  },
  action: {
    flexDirection: 'row',
    alignItems: 'center',
    alignSelf: 'flex-end',
    paddingHorizontal: spacing.sm,
    paddingVertical: spacing.xs,
    borderRadius: radius.sm,
    borderWidth: 1,
    gap: 2,
  },
  actionText: {
    fontSize: fontSize.sm,
    fontWeight: '600',
  },
});
