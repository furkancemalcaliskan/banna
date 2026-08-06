import { Ionicons } from '@expo/vector-icons';
import Card from '../../components/Layout/Card';
import { useFocusEffect } from '@react-navigation/native';
import { useHeaderHeight } from '@react-navigation/elements';
import i18n from '../../utils/i18n';
import {
  Avatar,
  AvatarImage,
  AvatarFallbackText,
  Button,
  ButtonText,
  Divider,
  FormControl,
  FormControlLabel,
  FormControlLabelText,
  HStack,
  Icon,
  Text,
  VStack,
  Pressable,
  Box,
  ScrollView,
} from '@gluestack-ui/themed';
import React, { useCallback, useContext, useEffect, useRef, useState } from 'react';
import { useSafeAreaInsets } from 'react-native-safe-area-context';
import { RefreshControl, DevSettings } from 'react-native';
import { FormButtonBar } from '../../components/FormButtons';
import { ComboBox } from '../../components/ComboBox';
import { getProfileDetail } from '../../api/IdentityAPI';
import AppActions from '../../store/actions/AppActions';
import {
  createLanguageSelector,
  createLanguagesSelector,
} from '../../store/selectors/AppSelectors';
import {
  createTenantSelector,
  createColorModeSelector,
} from '../../store/selectors/PersistentStorageSelectors';
import PersistentStorageActions from '../../store/actions/PersistentStorageActions';
import { connectToRedux } from '../../utils/ReduxConnect';
import { store } from '../../store/index';
import { getEnvVars } from '../../../Environment';
import { LocalizationContext } from '../../contexts/LocalizationContext';

const env = getEnvVars();

type SettingsScreenProps = {
  navigation: any;
  language: any;
  languages: any[];
  setLanguageAsync: (language: any) => void;
  logoutAsync: (payload: any) => void;
  tenant?: Record<string, any>;
  colorMode?: string;
  setColorMode?: (mode: string) => void;
};

function SettingsScreen({
  navigation,
  language,
  languages,
  setLanguageAsync,
  logoutAsync,
  tenant = {},
  colorMode,
  setColorMode,
}: SettingsScreenProps) {
  const localization = useContext(LocalizationContext);
  const t = localization?.t || i18n.t;
  const locale = localization?.locale;
  const [user, setUser] = useState<Record<string, any>>({});
  const [token, setToken] = useState<any>(store.getState().persistentStorage.token);
  const [refreshing, setRefreshing] = useState(false);
  const lastLanguageRef = useRef(language?.cultureName || language);
  const headerHeight = useHeaderHeight();
  const insets = useSafeAreaInsets();
  const topInset = headerHeight > 0 ? 0 : insets.top || 0;
  const bottomInset = Math.max(insets.bottom || 0, 12);

  const fetchUser = () =>
    getProfileDetail().then(data => {
      setUser(data || {});
    });

  useFocusEffect(
    useCallback(() => {
      fetchUser();
    }, []),
  );

  const logout = () => {
    const { clientId } = env.oAuthConfig;
    const { access_token, refresh_token } = token;

    logoutAsync({
      client_id: clientId,
      token: access_token,
      refresh_token: refresh_token,
    });
  };

  const displayName = user.userName || '';

  useEffect(() => {
    navigation.setOptions({
      title: t('AbpSettingManagement::Settings'),
    });
  }, [navigation, t, locale]);

  const reloadApp = useCallback(async () => {
    try {
      const Updates = await import('expo-updates');
      if (Updates?.reloadAsync) {
        await Updates.reloadAsync();
        return;
      }
    } catch (error) {
      // Fallback to dev reload below
    }
    if (DevSettings?.reload) {
      DevSettings.reload();
    }
  }, []);

  useEffect(() => {
    const currentLang = language?.cultureName || language;
    if (lastLanguageRef.current && lastLanguageRef.current !== currentLang) {
      reloadApp();
    }
    lastLanguageRef.current = currentLang;
  }, [language, reloadApp]);

  return (
    <Box flex={1}>
      <ScrollView
        contentContainerStyle={{
          paddingTop: topInset,
          paddingBottom: 96 + bottomInset,
        }}
        refreshControl={
          <RefreshControl
            refreshing={refreshing}
            onRefresh={() => {
              setRefreshing(true);
              fetchUser().finally(() => setRefreshing(false));
            }}
          />
        }
      >
        {/* Profile Section */}
        <Card m="$4" mb="$2">
          <HStack space="md" alignItems="center" mb="$4">
            <Avatar size="lg" bg="$primary500">
              <AvatarFallbackText>{user.userName}</AvatarFallbackText>
              <AvatarImage source={require('../../../assets/avatar.png')} />
            </Avatar>
            <VStack>
              <Text size="lg" fontWeight="$bold">
                {user.userName}
              </Text>
              <Text
                size="sm"
                color="$textDark500"
                _dark={{ color: '$textDarkMuted' }}
              >
                {user.email}
              </Text>
            </VStack>
          </HStack>
          <Button
            size="sm"
            variant="outline"
            action="primary"
            onPress={() => navigation.navigate('ManageProfile')}
            borderRadius="$full"
            borderColor="$borderColor"
            _dark={{ borderColor: '$borderColorDark' }}
          >
            <ButtonText
              color="$primary600"
              _dark={{ color: '$textDark' }}
            >
              {t('AbpAccount::MyAccount')}
            </ButtonText>
          </Button>
        </Card>

        {/* Settings Options */}
        <Card m="$4" mt="$2">
          <VStack space="md" divider={<Divider bg="$themeBorderDim" />}>
            <Pressable
              onPress={() => navigation.navigate('ChangePassword')}
              py="$2"
            >
              <HStack justifyContent="space-between" alignItems="center">
                <HStack space="md" alignItems="center">
                  <Box
                    p="$2"
                    bg="$primary100"
                    _dark={{ bg: '$primary900' }}
                    borderRadius="$full"
                  >
                    <Icon
                      as={Ionicons}
                      name="key-outline"
                      size="md"
                      color="$primary600"
                      _dark={{ color: '$primary300' }}
                    />
                  </Box>
                  <Text size="md" fontWeight="$medium">
                    {t('AbpAccount::ChangePassword')}
                  </Text>
                </HStack>
                <Icon
                  as={Ionicons}
                  name="chevron-forward"
                  size="sm"
                  color="$textLight400"
                  _dark={{ color: '$textDarkMuted' }}
                />
              </HStack>
            </Pressable>

            <Box py="$2">
              <HStack
                justifyContent="space-between"
                alignItems="center"
                mb="$2"
              >
                <HStack space="md" alignItems="center">
                  <Box
                    p="$2"
                    bg="$info100"
                    _dark={{ bg: '$info900' }}
                    borderRadius="$full"
                  >
                    <Icon
                      as={Ionicons}
                      name="language-outline"
                      size="md"
                      color="$info600"
                      _dark={{ color: '$info300' }}
                    />
                  </Box>
                  <Text size="md" fontWeight="$medium">
                    {t('AbpUi::Language')}
                  </Text>
                </HStack>
              </HStack>
              <ComboBox
                value={language.cultureName}
                onValueChange={(val) => setLanguageAsync(val)}
                placeholder={t('AbpUi::Select')}
                options={languages.map((lang) => ({
                  label: lang.displayName,
                  value: lang.cultureName,
                }))}
              />
            </Box>

            <Box py="$2">
              <HStack
                justifyContent="space-between"
                alignItems="center"
                mb="$2"
              >
                <HStack space="md" alignItems="center">
                  <Box
                    p="$2"
                    bg="$secondary100"
                    _dark={{ bg: '$secondary800' }}
                    borderRadius="$full"
                  >
                    <Icon
                      as={Ionicons}
                      name="moon-outline"
                      size="md"
                      color="$secondary600"
                      _dark={{ color: '$secondary300' }}
                    />
                  </Box>
                  <Text size="md" fontWeight="$medium">
                    {t('Theme::ColorMode') || 'Theme'}
                  </Text>
                </HStack>
              </HStack>
              <ComboBox
                value={colorMode}
                onValueChange={(val) => setColorMode(val)}
                placeholder={t('AbpUi::Select')}
                options={[
                  { label: t('Theme::System') || 'System', value: 'system' },
                  { label: t('Theme::Light') || 'Light', value: 'light' },
                  { label: t('Theme::Dark') || 'Dark', value: 'dark' },
                ]}
              />
            </Box>
          </VStack>
        </Card>
      </ScrollView>
      <FormButtonBar>
        <Button
          action="negative"
          variant="solid"
          size="lg"
          borderRadius="$xl"
          onPress={logout}
          bg="$danger500"
          boxShadow="$soft"
        >
          <ButtonText fontWeight="$bold">
            {t('AbpAccount::Logout')}
          </ButtonText>
        </Button>
      </FormButtonBar>
    </Box>
  );
}

export default connectToRedux({
  component: SettingsScreen,
  stateProps: state => ({
    languages: createLanguagesSelector()(state),
    language: createLanguageSelector()(state),
    tenant: createTenantSelector()(state),
    colorMode: createColorModeSelector()(state),
  }),
  dispatchProps: {
    setLanguageAsync: AppActions.setLanguageAsync,
    logoutAsync: AppActions.logoutAsync,
    setColorMode: PersistentStorageActions.setColorMode,
  },
});
