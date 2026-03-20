import React from 'react';
import {View, StyleSheet} from 'react-native';
import {colors} from '../theme';

interface Props {
  data: number[];
  width?: number;
  height?: number;
  color?: string;
  strokeWidth?: number;
}

export default function SparklineChart({
  data,
  width = 80,
  height = 30,
  color = colors.cyan,
  strokeWidth = 2,
}: Props) {
  if (data.length < 2) {return null;}

  const min = Math.min(...data);
  const max = Math.max(...data);
  const range = max - min || 1;

  const points = data.map((val, i) => ({
    x: (i / (data.length - 1)) * width,
    y: height - ((val - min) / range) * (height - strokeWidth * 2) - strokeWidth,
  }));

  // Render as simple bars since react-native-svg is not installed
  const barWidth = Math.max(1, (width / data.length) - 1);

  return (
    <View style={[styles.container, {width, height}]}>
      {data.map((val, i) => {
        const barHeight = Math.max(2, ((val - min) / range) * (height - 4));
        return (
          <View
            key={i}
            style={[
              styles.bar,
              {
                width: barWidth,
                height: barHeight,
                backgroundColor: color,
                opacity: 0.4 + (i / data.length) * 0.6,
              },
            ]}
          />
        );
      })}
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flexDirection: 'row',
    alignItems: 'flex-end',
    gap: 1,
  },
  bar: {
    borderRadius: 1,
  },
});
