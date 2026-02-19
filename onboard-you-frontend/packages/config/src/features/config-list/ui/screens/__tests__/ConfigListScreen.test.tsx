import { describe, it, expect, vi, beforeEach } from 'vitest';
import { screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { renderWithProviders } from '@/shared/test/testWrapper';
import type { ConfigListState } from '../../../domain/types';
import type { PipelineConfig } from '@/shared/domain/types';

// Mock react-router-dom
const mockNavigate = vi.fn();
vi.mock('react-router-dom', async () => {
  const actual = await vi.importActual<typeof import('react-router-dom')>('react-router-dom');
  return { ...actual, useNavigate: () => mockNavigate };
});

// Mock fetchConfigs to prevent the useEffect from firing real API calls
vi.mock('../../../state/configListSlice', async () => {
  const actual = await vi.importActual<Record<string, unknown>>('../../../state/configListSlice');
  return {
    ...actual,
    fetchConfigs: () => ({ type: 'configList/fetchConfigs/noop' }),
  };
});

function makeConfig(overrides: Partial<PipelineConfig> = {}): PipelineConfig {
  return {
    customerCompanyId: 'acme-corp',
    name: 'Acme Integration',
    cron: 'rate(1 day)',
    organizationId: 'org-1',
    pipeline: { version: '1.0', actions: [{ id: 'a1', action_type: 'rename_column' as any, config: { mappings: {} } as any }] },
    lastEdited: new Date(Date.now() - 3_600_000 * 2).toISOString(),
    ...overrides,
  };
}

function makeState(overrides: Partial<ConfigListState> = {}) {
  return {
    configList: {
      configs: [],
      isLoading: false,
      error: null,
      searchQuery: '',
      ...overrides,
    } as ConfigListState,
  };
}

describe('ConfigListScreen', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('shows loading spinner when isLoading is true', async () => {
    const { ConfigListScreen } = await import('../ConfigListScreen');
    renderWithProviders(<ConfigListScreen />, {
      preloadedState: makeState({ isLoading: true }),
    });
    expect(screen.getByTestId('config-list-loading')).toBeInTheDocument();
  });

  it('shows error message when error is set', async () => {
    const { ConfigListScreen } = await import('../ConfigListScreen');
    renderWithProviders(<ConfigListScreen />, {
      preloadedState: makeState({ error: 'Network error' }),
    });
    expect(screen.getByTestId('config-list-error')).toBeInTheDocument();
    expect(screen.getByText('Network error')).toBeInTheDocument();
  });

  it('shows empty state when no configs match', async () => {
    const { ConfigListScreen } = await import('../ConfigListScreen');
    renderWithProviders(<ConfigListScreen />, {
      preloadedState: makeState({ configs: [] }),
    });
    expect(screen.getByTestId('config-list-empty')).toBeInTheDocument();
  });

  it('renders config items in a grid', async () => {
    const { ConfigListScreen } = await import('../ConfigListScreen');
    renderWithProviders(<ConfigListScreen />, {
      preloadedState: makeState({
        configs: [
          makeConfig({ customerCompanyId: 'alpha', name: 'Alpha Corp' }),
          makeConfig({ customerCompanyId: 'beta', name: 'Beta Inc' }),
        ],
      }),
    });
    expect(screen.getByTestId('config-list-grid')).toBeInTheDocument();
    expect(screen.getByText('Alpha Corp')).toBeInTheDocument();
    expect(screen.getByText('Beta Inc')).toBeInTheDocument();
  });

  it('filters configs based on search query', async () => {
    const { ConfigListScreen } = await import('../ConfigListScreen');
    renderWithProviders(<ConfigListScreen />, {
      preloadedState: makeState({
        configs: [
          makeConfig({ customerCompanyId: 'alpha', name: 'Alpha Corp' }),
          makeConfig({ customerCompanyId: 'beta', name: 'Beta Inc' }),
        ],
        searchQuery: 'alpha',
      }),
    });
    expect(screen.getByText('Alpha Corp')).toBeInTheDocument();
    expect(screen.queryByText('Beta Inc')).not.toBeInTheDocument();
  });
});
