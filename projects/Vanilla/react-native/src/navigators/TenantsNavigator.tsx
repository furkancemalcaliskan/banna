
import React from 'react';
import { useColorScheme } from 'react-native';
import { useSelector } from 'react-redux';
import { createColorModeSelector } from '../store/selectors/PersistentStorageSelectors';
import TenantsScreen from '../screens/Tenants/TenantsScreen';
import CreateUpdateTenantScreen from '../screens/CreateUpdateTenant/CreateUpdateTenantScreen';
import { LocalizationContext } from '../contexts/LocalizationContext';
import HamburgerIcon from '../components/HamburgerIcon/HamburgerIcon';
import AddIcon from '../components/AddIcon/AddIcon';
import HeaderBackground from '../components/Layout/HeaderBackground';
import { createAppStackNavigator, getStackScreenOptions } from './StackFactory';
import { useAppSafeAreaTop, normalizeColorMode, getColorToken } from '../theme';

const Stack = createAppStackNavigator();

export default function TenantsStackNavigator() {
  const { t } = React.useContext(LocalizationContext);
  const colorMode = useSelector(createColorModeSelector());
  const systemScheme = useColorScheme();
  const activeMode = normalizeColorMode(colorMode, systemScheme);
  const headerTint = getColorToken(activeMode, 'navHeaderTint', 'navHeaderTintDark');
  const headerText = getColorToken(activeMode, 'navHeaderText', 'navHeaderTextDark');
  const safeAreaTop = useAppSafeAreaTop();

  return (
    <Stack.Navigator
      initialRouteName="Tenants"
      screenOptions={{
        ...getStackScreenOptions({ headerStatusBarHeight: safeAreaTop }),
        headerBackground: () => <HeaderBackground />,
        headerTintColor: headerTint.value,
        headerTitleStyle: { color: headerText.value },
      }}
    >
      <Stack.Screen
        name="Tenants"
        component={TenantsScreen}
        options={({ navigation }) => ({
          title: t('AbpTenantManagement::Tenants'),
          headerLeft: (props) => <HamburgerIcon navigation={navigation} color={props.tintColor} />,
          headerRight: () => null,
        })}
      />
      <Stack.Screen
        name="CreateUpdateTenant"
        component={CreateUpdateTenantScreen}
        options={({ route }) => ({
          title: t(route.params?.tenantId ? 'AbpTenantManagement::Edit' : 'AbpTenantManagement::NewTenant'),
        })}
      />
    </Stack.Navigator>
  );
}
