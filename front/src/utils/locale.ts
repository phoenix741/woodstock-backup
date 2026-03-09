import { ref } from 'vue';

const DEFAULT_LOCALE = 'en-US';

function canonicalizeLocale(locale?: string | null) {
  if (!locale) {
    return DEFAULT_LOCALE;
  }

  try {
    return Intl.getCanonicalLocales(locale)[0] ?? DEFAULT_LOCALE;
  } catch {
    return DEFAULT_LOCALE;
  }
}

function getNavigatorLocales() {
  if (typeof navigator === 'undefined') {
    return [DEFAULT_LOCALE];
  }

  const locales = [...(navigator.languages ?? []), navigator.language].filter((value): value is string => !!value);
  const canonicalLocales = locales.map(canonicalizeLocale);

  return Array.from(new Set(canonicalLocales));
}

function browserLocaleToVuetifyLocale(locale: string) {
  const normalized = locale.toLowerCase();

  if (normalized.startsWith('sr-latn')) {
    return 'srLatn';
  }

  if (normalized.startsWith('sr-cyrl')) {
    return 'srCyrl';
  }

  const language = normalized.split('-')[0];
  return language || 'en';
}

function updateDocumentLocale(locale: string) {
  if (typeof document !== 'undefined') {
    document.documentElement.lang = locale;
  }
}

const initialLocale = getNavigatorLocales()[0] ?? DEFAULT_LOCALE;

export const appLocale = ref(initialLocale);

export function getAppLocale() {
  return appLocale.value;
}

export function getBrowserLocales() {
  return getNavigatorLocales();
}

export function setAppLocale(locale: string) {
  const nextLocale = canonicalizeLocale(locale);
  appLocale.value = nextLocale;
  updateDocumentLocale(nextLocale);
}

export function initializeAppLocale() {
  updateDocumentLocale(appLocale.value);
}

export function getVuetifyLocale() {
  return browserLocaleToVuetifyLocale(appLocale.value);
}

export function getVuetifyDateLocale() {
  return {
    en: 'en-US',
    [getVuetifyLocale()]: appLocale.value,
  };
}
