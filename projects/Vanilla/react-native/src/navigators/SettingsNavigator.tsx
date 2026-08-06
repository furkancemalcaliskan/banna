import i18n from '../utils/i18n';
import React from 'react';
import { useColorScheme } from 'react-native';
import { useSelector } from 'react-redux';
import { createColorModeSelector } from '../store/selectors/PersistentStorageSelectors';
import HamburgerIcon from '../components/HamburgerIcon/HamburgerIcon';
import { LocalizationContext } from '../contexts/LocalizationContext';
import ChangePasswordScreen from '../screens/ChangePassword/ChangePasswordScreen';
import ManageProfileScreen from '../screens/ManageProfile/ManageProfileScreen';
import SettingsScreen from '../screens/Settings/SettingsScreen';
import HeaderBackground from '../components/Layout/HeaderBackground';
import { createAppStackNavigator, getStackScreenOptions } from './StackFactory';
import { useAppSafeAreaTop, normalizeColorMode, getColorToken } from '../theme';

const Stack = createAppStackNavigator();

export default function SettingsStackNavigator() {
  const { t } = React.useContext(LocalizationContext);
  const colorMode = useSelector(createColorModeSelector());
  const systemScheme = useColorScheme();
  const activeMode = normalizeColorMode(colorMode, systemScheme);
  const headerTint = getColorToken(activeMode, 'navHeaderTint', 'navHeaderTintDark');
  const headerText = getColorToken(activeMode, 'navHeaderText', 'navHeaderTextDark');
  const safeAreaTop = useAppSafeAreaTop();

  return (
    <Stack.Navigator
      initialRouteName="Settings"
      screenOptions={{
        ...getStackScreenOptions({ headerStatusBarHeight: safeAreaTop }),
        headerBackground: () => <HeaderBackground />,
        headerTintColor: headerTint.value,
        headerTitleStyle: { color: headerText.value },
      }}
    >
      <Stack.Screen
        name="Settings"
        component={SettingsScreen}
        options={({ navigation }) => ({
          headerLeft: (props) => <HamburgerIcon navigation={navigation} color={props.tintColor} />,
          title: t('AbpSettingManagement::Settings'),
        })}
      />
      <Stack.Screen
        name="ChangePassword"
        component={ChangePasswordScreen}
        options={{
          title: i18n.t('AbpUi::ChangePassword'),
        }}
      />
      <Stack.Screen
        name="ManageProfile"
        component={ManageProfileScreen}
        options={{
          title: i18n.t('AbpAccount::MyAccount'),
        }}
      />
    </Stack.Navigator>
  );
}
