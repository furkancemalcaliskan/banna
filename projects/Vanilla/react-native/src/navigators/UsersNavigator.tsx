import React from 'react';
import { useColorScheme } from 'react-native';
import { useSelector } from 'react-redux';
import { createColorModeSelector } from '../store/selectors/PersistentStorageSelectors';
import AddIcon from '../components/AddIcon/AddIcon';
import HamburgerIcon from '../components/HamburgerIcon/HamburgerIcon';
import { LocalizationContext } from '../contexts/LocalizationContext';
import CreateUpdateUserScreen from '../screens/CreateUpdateUser/CreateUpdateUserScreen';
import UsersScreen from '../screens/Users/UsersScreen';
import HeaderBackground from '../components/Layout/HeaderBackground';
import { createAppStackNavigator, getStackScreenOptions } from './StackFactory';
import {
  useAppSafeAreaTop,
  normalizeColorMode,
  getColorToken,
} from '../theme';

const Stack = createAppStackNavigator();

export default function UsersStackNavigator() {
  const { t } = React.useContext(LocalizationContext);
  const colorMode = useSelector(createColorModeSelector());
  const systemScheme = useColorScheme();
  const activeMode = normalizeColorMode(colorMode, systemScheme);
  const headerTint = getColorToken(activeMode, 'navHeaderTint', 'navHeaderTintDark');
  const headerText = getColorToken(activeMode, 'navHeaderText', 'navHeaderTextDark');
  const safeAreaTop = useAppSafeAreaTop();

  return (
    <Stack.Navigator
      initialRouteName="Users"
      screenOptions={{
        ...getStackScreenOptions({ headerStatusBarHeight: safeAreaTop }),
        headerBackground: () => <HeaderBackground />,
        headerTintColor: headerTint.value,
        headerTitleStyle: { color: headerText.value },
      }}
    >
      <Stack.Screen
        name="Users"
        component={UsersScreen}
        options={({ navigation }) => ({
          title: t('AbpIdentity::Users'),
          headerLeft: (props) => <HamburgerIcon navigation={navigation} color={props.tintColor} />,
          headerLeftContainerStyle: { paddingRight: 8 },
          headerRight: () => null,
        })}
      />
      <Stack.Screen
        name="CreateUpdateUser"
        component={CreateUpdateUserScreen}
        options={({ route }) => ({
          title: t(route.params?.userId ? 'AbpIdentity::Edit' : 'AbpIdentity::NewUser'),
        })}
      />
    </Stack.Navigator>
  );
}
