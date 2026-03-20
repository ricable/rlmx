/**
 * PulseAnimation — animated microphone button with pulsing circle effect.
 *
 * Visual states:
 *  - Idle:       cyan (#00d4ff) with subtle breathing pulse
 *  - Listening:  violet (#7c3aed) with expanding ripples
 *  - Processing: amber (#f59e0b) with fast pulse
 */

import React, { useEffect, useRef } from 'react';
import {
  Animated,
  Easing,
  Pressable,
  StyleSheet,
  Text,
  View,
  ViewStyle,
} from 'react-native';

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

export type PulseState = 'idle' | 'listening' | 'processing';

interface PulseAnimationProps {
  state: PulseState;
  onPress?: () => void;
  size?: number;
  style?: ViewStyle;
}

// ---------------------------------------------------------------------------
// Color map
// ---------------------------------------------------------------------------

const STATE_COLORS: Record<PulseState, string> = {
  idle: '#00d4ff',       // cyan
  listening: '#7c3aed',  // violet
  processing: '#f59e0b', // amber
};

const STATE_ICONS: Record<PulseState, string> = {
  idle: 'Tap',
  listening: 'Listening...',
  processing: 'Thinking...',
};

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

export const PulseAnimation: React.FC<PulseAnimationProps> = ({
  state,
  onPress,
  size = 80,
  style,
}) => {
  const pulseScale = useRef(new Animated.Value(1)).current;
  const pulseOpacity = useRef(new Animated.Value(0.4)).current;
  const ripple1Scale = useRef(new Animated.Value(1)).current;
  const ripple1Opacity = useRef(new Animated.Value(0.3)).current;
  const ripple2Scale = useRef(new Animated.Value(1)).current;
  const ripple2Opacity = useRef(new Animated.Value(0.3)).current;
  const colorAnim = useRef(new Animated.Value(0)).current;

  // Color transition
  useEffect(() => {
    Animated.timing(colorAnim, {
      toValue: state === 'idle' ? 0 : state === 'listening' ? 1 : 2,
      duration: 300,
      useNativeDriver: false,
    }).start();
  }, [state, colorAnim]);

  // Pulse animation
  useEffect(() => {
    pulseScale.setValue(1);
    pulseOpacity.setValue(0.4);

    const duration = state === 'processing' ? 600 : state === 'listening' ? 1200 : 2000;
    const maxScale = state === 'processing' ? 1.15 : state === 'listening' ? 1.25 : 1.08;

    const pulse = Animated.loop(
      Animated.sequence([
        Animated.parallel([
          Animated.timing(pulseScale, {
            toValue: maxScale,
            duration: duration / 2,
            easing: Easing.inOut(Easing.ease),
            useNativeDriver: true,
          }),
          Animated.timing(pulseOpacity, {
            toValue: 0.15,
            duration: duration / 2,
            easing: Easing.inOut(Easing.ease),
            useNativeDriver: true,
          }),
        ]),
        Animated.parallel([
          Animated.timing(pulseScale, {
            toValue: 1,
            duration: duration / 2,
            easing: Easing.inOut(Easing.ease),
            useNativeDriver: true,
          }),
          Animated.timing(pulseOpacity, {
            toValue: 0.4,
            duration: duration / 2,
            easing: Easing.inOut(Easing.ease),
            useNativeDriver: true,
          }),
        ]),
      ]),
    );

    pulse.start();
    return () => pulse.stop();
  }, [state, pulseScale, pulseOpacity]);

  // Ripple animations (listening mode only)
  useEffect(() => {
    if (state !== 'listening') {
      ripple1Scale.setValue(1);
      ripple1Opacity.setValue(0);
      ripple2Scale.setValue(1);
      ripple2Opacity.setValue(0);
      return;
    }

    const createRipple = (
      scale: Animated.Value,
      opacity: Animated.Value,
      delay: number,
    ) =>
      Animated.loop(
        Animated.sequence([
          Animated.delay(delay),
          Animated.parallel([
            Animated.timing(scale, {
              toValue: 2.5,
              duration: 1500,
              easing: Easing.out(Easing.ease),
              useNativeDriver: true,
            }),
            Animated.timing(opacity, {
              toValue: 0,
              duration: 1500,
              easing: Easing.out(Easing.ease),
              useNativeDriver: true,
            }),
          ]),
          Animated.parallel([
            Animated.timing(scale, {
              toValue: 1,
              duration: 0,
              useNativeDriver: true,
            }),
            Animated.timing(opacity, {
              toValue: 0.3,
              duration: 0,
              useNativeDriver: true,
            }),
          ]),
        ]),
      );

    const r1 = createRipple(ripple1Scale, ripple1Opacity, 0);
    const r2 = createRipple(ripple2Scale, ripple2Opacity, 750);

    r1.start();
    r2.start();

    return () => {
      r1.stop();
      r2.stop();
    };
  }, [state, ripple1Scale, ripple1Opacity, ripple2Scale, ripple2Opacity]);

  const color = STATE_COLORS[state];
  const halfSize = size / 2;

  return (
    <View style={[styles.container, { width: size * 3, height: size * 3 }, style]}>
      {/* Ripple 1 */}
      <Animated.View
        style={[
          styles.ripple,
          {
            width: size,
            height: size,
            borderRadius: halfSize,
            borderColor: color,
            transform: [{ scale: ripple1Scale }],
            opacity: ripple1Opacity,
          },
        ]}
      />

      {/* Ripple 2 */}
      <Animated.View
        style={[
          styles.ripple,
          {
            width: size,
            height: size,
            borderRadius: halfSize,
            borderColor: color,
            transform: [{ scale: ripple2Scale }],
            opacity: ripple2Opacity,
          },
        ]}
      />

      {/* Pulse background */}
      <Animated.View
        style={[
          styles.pulseCircle,
          {
            width: size * 1.4,
            height: size * 1.4,
            borderRadius: (size * 1.4) / 2,
            backgroundColor: color,
            transform: [{ scale: pulseScale }],
            opacity: pulseOpacity,
          },
        ]}
      />

      {/* Main button */}
      <Pressable
        onPress={onPress}
        style={[
          styles.button,
          {
            width: size,
            height: size,
            borderRadius: halfSize,
            backgroundColor: color,
          },
        ]}
      >
        <Text style={styles.buttonText}>{STATE_ICONS[state]}</Text>
      </Pressable>
    </View>
  );
};

// ---------------------------------------------------------------------------
// Styles
// ---------------------------------------------------------------------------

const styles = StyleSheet.create({
  container: {
    alignItems: 'center',
    justifyContent: 'center',
  },
  ripple: {
    position: 'absolute',
    borderWidth: 2,
  },
  pulseCircle: {
    position: 'absolute',
  },
  button: {
    alignItems: 'center',
    justifyContent: 'center',
    elevation: 8,
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 4 },
    shadowOpacity: 0.3,
    shadowRadius: 8,
  },
  buttonText: {
    color: '#ffffff',
    fontSize: 12,
    fontWeight: '700',
    textAlign: 'center',
  },
});

export default PulseAnimation;
