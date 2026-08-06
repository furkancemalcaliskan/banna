import { createConfig } from '@gluestack-ui/themed';
import { config as defaultConfig } from '@gluestack-ui/config';
import { Dimensions } from 'react-native';
import { useColorScheme } from 'react-native';
import { useSelector } from 'react-redux';
import { createColorModeSelector } from '../store/selectors/PersistentStorageSelectors';

const { width } = Dimensions.get('window');
const fontScale = Math.max(0.9, Math.min(1.15, width / 375));
const scaleFont = (value: any) =>
  typeof value === 'number' ? Math.round(value * fontScale) : value;

const scaleTokenMap = (tokenMap: Record<string, any>) =>
  Object.fromEntries(
    Object.entries(tokenMap).map(([key, value]) => [key, scaleFont(value)])
  );
import { useSafeAreaInsets } from 'react-native-safe-area-context';

export const useAppSafeAreaTop = () => {
  const insets = useSafeAreaInsets();
  return insets.top || 0;
};

export const useAppColorMode = () => {
  const colorMode = useSelector(createColorModeSelector());
  const systemScheme = useColorScheme();
  return normalizeColorMode(colorMode, systemScheme);
};

export const useInputColors = () => {
  const mode = useAppColorMode();
  const isDark = mode === 'dark';

  return {
    bg: isDark ? '$themeInputBgDark' : '$themeInputBg',
    borderColor: isDark ? '$themeBorderDark' : '$themeBorder',
    textColor: isDark ? '$textDark900' : '$textLight900',
    placeholderColor: isDark ? '$textDarkMuted' : '$textLightMuted',
    iconColor: isDark ? '$textDarkMuted' : '$textLight400',
    activeHighlight: isDark ? '$activeHighlightDark' : '$activeHighlight',
    cardBg: isDark ? '$cardBgDark' : '$cardBg',
  };
};

export const useSurfaceColors = () => {
  const mode = useAppColorMode();
  const isDark = mode === 'dark';

  return {
    sidebarBg: isDark ? '$sidebarBgDark' : '$sidebarBg',
    borderColor: isDark ? '$themeBorderDark' : '$themeBorder',
    textMuted: isDark ? '$textDarkMuted' : '$textLight400',
    primaryText: isDark ? '$primary200' : '$primary600',
    activeHighlight: isDark ? '$activeHighlightDark' : '$activeHighlight',
    footerBg: isDark ? '$themeSurfaceBgDark' : '$secondary100',
    shadowColor: isDark ? '#000000' : '#0a162d',
  };
};

export const usePopoverColors = () => {
  const mode = useAppColorMode();
  const isDark = mode === 'dark';

  return {
    bg: isDark ? '$cardBgDark' : '$cardBg',
    borderColor: isDark ? '$borderColorDark' : '$borderColor',
    textColor: isDark ? '$textDark' : '$textLight900',
    mutedText: isDark ? '$textDarkMuted' : '$textLight700',
    iconColor: isDark ? '$textDarkMuted' : '$secondary500',
    pressedBg: isDark ? '$activeHighlightDark' : '$activeHighlight',
  };
};

export const useNavIconColor = () => {
  const mode = useAppColorMode();
  const isDark = mode === 'dark';
  return isDark ? '$navHeaderTextDark' : '$navHeaderText';
};

export const normalizeColorMode = (colorMode: string | undefined | null, systemScheme: string | null = 'light') => {
  const normalized = (colorMode || 'system').toLowerCase();
  return normalized === 'system' ? systemScheme || 'light' : normalized;
};

export const getColorToken = (activeMode: string | undefined | null, lightKey: string, darkKey: string) => {
  const isDark = (activeMode || 'light').toLowerCase() === 'dark';
  const key = isDark ? darkKey : lightKey;
  return {
    token: `$${key}`,
    value: (config.tokens as any).colors[key],
  };
};

export const appLayout = {
  drawerWidth: '68%',
};

const config = createConfig({
  ...defaultConfig,
  tokens: {
    ...defaultConfig.tokens,
    colors: {
      ...defaultConfig.tokens.colors,
      primary0: '#eef6fb',
      primary50: '#e3f0fa',
      primary100: '#d7e8f7',
      primary200: '#b9d7f2',
      primary300: '#93c3eb',
      primary400: '#5fa6df',
      primary500: '#268bd2',
      primary600: '#1f6aa5',
      primary700: '#1a557f',
      primary800: '#124164',
      primary900: '#0c304c',

      secondary0: '#f9f2de',
      secondary50: '#f6ecd1',
      secondary100: '#f2e3bf',
      secondary200: '#e9d6a3',
      secondary300: '#d8bf7d',
      secondary400: '#caa75a',
      secondary500: '#586e75',
      secondary600: '#42545a',
      secondary700: '#2f3c42',
      secondary800: '#233037',
      secondary900: '#1a262d',

      success500: '#859900',
      danger500: '#dc322f',
      info500: '#2aa198',
      warning500: '#b58900',

      backgroundLight: '#fdf6e3',
      backgroundDark: '#002b36',

      cardBg: '#fdfaf3',
      borderColor: 'rgba(88, 110, 117, 0.22)',
      separatorColor: 'rgba(88, 110, 117, 0.14)',
      inputBg: '#ffffff',
      sidebarBg: '#fdf6e3',
      textLight: '#073642',
      textLight900: '#073642',
      textLight800: '#0f4a56',
      textLight700: '#295464',
      textLight500: '#4f646b',
      textLightMuted: '#586e75',

      cardBgDark: '#073642',
      borderColorDark: 'rgba(148, 174, 184, 0.2)',
      inputBgDark: '#073642',
      sidebarBgDark: '#002b36',
      textDark: '#eee8d5',
      textDark900: '#eee8d5',
      textDark800: '#e6ddc1',
      textDark700: '#d9cfae',
      textDark600: '#cfc6a3',
      textDark500: '#c3b896',
      textDarkMuted: '#b3ab8a',

      themeSurfaceBg: '#fdfaf3',
      themeSurfaceBgDark: '#073642',
      themeInputBg: '#ffffff',
      themeInputBgDark: '#073642',
      themeBorder: 'rgba(88, 110, 117, 0.22)',
      themeBorderDim: 'rgba(88, 110, 117, 0.14)',
      themeBorderDark: 'rgba(148, 174, 184, 0.2)',

      navHeaderTint: '#268bd2',
      navHeaderTintDark: '#5fa6df',
      navHeaderText: '#073642',
      navHeaderTextDark: '#eee8d5',
      navDrawerBg: '#fdf6e3',
      navDrawerBgDark: '#002b36',
      backdropColor: 'rgba(0, 0, 0, 0.35)',

      buttonBg: '#268bd2',
      buttonBgDark: '#5fa6df',
      buttonBorder: '#1f6aa5',
      buttonBorderDark: '#2a7fcf',
      buttonTextLight: '#ffffff',
      buttonTextDark: '#ffffff',

      activeHighlight: 'rgba(38, 139, 210, 0.16)',
      activeHighlightDark: 'rgba(95, 166, 223, 0.18)',
    },
    fontSizes: {
      ...scaleTokenMap(defaultConfig.tokens.fontSizes),
    },
    lineHeights: {
      ...scaleTokenMap(defaultConfig.tokens.lineHeights),
    },
    radii: {
      ...defaultConfig.tokens.radii,
      md: 8,
      lg: 12,
      xl: 16,
      '2xl': 20,
    },
    shadows: {
      ...defaultConfig.tokens.shadows,
      soft: {
        shadowColor: '#0a162d',
        shadowOffset: { width: 0, height: 4 },
        shadowOpacity: 0.08,
        shadowRadius: 8,
        elevation: 4,
      },
      softDark: {
        shadowColor: '#000000',
        shadowOffset: { width: 0, height: 4 },
        shadowOpacity: 0.22,
        shadowRadius: 10,
        elevation: 6,
      },
      glow: {
        shadowColor: 'transparent',
        shadowOffset: { width: 0, height: 0 },
        shadowOpacity: 0,
        shadowRadius: 0,
        elevation: 0,
      },
      glowDark: {
        shadowColor: 'transparent',
        shadowOffset: { width: 0, height: 0 },
        shadowOpacity: 0,
        shadowRadius: 0,
        elevation: 0,
      },
    },
  },
  components: {
    ...(defaultConfig.components || {}),
    Input: {
      theme: {
        ...(defaultConfig.components?.Input?.theme || {}),
        borderWidth: 1,
        borderRadius: '$lg',
        _light: {
          bg: '$themeInputBg',
          borderColor: '$themeBorder',
        },
        _dark: {
          bg: '$themeInputBgDark',
          borderColor: '$themeBorderDark',
        },
      },
    },
    InputField: {
      theme: {
        ...(defaultConfig.components?.InputField?.theme || {}),
        _light: {
          color: '$textLight900',
          placeholderTextColor: '$textLightMuted',
        },
        _dark: {
          color: '$textDark900',
          placeholderTextColor: '$textDarkMuted',
        },
      },
    },
  },
  globalStyle: {
    fontFamily: 'System',
  },
});

export default config;
