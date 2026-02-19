import { type ReactElement, type ReactNode } from 'react';
import { render, type RenderOptions } from '@testing-library/react';
import { Provider } from 'react-redux';
import { configureStore, type EnhancedStore } from '@reduxjs/toolkit';
import { ChakraProvider } from '@chakra-ui/react';
import { I18nextProvider } from 'react-i18next';
import { MemoryRouter } from 'react-router-dom';
import i18n from '@/i18n';
import { system } from '@/theme';
import type { RootState } from '@/store';
import authReducer from '@/features/auth/state/authSlice';
import layoutReducer from '@/features/layout/state/layoutSlice';
import globalReducer from '@/shared/state/globalSlice';
import settingsReducer from '@/features/settings/state/settingsSlice';

/* ── Test store factory ────────────────────────────────────── */

export function createTestStore(preloadedState?: Partial<RootState>): EnhancedStore {
  return configureStore({
    reducer: {
      auth: authReducer,
      layout: layoutReducer,
      global: globalReducer,
      settings: settingsReducer,
    },
    preloadedState: preloadedState as RootState | undefined,
  });
}

/* ── AllProviders wrapper ──────────────────────────────────── */

interface AllProvidersProps {
  children: ReactNode;
  store?: EnhancedStore;
  initialRoute?: string;
}

/**
 * Wraps children with every provider the platform package components
 * expect at runtime: Chakra, i18n, Redux, and React Router.
 */
export function AllProviders({ children, store, initialRoute = '/' }: AllProvidersProps) {
  const testStore = store ?? createTestStore();
  return (
    <ChakraProvider value={system}>
      <I18nextProvider i18n={i18n}>
        <Provider store={testStore}>
          <MemoryRouter initialEntries={[initialRoute]}>
            {children}
          </MemoryRouter>
        </Provider>
      </I18nextProvider>
    </ChakraProvider>
  );
}

/* ── renderWithProviders helper ────────────────────────────── */

interface ExtendedRenderOptions extends Omit<RenderOptions, 'queries'> {
  preloadedState?: Partial<RootState>;
  store?: EnhancedStore;
  initialRoute?: string;
}

export function renderWithProviders(
  ui: ReactElement,
  { preloadedState, store, initialRoute, ...renderOptions }: ExtendedRenderOptions = {},
): { store: EnhancedStore } & ReturnType<typeof render> {
  const testStore = store ?? createTestStore(preloadedState);

  const renderResult = render(ui, {
    wrapper: ({ children }: { children: ReactNode }) => (
      <AllProviders store={testStore} initialRoute={initialRoute}>
        {children}
      </AllProviders>
    ),
    ...renderOptions,
  });

  return {
    store: testStore,
    ...renderResult,
  } as { store: EnhancedStore } & ReturnType<typeof render>;
}
