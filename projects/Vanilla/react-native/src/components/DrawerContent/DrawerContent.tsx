import { Ionicons } from '@expo/vector-icons';
import Constants from 'expo-constants';
import type { DrawerContentComponentProps } from '@react-navigation/drawer';
import {
  HStack,
  Icon,
  Pressable,
  Text,
  VStack,
  View,
  Box,
  PressableProps,
} from '@gluestack-ui/themed';
import React from 'react';
import { Image, StyleSheet } from 'react-native';
import { SafeAreaView } from 'react-native-safe-area-context';
import { withPermission } from '../../hocs/PermissionHOC';
import { LocalizationContext } from '../../contexts/LocalizationContext';
import i18n from '../../utils/i18n';
import { useSurfaceColors } from '../../theme';

type DrawerItemProps = PressableProps & {
  onPress: () => void;
  children?: React.ReactNode;
};

const screens: Record<
  string,
  { label: string; iconName: React.ComponentProps<typeof Ionicons>['name']; requiredPolicy?: string }
> = {
  HomeStack: { label: '::Menu:Home', iconName: 'home' },
  UsersStack: {
    label: 'AbpIdentity::Users',
    iconName: 'people',
    requiredPolicy: 'AbpIdentity.Users',
  },
  TenantsStack: {
    label: 'AbpTenantManagement::Tenants',
    iconName: 'book-outline',
    requiredPolicy: 'AbpTenantManagement.Tenants',
  },
  SettingsStack: { label: 'AbpSettingManagement::Settings', iconName: 'cog' },
};

const DrawerItem: React.FC<DrawerItemProps> = ({ onPress, bg, children, ...props }) => (
  <Pressable onPress={onPress} bg={bg} py="$3" px="$4" {...props}>
    <HStack space="md" alignItems="center">
      {children}
    </HStack>
  </Pressable>
);

const DrawerItemWithPermission = withPermission(DrawerItem);

const DrawerContent: React.FC<DrawerContentComponentProps> = ({
  navigation,
  state: { routeNames, index: currentScreenIndex },
}) => {
  const localization = React.useContext(LocalizationContext);
  const t = localization?.t || i18n.t;
  const surface = useSurfaceColors();
  const navigate = (screen: string) => {
    navigation.navigate(screen);
    navigation.closeDrawer();
  };

  return (
    <View style={styles.container}>
      <Box
        flex={1}
        bg={surface.sidebarBg}
        borderRightWidth={StyleSheet.hairlineWidth}
        borderColor={surface.borderColor}
      >
        <SafeAreaView
          style={styles.container}
          forceInset={{ top: 'always', horizontal: 'never' }}
        >
          <View style={styles.headerView}>
            <Image
              style={styles.logo}
              source={require('../../../assets/logo.png')}
              resizeMode="contain"
            />
          </View>
          <VStack
            flex={1}
            p="$4"
            pt="$12"
          >
            {routeNames.map((name) => {
              const isActive = name === routeNames[currentScreenIndex];
              return (
                <DrawerItemWithPermission
                  key={name}
                  policyKey={screens[name].requiredPolicy}
                  bg={isActive ? surface.activeHighlight : 'transparent'}
                  borderWidth={isActive ? 1 : 0}
                  borderColor={isActive ? surface.borderColor : 'transparent'}
                  onPress={() => navigate(name)}
                >
                  <Icon
                    as={Ionicons}
                    name={screens[name].iconName}
                    size="md"
                    color={isActive ? surface.primaryText : '$textLight700'}
                    _dark={{
                      color: isActive ? surface.primaryText : '$textDark',
                    }}
                  />
                  <Text
                    fontWeight={isActive ? '$bold' : '$normal'}
                    color={isActive ? surface.primaryText : '$textLight700'}
                    _dark={{
                      color: isActive ? surface.primaryText : '$textDark',
                    }}
                  >
                    {t(screens[name].label)}
                  </Text>
                </DrawerItemWithPermission>
              );
            })}
          </VStack>
        </SafeAreaView>
        <Box
          py="$4"
          px="$4"
          flexDirection="row"
          justifyContent="space-between"
          bg={surface.footerBg}
          borderTopWidth={1}
          borderColor={surface.borderColor}
        >
          <Text size="sm" color="$textLight400" _dark={{ color: surface.textMuted }}>
            © MyProjectName
          </Text>
          <Text size="sm" color="$textLight400" _dark={{ color: surface.textMuted }}>
            v{Constants.expoConfig.version}
          </Text>
        </Box>
      </Box>
    </View>
  );
};

const styles = StyleSheet.create({
  container: {
    flexGrow: 1,
  },
  logo: {
    width: 96,
    height: 96,
    marginTop: 20,
    marginBottom: 15,
  },
  headerView: {
    alignItems: 'center',
  },
});

export default DrawerContent;
