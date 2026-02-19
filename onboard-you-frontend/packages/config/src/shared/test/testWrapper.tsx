import { type ReactElement, type ReactNode } from 'react';
import { render, type RenderOptions } from '@testing-library/react';
import { Provider } from 'react-redux';
import { configureStore, type EnhancedStore } from '@reduxjs/toolkit';
import { ChakraProvider } from '@chakra-ui/react';
import { I18nextProvider } from 'react-i18next';
import { type Mock, vi } from 'vitest';
import i18n from '@/i18n';
import { system } from '@/theme';
import type { RootState } from '@/store';
import chatReducer from '@/features/chat/state/chatSlice';
import configDetailsReducer from '@/features/config-details/state/configDetailsSlice';
import configListReducer from '@/features/config-list/state/configListSlice';

/* ── Mock thunk extra (mirrors the real ThunkExtra shape) ── */
export const mockShowNotification: Mock = vi.fn();

const mockThunkExtra = {
  showNotification: mockShowNotification,
};

/* ── Test store factory ────────────────────────────────────── */

export function createTestStore(preloadedState?: Partial<RootState>): EnhancedStore {
  return configureStore({
    reducer: {
      chat: chatReducer,
      configDetails: configDetailsReducer,
      configList: configListReducer,
    },
    preloadedState: preloadedState as RootState | undefined,
    middleware: (getDefaultMiddleware) =>
      getDefaultMiddleware({
        thunk: { extraArgument: mockThunkExtra },
      }),
  });
}

/* ── AllProviders wrapper ──────────────────────────────────── */

interface AllProvidersProps {
  children: ReactNode;
  store?: EnhancedStore;
}

/**
 * Wraps children with every provider the config package components
 * expect at runtime: Chakra, i18n, and Redux.
 *
 * Usage in component tests:
 * ```tsx
 * const { store } = renderWithProviders(<MyComponent />, {
 *   preloadedState: { configDetails: { ...overrides } },
 * });
 * ```
 *
 * Usage as a standalone wrapper (e.g. for hooks):
 * ```tsx
 * render(<AllProviders store={myStore}><Hook /></AllProviders>);
 * ```
 */
export function AllProviders({ children, store }: AllProvidersProps) {
  const testStore = store ?? createTestStore();
  return (
    <ChakraProvider value={system}>
      <I18nextProvider i18n={i18n}>
        <Provider store={testStore}>{children}</Provider>
      </I18nextProvider>
    </ChakraProvider>
  );
}

/* ── renderWithProviders helper ────────────────────────────── */

interface ExtendedRenderOptions extends Omit<RenderOptions, 'queries'> {
  preloadedState?: Partial<RootState>;
  store?: EnhancedStore;
}

export function renderWithProviders(
  ui: ReactElement,
  { preloadedState, store, ...renderOptions }: ExtendedRenderOptions = {},
): { store: EnhancedStore } & ReturnType<typeof render> {
  const testStore = store ?? createTestStore(preloadedState);

  const renderResult = render(ui, {
    wrapper: ({ children }: { children: ReactNode }) => (
      <AllProviders store={testStore}>{children}</AllProviders>
    ),
    ...renderOptions,
  });

  return {
    store: testStore,
    ...renderResult,
  } as { store: EnhancedStore } & ReturnType<typeof render>;
}
