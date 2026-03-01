import i18n from 'i18next';
import { initReactI18next } from 'react-i18next';
import en from './locales/en.json';

// when running as a remote the host may already have initialized
// the i18next singleton.  avoid re-initializing and instead merge our
// resources so we don’t wipe out the host translations.
if (!i18n.isInitialized) {
  i18n.use(initReactI18next).init({
    resources: { en: { translation: en } },
    lng: 'en',
    fallbackLng: 'en',
    interpolation: { escapeValue: false },
  });
} else {
  // always merge in our translations without overwriting existing keys
  // deep=true ensures nested structures combine correctly
  i18n.addResourceBundle('en', 'translation', en, true, false);
}

export default i18n;
