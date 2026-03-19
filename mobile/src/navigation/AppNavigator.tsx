import React from 'react';
import {createBottomTabNavigator} from '@react-navigation/bottom-tabs';
import {createStackNavigator} from '@react-navigation/stack';
import Icon from 'react-native-vector-icons/MaterialIcons';
import {colors, fontSize} from '../theme';

import HomeScreen from '../screens/HomeScreen';
import AgentsScreen from '../screens/AgentsScreen';
import VoiceScreen from '../screens/VoiceScreen';
import InsightsScreen from '../screens/InsightsScreen';
import ProfileScreen from '../screens/ProfileScreen';
import MarketplaceScreen from '../screens/MarketplaceScreen';
import AgentDetailScreen from '../screens/AgentDetailScreen';
import AgentCollectionScreen from '../screens/AgentCollectionScreen';

const Tab = createBottomTabNavigator();
const Stack = createStackNavigator();

const tabIcons: Record<string, string> = {
  Home: 'home',
  Agents: 'smart-toy',
  Voice: 'mic',
  Insights: 'insights',
  Profile: 'person',
};

function TabNavigator() {
  return (
    <Tab.Navigator
      initialRouteName="Home"
      screenOptions={({route}) => ({
        headerShown: false,
        tabBarStyle: {
          backgroundColor: colors.surface,
          borderTopColor: colors.surfaceBorder,
          borderTopWidth: 1,
          height: 64,
          paddingBottom: 8,
          paddingTop: 4,
          elevation: 0,
        },
        tabBarActiveTintColor: colors.cyan,
        tabBarInactiveTintColor: colors.textMuted,
        tabBarLabelStyle: {
          fontSize: fontSize.xs,
          fontWeight: '600' as const,
        },
        tabBarIcon: ({color, size}) => (
          <Icon name={tabIcons[route.name] || 'help'} size={size} color={color} />
        ),
      })}>
      <Tab.Screen name="Home" component={HomeScreen} />
      <Tab.Screen name="Agents" component={AgentsScreen} />
      <Tab.Screen
        name="Voice"
        component={VoiceScreen}
        options={{
          tabBarIcon: ({focused}) => (
            <Icon
              name="mic"
              size={28}
              color={focused ? colors.cyan : colors.textMuted}
              style={{
                backgroundColor: focused ? colors.cyanDim : colors.surfaceLight,
                borderRadius: 20,
                width: 40,
                height: 40,
                textAlign: 'center',
                textAlignVertical: 'center',
                lineHeight: 40,
                overflow: 'hidden',
              }}
            />
          ),
        }}
      />
      <Tab.Screen name="Insights" component={InsightsScreen} />
      <Tab.Screen name="Profile" component={ProfileScreen} />
    </Tab.Navigator>
  );
}

export default function AppNavigator() {
  return (
    <Stack.Navigator
      screenOptions={{
        headerShown: false,
        cardStyle: {backgroundColor: colors.background},
      }}>
      <Stack.Screen name="Tabs" component={TabNavigator} />
      <Stack.Screen name="Marketplace" component={MarketplaceScreen} />
      <Stack.Screen name="AgentDetail" component={AgentDetailScreen} />
      <Stack.Screen name="AgentCollection" component={AgentCollectionScreen} />
    </Stack.Navigator>
  );
}
