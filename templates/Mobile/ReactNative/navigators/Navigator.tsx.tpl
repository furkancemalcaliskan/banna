import { useColorScheme } from 'react-native';
import { useSelector } from 'react-redux';
import React from 'react';

import __entity_plural__Screen from '../screens/__entity_plural__/__entity_plural__Screen';
import CreateUpdate__entity_name__Screen from '../screens/__entity_plural__/CreateUpdate__entity_name__/CreateUpdate__entity_name__Screen';
import HamburgerIcon from '../components/HamburgerIcon/HamburgerIcon';
import HeaderBackground from '../components/Layout/HeaderBackground';
import { LocalizationContext } from '../contexts/LocalizationContext';
import { createColorModeSelector } from '../store/selectors/PersistentStorageSelectors';
import { createAppStackNavigator, getStackScreenOptions } from './StackFactory';
import {
  useAppSafeAreaTop,
  normalizeColorMode,
  getColorToken,
} from '../theme';

const Stack = createAppStackNavigator();

export default function __entity_name__StackNavigator(): JSX.Element {
  const { t } = React.useContext(LocalizationContext);
  const colorMode = useSelector(createColorModeSelector());
  const systemScheme = useColorScheme();
  const activeMode = normalizeColorMode(colorMode, systemScheme);
  const headerTint = getColorToken(activeMode, 'navHeaderTint', 'navHeaderTintDark');
  const headerText = getColorToken(activeMode, 'navHeaderText', 'navHeaderTextDark');
  const safeAreaTop = useAppSafeAreaTop();

  return (
    <Stack.Navigator
      initialRouteName="__entity_plural__"
      screenOptions={{
        ...getStackScreenOptions({ headerStatusBarHeight: safeAreaTop }),
        headerBackground: () => <HeaderBackground />,
        headerTintColor: headerTint.value,
        headerTitleStyle: { color: headerText.value },
      }}
    >
      <Stack.Screen
        name="__entity_plural__"
        component={__entity_plural__Screen}
        options={({ navigation }: any) => ({
          title: t('__domain_short__::Menu:__entity_plural__'),
          headerLeft: (props) => <HamburgerIcon navigation={navigation} color={props.tintColor} />,
          headerLeftContainerStyle: { paddingRight: 8 },
        })}
      />
      <Stack.Screen
        name="CreateUpdate__entity_name__"
        component={CreateUpdate__entity_name__Screen}
        options={({ route }: any) => ({
          title: t(
            route.params?.__entity_name_lower__Id
              ? 'AbpUi::Edit'
              : '__domain_short__::New__entity_name__'
          ),
        })}
      />
    </Stack.Navigator>
  );
}
