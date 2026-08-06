import { NavigationContainer, DefaultTheme as NavigationDefaultTheme } from '@react-navigation/native';
import { createNativeStackNavigator } from '@react-navigation/native-stack';
import { GluestackUIProvider, useToast } from '@gluestack-ui/themed';
import { LinearGradient } from 'expo-linear-gradient';
import React, { useEffect, useMemo, useState } from 'react';
import { useColorScheme } from 'react-native';
import { SafeAreaProvider } from 'react-native-safe-area-context';
import config, { normalizeColorMode } from './src/theme';
import { enableScreens } from 'react-native-screens';
import { Provider, useSelector } from 'react-redux';
import { PersistGate } from 'redux-persist/integration/react';
import { getEnvVars } from './Environment';
import Loading from './src/components/Loading/Loading';
import Background from './src/components/Layout/Background';
import { LocalizationContext } from './src/contexts/LocalizationContext';
import { initAPIInterceptor, setAPIInterceptorToast } from './src/interceptors/APIInterceptor';
import AuthNavigator from './src/navigators/AuthNavigator';
import DrawerNavigator from './src/navigators/DrawerNavigator';
import { persistor, store } from './src/store';
import AppActions from './src/store/actions/AppActions';
import PersistentStorageActions from './src/store/actions/PersistentStorageActions';
import { createAppConfigSelector, createLanguageSelector } from './src/store/selectors/AppSelectors';
import { createTokenSelector, createColorModeSelector } from './src/store/selectors/PersistentStorageSelectors';
import { connectToRedux } from './src/utils/ReduxConnect';
import { isTokenValid } from './src/utils/TokenUtils';
import i18n from './src/utils/i18n';
import {
  registerTranslations,
  resolveLocale,
  setLocale,
} from './src/localization/appLocalization';
import type { Theme } from '@react-navigation/native';
import type { ReactNode } from 'react';

const Stack = createNativeStackNavigator();

const { localization } = getEnvVars();

i18n.defaultSeparator = '::';
registerTranslations(i18n);

const cloneT = i18n.t.bind(i18n);
i18n.t = (key, ...args) => {
  if (key.slice(0, 2) === '::') {
    key = localization.defaultResourceName + key;
  }
  return cloneT(key, ...args);
};

enableScreens();

type AppInitializerProps = {
  onReady?: () => void;
  children?: ReactNode;
};

const AppInitializer: React.FC<AppInitializerProps> = ({ onReady, children }) => {
  useEffect(() => {
    initAPIInterceptor(store);
    const state = store.getState();
    const storedLanguage = state?.persistentStorage?.language;
    const storedColorMode = state?.persistentStorage?.colorMode;
    const systemLocale =
      Intl?.DateTimeFormat?.().resolvedOptions?.().locale || 'en';
    const resolvedLanguage = resolveLocale(
      storedLanguage || systemLocale,
    );

    if (!storedLanguage || storedLanguage !== resolvedLanguage) {
      store.dispatch(PersistentStorageActions.setLanguage(resolvedLanguage));
    }
    setLocale(i18n, resolvedLanguage);
    if (!storedColorMode) {
      store.dispatch(PersistentStorageActions.setColorMode('system'));
    }
    store.dispatch(
      AppActions.fetchAppConfigAsync({
        callback: onReady,
        showLoading: true,
      })
    );
  }, []);
  return children;
};

type ThemeWrapperProps = {
  children?: ReactNode;
};

const ThemeWrapper: React.FC<ThemeWrapperProps> = ({ children }) => {
  const colorMode = useSelector(createColorModeSelector());
  const systemScheme = useColorScheme();
  const activeMode = normalizeColorMode(colorMode, systemScheme);

  return (
    <GluestackUIProvider config={config} colorMode={activeMode}>
      {children}
    </GluestackUIProvider>
  );
};

const DefaultTheme: Theme = {
  ...NavigationDefaultTheme,
  dark: true, // Dark mode'u etkinleştir
  colors: {
    ...NavigationDefaultTheme.colors,
    background: 'transparent',
    card: '#1e293b', // Dark mode card background
    text: '#e8eefc', // Dark mode text color
    border: '#334155', // Dark mode border color
    primary: '#3b82f6', // Primary color
  },
};

type LocalizationProviderProps = {
  children?: ReactNode;
};

const LocalizationProvider: React.FC<LocalizationProviderProps> = ({ children }) => {
  const language = useSelector(createLanguageSelector());
  const locale = useMemo(
    () => resolveLocale(language?.cultureName || language),
    [language],
  );

  useEffect(() => {
    setLocale(i18n, locale);
  }, [locale]);

  const localizationContextValue = useMemo(
    () => ({
      t: i18n.t,
      locale,
    }),
    [locale]
  );

  return (
    <LocalizationContext.Provider value={localizationContextValue}>
      {children}
    </LocalizationContext.Provider>
  );
};

export default function App() {
  const [isReady, setIsReady] = useState(false);

  return (
    <Provider store={store}>
      <PersistGate loading={null} persistor={persistor}>
        <AppInitializer onReady={() => setIsReady(true)}>
          <ThemeWrapper>
            <SafeAreaProvider>
              <Background>
                <NavigationRoot isReady={isReady} />
              </Background>
            </SafeAreaProvider>
          </ThemeWrapper>
        </AppInitializer>
      </PersistGate>
    </Provider>
  );
}

type NavigationRootProps = {
  isReady: boolean;
};

const NavigationRoot: React.FC<NavigationRootProps> = ({ isReady }) => {
  const token = useSelector(createTokenSelector());
  const isValid = useMemo(() => isTokenValid(token), [token]);
  const navKey = isValid ? 'auth' : 'guest';

  return (
    <NavigationContainer key={navKey} theme={DefaultTheme}>
      {isReady ? (
        <LocalizationProvider>
          <ConnectedAppContainer />
        </LocalizationProvider>
      ) : null}
      <Loading />
    </NavigationContainer>
  );
};

type AppContainerProps = {
  token: any;
  setToken: (next: any) => void;
};

const AppContainer: React.FC<AppContainerProps> = ({ token, setToken }) => {
  const isValid = useMemo(() => isTokenValid(token), [token]);
  const toast = useToast();
  const appConfig = useSelector(createAppConfigSelector());
  const drawerKey = useMemo(() => {
    const policies = appConfig?.auth?.grantedPolicies || {};
    return `drawer-${Object.keys(policies).length}`;
  }, [appConfig]);

  useEffect(() => {
    setAPIInterceptorToast(toast);
  }, [toast]);

  useEffect(() => {
    if (!isValid && token && token.access_token) {
      setToken({})
    }
  }, [isValid]);

  return isValid ? <DrawerNavigator key={drawerKey} /> : <AuthNavigator />
};


const ConnectedAppContainer = connectToRedux({
  component: AppContainer,
  stateProps: (state) => ({
    token: createTokenSelector()(state),
  }),
  dispatchProps: {
    setToken: PersistentStorageActions.setToken,
  },
});
