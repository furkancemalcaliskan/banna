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
            // Primary Palette (Blue)
            primary0: '#e0edff',
            primary50: '#eff6ff',
            primary100: '#dbeafe',
            primary200: '#bfdbfe',
            primary300: '#93c5fd',
            primary400: '#60a5fa',
            primary500: '#3b82f6', // modern.css dark --bs-primary
            primary600: '#2b78ff', // modern.css light --bs-primary
            primary700: '#1d4ed8',
            primary800: '#1e40af',
            primary900: '#1e3a8a',

            // Secondary Palette (Gray/Slate)
            secondary0: '#f8f9fa',
            secondary50: '#f8fafc',
            secondary100: '#f1f5f9',
            secondary200: '#e2e8f0',
            secondary300: '#cbd5e1',
            secondary400: '#94a3b8',
            secondary500: '#64748b',
            secondary600: '#475569',
            secondary700: '#334155',
            secondary800: '#1e293b',
            secondary900: '#0f172a',

            // Functional Colors
            success500: '#22c55e',
            danger500: '#ef4444',
            info500: '#0ea5e9',
            warning500: '#eab308',

            // Backgrounds
            backgroundLight: '#f8fafc',
            backgroundDark: '#0f172a',

            // Semantic Theme Tokens - Light (flat)
            cardBg: '#ffffff',
            borderColor: 'rgba(120, 140, 170, 0.25)',
            separatorColor: 'rgba(120, 140, 170, 0.15)',
            inputBg: '#ffffff',
            sidebarBg: '#ffffff',
            textLight: '#0b1220',
            textLight900: '#0b1220',
            textLight800: '#1f2937',
            textLight700: '#5f6b7a',
            textLight500: '#94a3b8',
            textLightMuted: '#7a8698',

            // Semantic Theme Tokens - Dark (flat)
            cardBgDark: '#1e293b',
            borderColorDark: 'rgba(180, 200, 240, 0.2)',
            inputBgDark: '#1e293b',
            sidebarBgDark: '#0f172a',
            textDark: '#e8eefc',
            textDark900: '#e8eefc',
            textDark800: '#d9e4f8',
            textDark700: '#c5d3f4',
            textDark600: '#b2c4ec',
            textDark500: '#a5b7e6',
            textDarkMuted: '#9fb2d0',

            // Standard Surface & Border tokens (Generic names)
            themeSurfaceBg: '#ffffff',
            themeSurfaceBgDark: '#1e293b',
            themeInputBg: '#ffffff',
            themeInputBgDark: '#1e293b',
            themeBorder: 'rgba(120, 140, 170, 0.25)',
            themeBorderDim: 'rgba(120, 140, 170, 0.15)',
            themeBorderDark: 'rgba(180, 200, 240, 0.2)',

            // Navigation specific tokens
            navHeaderTint: '#2b78ff',
            navHeaderTintDark: '#3b82f6',
            navHeaderText: '#0b1220',
            navHeaderTextDark: '#e8eefc',
            navDrawerBg: '#ffffff',
            navDrawerBgDark: '#0f172a',
            backdropColor: 'rgba(0, 0, 0, 0.3)',

            buttonBg: '#2b78ff',
            buttonBgDark: '#3b82f6',
            buttonBorder: '#1f5fe0',
            buttonBorderDark: '#2563eb',
            buttonTextLight: '#ffffff',
            buttonTextDark: '#ffffff',

            // Active/highlight state tokens
            activeHighlight: 'rgba(43, 120, 255, 0.12)',
            activeHighlightDark: 'rgba(43, 120, 255, 0.22)',
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
                variants: {
                    ...defaultConfig.components?.Input?.theme?.variants,
                    outline: {
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
        SelectTrigger: {
            theme: {
                ...(defaultConfig.components?.SelectTrigger?.theme || {}),
                borderWidth: 1,
                borderRadius: '$lg',
                _dark: {
                    bg: '$themeInputBgDark',
                    borderColor: '$themeBorderDark',
                },
                _light: {
                    bg: '$themeInputBg',
                    borderColor: '$themeBorder',
                },
                variants: {
                    ...defaultConfig.components?.SelectTrigger?.theme?.variants,
                    outline: {
                        bg: '$themeInputBg',
                        borderColor: '$themeBorder',
                        _dark: {
                            bg: '$themeInputBgDark',
                            borderColor: '$themeBorderDark',
                        },
                    },
                },
            },
        },
        SelectInput: {
            theme: {
                ...(defaultConfig.components?.SelectInput?.theme || {}),
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
        SelectContent: {
            theme: {
                ...(defaultConfig.components?.SelectContent?.theme || {}),
                _light: {
                    bg: '$cardBg',
                },
                _dark: {
                    bg: '$cardBgDark',
                },
            },
        },
        SelectItemText: {
            theme: {
                ...(defaultConfig.components?.SelectItemText?.theme || {}),
                _light: { color: '$textLight900' },
                _dark: { color: '$textDark900' },
            },
        },
        Textarea: {
            theme: {
                ...(defaultConfig.components?.Textarea?.theme || {}),
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
        TextareaInput: {
            theme: {
                ...(defaultConfig.components?.TextareaInput?.theme || {}),
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
        FormControlLabelText: {
            theme: {
                ...(defaultConfig.components?.FormControlLabelText?.theme || {}),
                _light: { color: '$textLight800' },
                _dark: { color: '$textDark800' },
            },
        },
        Checkbox: {
            theme: {
                ...(defaultConfig.components?.Checkbox?.theme || {}),
                borderWidth: 1,
                borderRadius: '$md',
                _light: { borderColor: '$themeBorder' },
                _dark: { borderColor: '$themeBorderDark' },
                _checked: {
                    _light: { bg: '$primary500', borderColor: '$primary500' },
                    _dark: { bg: '$primary400', borderColor: '$primary400' },
                },
            },
        },
        Radio: {
            theme: {
                ...(defaultConfig.components?.Radio?.theme || {}),
                borderWidth: 1,
                _light: { borderColor: '$themeBorder' },
                _dark: { borderColor: '$themeBorderDark' },
                _checked: {
                    _light: { borderColor: '$primary500' },
                    _dark: { borderColor: '$primary400' },
                },
            },
        },
        Switch: {
            theme: {
                ...(defaultConfig.components?.Switch?.theme || {}),
                _light: {
                    trackColor: '$secondary200',
                    _checked: { trackColor: '$primary500' },
                },
                _dark: {
                    trackColor: '$secondary800',
                    _checked: { trackColor: '$primary400' },
                },
            },
        },
        Button: {
            theme: {
                ...(defaultConfig.components?.Button?.theme || {}),
                borderWidth: 1,
                borderRadius: '$full',
                minHeight: 44,
                justifyContent: 'center',
                alignItems: 'center',
                flexDirection: 'row',
                px: '$4',
                py: '$2', // Daha az padding
                _text: { // Button içindeki Text için özel stil
                    textAlign: 'center',
                    textAlignVertical: 'center',
                },
                _light: {
                    bg: '$buttonBg',
                    borderColor: '$buttonBorder',
                },
                _dark: {
                    bg: '$buttonBgDark',
                    borderColor: '$buttonBorderDark',
                },
                _hover: {
                    bg: '$primary600',
                    _dark: { bg: '$primary500' },
                },
                _pressed: {
                    bg: '$primary700',
                    _dark: { bg: '$primary600' },
                },
            },
        },
        ButtonText: {
            theme: {
                ...(defaultConfig.components?.ButtonText?.theme || {}),
                fontWeight: '$semibold',
                fontSize: scaleFont(14),
                lineHeight: scaleFont(18), // Daha yakın font-size/line-height oranı
                includeFontPadding: false, // Font padding'ini kapat
                textAlign: 'center', // Text'i ortala
                textAlignVertical: 'center', // Android için dikey ortalama
                _light: {
                    color: '$buttonTextLight',
                },
                _dark: {
                    color: '$buttonTextDark',
                },
            },
        },
        Text: {
            theme: {
                ...(defaultConfig.components?.Text?.theme || {}),
                _light: {
                    color: '$textLight900',
                },
                _dark: {
                    color: '$textDark900',
                },
            },
        },
        Heading: {
            theme: {
                ...(defaultConfig.components?.Heading?.theme || {}),
                _light: {
                    color: '$textLight900',
                },
                _dark: {
                    color: '$textDark900',
                },
            },
        },
    },
    globalStyle: {
        fontFamily: 'System',
    },
});

export default config;
