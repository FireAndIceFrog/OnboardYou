import i18n from 'i18next';
import { initReactI18next } from 'react-i18next';
import en from './locales/en.json';

// initialize once; merge if someone else (remote) reimports
if (!i18n.isInitialized) {
  i18n.use(initReactI18next).init({
    resources: { en: { translation: en } },
    lng: 'en',
    fallbackLng: 'en',
    interpolation: { escapeValue: false },
  });
} else {
  i18n.addResourceBundle('en', 'translation', en, true, false);
}

export default i18n;
