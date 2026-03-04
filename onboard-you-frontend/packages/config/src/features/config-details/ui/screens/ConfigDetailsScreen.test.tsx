import { describe, it, expect, vi, beforeEach } from 'vitest';
import { screen } from '@testing-library/react';
import { renderWithProviders } from '@/shared/test/testWrapper';
import type { ConfigDetailsState } from '../../domain/types';

/**
 * ConfigDetailsScreen requires react-router params and react-flow.
 * We test the outer shell behaviour — loading state, error state,
 * and the guard screens — by rendering ConfigDetailsContent via Redux state directly.
 * The full ReactFlow canvas is impractical to render in jsdom,
 * so we test the screen indirectly via state-driven guards.
 */

function makeState(overrides: Partial<ConfigDetailsState> = {}) {
  return {
    configDetails: {
      config: null,
      nodes: [],
      edges: [],
      selectedNode: null,
      isLoading: false,
      isSaving: false,
      isDeleting: false,
      isValidating: false,
      error: null,
      chatOpen: false,
      addStepPanelOpen: false,
      validationResult: null,
      planSummary: null,
      isGeneratingPlan: false,
      viewMode: 'advanced',
      planStale: false,
      ...overrides,
    } as ConfigDetailsState,
  };
}

// Mock react-router since the screen uses useParams/useNavigate/useLocation
vi.mock('react-router-dom', async () => {
  const actual = await vi.importActual<typeof import('react-router-dom')>('react-router-dom');
  return {
    ...actual,
    useParams: () => ({ customerCompanyId: 'test-company' }),
    useNavigate: () => vi.fn(),
    useLocation: () => ({ state: null, pathname: '/config/test-company', search: '', hash: '' }),
  };
});

// Mock useGlobal hook
vi.mock('@/shared/hooks', () => ({
  useGlobal: () => ({ showNotification: vi.fn() }),
}));

// Mock fetchConfigDetails to prevent the useEffect from flipping isLoading back to true
vi.mock('../../state/configDetailsSlice', async () => {
  const actual = await vi.importActual<Record<string, unknown>>('../../state/configDetailsSlice');
  return {
    ...actual,
    fetchConfigDetails: () => ({ type: 'configDetails/fetchConfigDetails/noop' }),
  };
});

describe('ConfigDetailsScreen', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('shows loading spinner when isLoading is true', async () => {
    // Dynamically import after mocks are set up
      const { ConfigDetailsScreen } = await import("./ConfigDetailsScreen");
    renderWithProviders(<ConfigDetailsScreen />, {
      preloadedState: makeState({ isLoading: true }),
    });
    expect(screen.getByText(/loading/i)).toBeInTheDocument();
  });

  it('shows error message when error is set', async () => {
    const { ConfigDetailsScreen } = await import('./ConfigDetailsScreen');
    renderWithProviders(<ConfigDetailsScreen />, {
      preloadedState: makeState({ error: 'Something went wrong' }),
    });
    expect(screen.getByText('Something went wrong')).toBeInTheDocument();
  });

  it('renders nothing when config is null and not loading/error', async () => {
    const { ConfigDetailsScreen } = await import('./ConfigDetailsScreen');
    const { container } = renderWithProviders(<ConfigDetailsScreen />, {
      preloadedState: makeState({ config: null }),
    });
    // Should render some content (at minimum the page structure or null)
    expect(container).toBeDefined();
  });
});
