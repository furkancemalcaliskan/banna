// Centralized i18n-js instance that works for both CJS and ESM builds under Metro.
// eslint-disable-next-line @typescript-eslint/no-var-requires
const raw = require('i18n-js');

// Prefer default export if it already has translations/t; otherwise fallback to class constructor.
const base = raw?.default || raw;
let i18n = base?.t
  ? base
  : raw?.I18n
    ? new raw.I18n()
    : typeof base === 'function'
      ? new base()
      : base;

if (!i18n || typeof i18n.t !== 'function') {
  const fallback = raw?.I18n ? new raw.I18n() : {};
  i18n = Object.assign(fallback, i18n || {});
}

i18n.fallbacks = true;
i18n.defaultLocale = 'en';
i18n.translations = i18n.translations || {};

export default i18n;
