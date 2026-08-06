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
      primary0: '#e9f2f7',
      primary50: '#e5eef4',
      primary100: '#d8e8ef',
      primary200: '#c2dce7',
      primary300: '#a9cfe0',
      primary400: '#88c0d0',
      primary500: '#5e81ac',
      primary600: '#4c6692',
      primary700: '#3b536e',
      primary800: '#2e4256',
      primary900: '#243544',

      secondary0: '#f1f3f7',
      secondary50: '#eceff4',
      secondary100: '#e0e5ec',
      secondary200: '#d8dee9',
      secondary300: '#cfd6e3',
      secondary400: '#b7c2d5',
      secondary500: '#4c566a',
      secondary600: '#3b4252',
      secondary700: '#2e3440',
      secondary800: '#272f3a',
      secondary900: '#1f2630',

      success500: '#8fbcbb',
      danger500: '#bf616a',
      info500: '#5e81ac',
      warning500: '#d08770',

      backgroundLight: '#e5e9f0',
      backgroundDark: '#2e3440',

      cardBg: '#eceff4',
      borderColor: 'rgba(76, 86, 106, 0.22)',
      separatorColor: 'rgba(76, 86, 106, 0.14)',
      inputBg: '#ffffff',
      sidebarBg: '#eceff4',
      textLight: '#2e3440',
      textLight900: '#2e3440',
      textLight800: '#3b4252',
      textLight700: '#4c566a',
      textLight500: '#6b7280',
      textLightMuted: '#6b7280',

      cardBgDark: '#3b4252',
      borderColorDark: 'rgba(180, 200, 240, 0.2)',
      inputBgDark: '#3b4252',
      sidebarBgDark: '#2e3440',
      textDark: '#e5e9f0',
      textDark900: '#e5e9f0',
      textDark800: '#d8dee9',
      textDark700: '#cfd6e3',
      textDark600: '#c3ccd8',
      textDark500: '#b7c2d5',
      textDarkMuted: '#a7b2c4',

      themeSurfaceBg: '#eceff4',
      themeSurfaceBgDark: '#3b4252',
      themeInputBg: '#ffffff',
      themeInputBgDark: '#3b4252',
      themeBorder: 'rgba(76, 86, 106, 0.22)',
      themeBorderDim: 'rgba(76, 86, 106, 0.14)',
      themeBorderDark: 'rgba(180, 200, 240, 0.2)',

      navHeaderTint: '#5e81ac',
      navHeaderTintDark: '#88c0d0',
      navHeaderText: '#2e3440',
      navHeaderTextDark: '#e5e9f0',
      navDrawerBg: '#eceff4',
      navDrawerBgDark: '#2e3440',
      backdropColor: 'rgba(0, 0, 0, 0.35)',

      buttonBg: '#5e81ac',
      buttonBgDark: '#88c0d0',
      buttonBorder: '#4c6692',
      buttonBorderDark: '#5e81ac',
      buttonTextLight: '#ffffff',
      buttonTextDark: '#ffffff',

      activeHighlight: 'rgba(94, 129, 172, 0.14)',
      activeHighlightDark: 'rgba(136, 192, 208, 0.18)',
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
