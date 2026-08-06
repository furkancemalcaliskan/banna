const appResources: Record<string, any> = {
  ar: require('./MyProjectName/ar.json'),
  cs: require('./MyProjectName/cs.json'),
  de: require('./MyProjectName/de.json'),
  en: require('./MyProjectName/en.json'),
  'en-GB': require('./MyProjectName/en-GB.json'),
  es: require('./MyProjectName/es.json'),
  fi: require('./MyProjectName/fi.json'),
  fr: require('./MyProjectName/fr.json'),
  hi: require('./MyProjectName/hi.json'),
  hr: require('./MyProjectName/hr.json'),
  hu: require('./MyProjectName/hu.json'),
  is: require('./MyProjectName/is.json'),
  it: require('./MyProjectName/it.json'),
  nl: require('./MyProjectName/nl.json'),
  'pl-PL': require('./MyProjectName/pl-PL.json'),
  'pt-BR': require('./MyProjectName/pt-BR.json'),
  'ro-RO': require('./MyProjectName/ro-RO.json'),
  ru: require('./MyProjectName/ru.json'),
  sk: require('./MyProjectName/sk.json'),
  sl: require('./MyProjectName/sl.json'),
  sv: require('./MyProjectName/sv.json'),
  tr: require('./MyProjectName/tr.json'),
  vi: require('./MyProjectName/vi.json'),
  'zh-Hans': require('./MyProjectName/zh-Hans.json'),
  'zh-Hant': require('./MyProjectName/zh-Hant.json'),
};

const normalizeLocale = (locale: string | null | undefined) => (locale || '').replace('_', '-');

const availableLocales = Object.keys(appResources).map(normalizeLocale);

const localeDisplayNames: Record<string, string> = {
  ar: 'العربية',
  cs: 'Čeština',
  de: 'Deutsch',
  en: 'English',
  'en-GB': 'English (United Kingdom)',
  es: 'Español',
  fi: 'Suomi',
  fr: 'Français',
  hi: 'हिन्दी',
  hr: 'Hrvatski',
  hu: 'Magyar',
  is: 'Íslenska',
  it: 'Italiano',
  nl: 'Nederlands',
  'pl-PL': 'Polski',
  'pt-BR': 'Português (Brasil)',
  'ro-RO': 'Română',
  ru: 'Русский',
  sk: 'Slovenčina',
  sl: 'Slovenščina',
  sv: 'Svenska',
  tr: 'Türkçe',
  vi: 'Tiếng Việt',
  'zh-Hans': '简体中文',
  'zh-Hant': '繁體中文',
};

const getLocaleDisplayName = (locale: string) => {
  const normalized = normalizeLocale(locale);
  if (localeDisplayNames[normalized]) return localeDisplayNames[normalized];

  const base = normalized.split('-')[0];
  if (localeDisplayNames[base]) return localeDisplayNames[base];

  return normalized || 'en';
};

export const appLanguages = availableLocales.map((locale) => ({
  cultureName: locale,
  displayName: getLocaleDisplayName(locale),
}));

export const resolveLocale = (locale?: string | null) => {
  if (!locale) return 'en';
  const normalized = normalizeLocale(locale);
  if (appResources[normalized]) return normalized;
  const base = normalized.split('-')[0];
  const fallback = availableLocales.find((item) => item.split('-')[0] === base);
  return fallback || 'en';
};

export const registerTranslations = (i18nInstance: any) => {
  if (!i18nInstance) return;
  i18nInstance.translations = i18nInstance.translations || {};
  Object.entries(appResources).forEach(([locale, resource]) => {
    const cultureName = resolveLocale(locale);
    const existingLocale = i18nInstance.translations[cultureName] || {};
    i18nInstance.translations[cultureName] = {
      ...existingLocale,
      ...resource,
    };
  });
};

export const setLocale = (i18nInstance: any, locale?: string | null) => {
  if (!i18nInstance) return 'en';
  const resolved = resolveLocale(locale);
  i18nInstance.locale = resolved;
  return resolved;
};
