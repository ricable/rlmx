import React from 'react';
import {StatusBar} from 'react-native';
import {NavigationContainer} from '@react-navigation/native';
import {SafeAreaProvider} from 'react-native-safe-area-context';
import {GestureHandlerRootView} from 'react-native-gesture-handler';
import {AppProvider} from './src/context/AppContext';
import AppNavigator from './src/navigation/AppNavigator';
import {colors} from './src/theme';

function App() {
  return (
    <GestureHandlerRootView style={{flex: 1}}>
      <SafeAreaProvider>
        <NavigationContainer
          theme={{
            dark: true,
            colors: {
              primary: colors.cyan,
              background: colors.background,
              card: colors.surface,
              text: colors.text,
              border: colors.surfaceBorder,
              notification: colors.cyan,
            },
            fonts: {
              regular: {fontFamily: 'System', fontWeight: '400' as const},
              medium: {fontFamily: 'System', fontWeight: '500' as const},
              bold: {fontFamily: 'System', fontWeight: '700' as const},
              heavy: {fontFamily: 'System', fontWeight: '900' as const},
            },
          }}>
          <StatusBar barStyle="light-content" backgroundColor={colors.background} />
          <AppProvider>
            <AppNavigator />
          </AppProvider>
        </NavigationContainer>
      </SafeAreaProvider>
    </GestureHandlerRootView>
  );
}

export default App;
