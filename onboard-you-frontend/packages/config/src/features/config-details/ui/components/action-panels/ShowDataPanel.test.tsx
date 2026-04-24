import { describe, it, expect, vi, beforeEach, type Mock } from 'vitest';
import { screen, waitFor } from '@testing-library/react';
import { renderWithProviders } from '@/shared/test/testWrapper';
import { ShowDataPanel } from './ShowDataPanel';
import type { ShowDataResponse } from '@/generated/api';
import type { ConfigDetailsState } from '../../../domain/types';

/* ── Module mocks ────────────────────────────────────────── */

vi.mock('react-router-dom', () => ({
  useParams: () => ({ customerCompanyId: 'test-company' }),
}));

vi.mock('@/generated/api', () => ({
  getShowData: vi.fn(),
}));

import { getShowData } from '@/generated/api';
// eslint-disable-next-line @typescript-eslint/no-explicit-any
const mockGetShowData = vi.mocked(getShowData) as unknown as Mock<() => Promise<any>>;

/* ── Helpers ─────────────────────────────────────────────── */

const SAMPLE_DATA: ShowDataResponse = {
  columns: ['id', 'name', 'email'],
  rows: [
    { id: '1', name: 'Alice', email: 'alice@example.com' },
    { id: '2', name: 'Bob', email: 'bob@example.com' },
  ],
};

function makeState(overrides: Partial<ConfigDetailsState> = {}) {
  return {
    configDetails: {
      config: null,
      nodes: [],
      edges: [],
      selectedNode: {
        id: 'step-snapshot',
        position: { x: 0, y: 0 },
        data: {
          actionId: 'step-snapshot',
          actionType: 'show_data',
          category: 'egress',
          label: 'Show Data',
          config: {},
        },
      },
      isLoading: false,
      isSaving: false,
      isDeleting: false,
      isValidating: false,
      error: null,
      addStepPanelOpen: false,
      insertIndex: null,
      validationResult: null,
      validationErrors: {},
      ...overrides,
    } as unknown as ConfigDetailsState,
  };
}

function renderPanel(preloadedState = makeState()) {
  return renderWithProviders(
    <ShowDataPanel config={{}} onChange={() => {}} availableColumns={[]} />,
    { preloadedState },
  );
}

/* ── Declarative Cases ───────────────────────────────────── */

type MockData = ShowDataResponse | undefined | 'error';

interface DataStateCase {
  name: string;
  mockData: MockData;
  expectedText: RegExp;
}

const dataStateCases: DataStateCase[] = [
  {
    name: 'shows column headers when data is returned',
    mockData: SAMPLE_DATA,
    expectedText: /^email$/i,
  },
  {
    name: 'shows row cell values when data is returned',
    mockData: SAMPLE_DATA,
    expectedText: /^Alice$/,
  },
  {
    name: 'shows no-output alert when API returns no data property',
    mockData: undefined,
    expectedText: /no output yet/i,
  },
  {
    name: 'shows no-output alert on fetch error',
    mockData: 'error',
    expectedText: /no output yet/i,
  },
  {
    name: 'shows empty-data message when rows array is empty',
    mockData: { columns: ['id'], rows: [] },
    expectedText: /empty/i,
  },
];

/* ── Tests ───────────────────────────────────────────────── */

describe('ShowDataPanel', () => {
  beforeEach(() => vi.clearAllMocks());

  it('renders the panel root container immediately', () => {
    mockGetShowData.mockReturnValue(new Promise(() => {})); // never resolves
    renderPanel();
    expect(screen.getByTestId('show-data-panel')).toBeInTheDocument();
  });

  it('renders the Data Preview heading immediately', () => {
    mockGetShowData.mockReturnValue(new Promise(() => {}));
    renderPanel();
    expect(screen.getByText(/data preview/i)).toBeInTheDocument();
  });

  it('does not call the API when no node is selected', () => {
    mockGetShowData.mockReturnValue(new Promise(() => {}));
    renderPanel(makeState({ selectedNode: null }));
    expect(mockGetShowData).not.toHaveBeenCalled();
  });

  it.each(dataStateCases)('$name', async ({ mockData, expectedText }) => {
    if (mockData === 'error') {
      mockGetShowData.mockRejectedValue(new Error('fetch failed'));
    } else if (mockData === undefined) {
      mockGetShowData.mockResolvedValue({});
    } else {
      mockGetShowData.mockResolvedValue({ data: mockData });
    }

    renderPanel();

    await waitFor(() =>
      expect(screen.getByText(expectedText)).toBeInTheDocument(),
    );
  });

  it('shows truncation notice and limits rows to 50 when response has more', async () => {
    const manyRows = Array.from({ length: 60 }, (_, i) => ({ id: String(i + 1) }));
    mockGetShowData.mockResolvedValue({
      data: { columns: ['id'], rows: manyRows },
    });

    renderPanel();

    await waitFor(() =>
      expect(screen.getByText(/60/)).toBeInTheDocument(),
    );
    // Only 50 data rows rendered (header row + 50 body rows = 51 total)
    expect(screen.getAllByRole('row')).toHaveLength(51);
  });
});
