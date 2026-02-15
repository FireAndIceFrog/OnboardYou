import { useEffect } from 'react';
import { useAppSelector } from '@/store';
import { selectTheme } from '@/shared/state/globalSlice';

const THEME_STORAGE_KEY = 'onboardyou-theme';

/**
 * Syncs the Redux theme state to the DOM (`data-theme` attribute on `<html>`)
 * and persists the preference in localStorage.
 *
 * Must be rendered inside the Redux `<Provider>`.
 */
export function useThemeEffect() {
  const theme = useAppSelector(selectTheme);

  useEffect(() => {
    document.documentElement.setAttribute('data-theme', theme);
    localStorage.setItem(THEME_STORAGE_KEY, theme);
  }, [theme]);
}
