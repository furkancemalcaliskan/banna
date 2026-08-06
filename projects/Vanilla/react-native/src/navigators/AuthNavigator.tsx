import React, { useContext } from 'react';
import { useColorScheme } from 'react-native';
import { useSelector } from 'react-redux';
import { createColorModeSelector } from '../store/selectors/PersistentStorageSelectors';
import { LocalizationContext } from '../contexts/LocalizationContext';
import LoginScreen from '../screens/Login/LoginScreen';
import HeaderBackground from '../components/Layout/HeaderBackground';
import { createAppStackNavigator, getStackScreenOptions } from './StackFactory';
import { useAppSafeAreaTop } from '../theme';
import { normalizeColorMode, getColorToken } from '../theme';

const Stack = createAppStackNavigator();

export default function AuthNavigator() {
  const {t} = useContext(LocalizationContext);
  const colorMode = useSelector(createColorModeSelector());
  const systemScheme = useColorScheme();
  const activeMode = normalizeColorMode(colorMode, systemScheme);
  const isDark = activeMode === 'dark';
  const headerTint = getColorToken(activeMode, 'navHeaderTint', 'navHeaderTintDark');
  const headerText = getColorToken(activeMode, 'navHeaderText', 'navHeaderTextDark');
  const safeAreaTop = useAppSafeAreaTop();

  return (
    <Stack.Navigator screenOptions={getStackScreenOptions({ headerStatusBarHeight: safeAreaTop })}>
      <Stack.Screen
        name="Login"
        component={LoginScreen}
        options={() => ({
          title: t('AbpAccount::Login'),
          headerBackground: () => <HeaderBackground />,
          headerTintColor: headerTint.value,
          headerTitleStyle: { color: headerText.value }
        })}
      />
    </Stack.Navigator>
  );
}
