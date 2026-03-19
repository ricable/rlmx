import React, {useEffect, useRef, useState} from 'react';
import {View, Text, StyleSheet, Animated} from 'react-native';
import {colors, spacing, radius, fontSize} from '../theme';

interface Props {
  amount: number;
}

export default function MoneySavedCounter({amount}: Props) {
  const [displayed, setDisplayed] = useState(0);
  const animatedValue = useRef(new Animated.Value(0)).current;

  useEffect(() => {
    Animated.timing(animatedValue, {
      toValue: amount,
      duration: 1500,
      useNativeDriver: false,
    }).start();

    const listener = animatedValue.addListener(({value}) => {
      setDisplayed(Math.round(value));
    });

    return () => animatedValue.removeListener(listener);
  }, [amount, animatedValue]);

  return (
    <View style={styles.card}>
      <Text style={styles.label}>Total Saved</Text>
      <View style={styles.amountRow}>
        <Text style={styles.currency}>$</Text>
        <Text style={styles.amount}>{displayed.toLocaleString()}</Text>
      </View>
      <Text style={styles.subtitle}>Since Dec 2025</Text>
      <View style={styles.breakdown}>
        <View style={styles.breakdownItem}>
          <Text style={styles.breakdownValue}>$340</Text>
          <Text style={styles.breakdownLabel}>This month</Text>
        </View>
        <View style={styles.divider} />
        <View style={styles.breakdownItem}>
          <Text style={styles.breakdownValue}>$47</Text>
          <Text style={styles.breakdownLabel}>Subscriptions</Text>
        </View>
        <View style={styles.divider} />
        <View style={styles.breakdownItem}>
          <Text style={styles.breakdownValue}>$23</Text>
          <Text style={styles.breakdownLabel}>Bills</Text>
        </View>
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
    borderColor: colors.greenDim,
  },
  label: {
    color: colors.textSecondary,
    fontSize: fontSize.md,
    fontWeight: '600',
  },
  amountRow: {
    flexDirection: 'row',
    alignItems: 'baseline',
    marginTop: spacing.xs,
  },
  currency: {
    color: colors.green,
    fontSize: fontSize.xl,
    fontWeight: '700',
  },
  amount: {
    color: colors.green,
    fontSize: fontSize.hero,
    fontWeight: '800',
  },
  subtitle: {
    color: colors.textMuted,
    fontSize: fontSize.sm,
    marginTop: spacing.xs,
  },
  breakdown: {
    flexDirection: 'row',
    marginTop: spacing.md,
    paddingTop: spacing.md,
    borderTopWidth: 1,
    borderTopColor: colors.surfaceBorder,
  },
  breakdownItem: {
    flex: 1,
    alignItems: 'center',
  },
  breakdownValue: {
    color: colors.text,
    fontSize: fontSize.lg,
    fontWeight: '700',
  },
  breakdownLabel: {
    color: colors.textMuted,
    fontSize: fontSize.xs,
    marginTop: 2,
  },
  divider: {
    width: 1,
    backgroundColor: colors.surfaceBorder,
  },
});
