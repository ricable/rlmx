/**
 * CounterAnimation — smooth animated number counter for Money Saved display.
 *
 * Uses requestAnimationFrame for 60fps updates with an ease-out cubic curve.
 * Supports currency formatting and configurable animation duration.
 */

import React, { useEffect, useRef, useState } from 'react';
import { StyleSheet, Text, TextStyle, View, ViewStyle } from 'react-native';

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

interface CounterAnimationProps {
  value: number;
  duration?: number;
  prefix?: string;
  suffix?: string;
  decimals?: number;
  style?: ViewStyle;
  textStyle?: TextStyle;
  labelStyle?: TextStyle;
  label?: string;
}

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

export const CounterAnimation: React.FC<CounterAnimationProps> = ({
  value,
  duration = 800,
  prefix = '$',
  suffix = '',
  decimals = 2,
  style,
  textStyle,
  labelStyle,
  label,
}) => {
  const [displayValue, setDisplayValue] = useState(0);
  const previousValue = useRef(0);
  const animationRef = useRef<number | null>(null);

  useEffect(() => {
    const startValue = previousValue.current;
    const diff = value - startValue;

    if (diff === 0) return;

    const startTime = Date.now();

    const animate = () => {
      const elapsed = Date.now() - startTime;
      const progress = Math.min(elapsed / duration, 1);

      // Ease-out cubic: 1 - (1 - t)^3
      const eased = 1 - Math.pow(1 - progress, 3);
      const current = startValue + diff * eased;

      setDisplayValue(current);

      if (progress < 1) {
        animationRef.current = requestAnimationFrame(animate);
      } else {
        setDisplayValue(value);
        previousValue.current = value;
      }
    };

    animationRef.current = requestAnimationFrame(animate);

    return () => {
      if (animationRef.current !== null) {
        cancelAnimationFrame(animationRef.current);
      }
    };
  }, [value, duration]);

  const formattedValue = formatNumber(displayValue, decimals);

  return (
    <View style={[styles.container, style]}>
      <Text style={[styles.value, textStyle]}>
        {prefix}
        {formattedValue}
        {suffix}
      </Text>
      {label ? (
        <Text style={[styles.label, labelStyle]}>{label}</Text>
      ) : null}
    </View>
  );
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function formatNumber(num: number, decimals: number): string {
  const fixed = Math.abs(num).toFixed(decimals);
  const parts = fixed.split('.');
  // Add thousands separators
  parts[0] = parts[0].replace(/\B(?=(\d{3})+(?!\d))/g, ',');
  const formatted = parts.join('.');
  return num < 0 ? `-${formatted}` : formatted;
}

// ---------------------------------------------------------------------------
// Styles
// ---------------------------------------------------------------------------

const styles = StyleSheet.create({
  container: {
    alignItems: 'center',
  },
  value: {
    fontSize: 36,
    fontWeight: '800',
    color: '#10b981', // green
    fontVariant: ['tabular-nums'],
  },
  label: {
    fontSize: 14,
    color: '#94a3b8',
    marginTop: 4,
  },
});

export default CounterAnimation;
