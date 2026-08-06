import { createDrawerNavigator } from '@react-navigation/drawer';
import React from 'react';
import { useColorScheme } from 'react-native';
import { useSelector } from 'react-redux';
import { createColorModeSelector } from '../store/selectors/PersistentStorageSelectors';
import DrawerContent from '../components/DrawerContent/DrawerContent';
import HamburgerIcon from '../components/HamburgerIcon/HamburgerIcon';
import { LocalizationContext } from '../contexts/LocalizationContext';
import HomeStackNavigator from './HomeNavigator';
import SettingsStackNavigator from './SettingsNavigator';
import TenantsStackNavigator from './TenantsNavigator';
import UsersStackNavigator from './UsersNavigator';
import HeaderBackground from '../components/Layout/HeaderBackground';
import {
  appLayout,
  getColorToken,
  normalizeColorMode,
  useAppSafeAreaTop,
} from '../theme';

const Drawer = createDrawerNavigator();

export default function DrawerNavigator() {
  const { t } = React.useContext(LocalizationContext);
  const colorMode = useSelector(createColorModeSelector());
  const systemScheme = useColorScheme();
  const activeMode = normalizeColorMode(colorMode, systemScheme);
  const headerTint = getColorToken(activeMode, 'navHeaderTint', 'navHeaderTintDark');
  const headerText = getColorToken(activeMode, 'navHeaderText', 'navHeaderTextDark');
  const drawerBg = getColorToken(activeMode, 'navDrawerBg', 'navDrawerBgDark');
  const safeAreaTop = useAppSafeAreaTop();

  return (
    <Drawer.Navigator
      initialRouteName="HomeStack"
      drawerContent={DrawerContent}
      screenOptions={{
        headerBackground: () => <HeaderBackground />,
        headerStatusBarHeight: safeAreaTop,
        headerTintColor: headerTint.value,
        headerTitleStyle: {
          color: headerText.value,
        },
        drawerStyle: {
          backgroundColor: drawerBg.value,
          width: appLayout.drawerWidth,
        },
      }}
    >
      <Drawer.Screen
        name="HomeStack"
        component={HomeStackNavigator}
        options={({ navigation }) => ({
          title: t('::Menu:Home'),
          headerLeft: (props) => <HamburgerIcon navigation={navigation} color={props.tintColor} />,
        })}
      />
      <Drawer.Screen
        name="TenantsStack"
        component={TenantsStackNavigator}
        options={{ header: () => null }}
      />
      <Drawer.Screen
        name="UsersStack"
        component={UsersStackNavigator}
        options={{ header: () => null }}
      />
      <Drawer.Screen
        name="SettingsStack"
        component={SettingsStackNavigator}
        options={{ header: () => null }}
      />
    </Drawer.Navigator>
  );
}
