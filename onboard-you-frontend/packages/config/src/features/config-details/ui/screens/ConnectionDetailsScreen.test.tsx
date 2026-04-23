import { describe, it, expect, vi, beforeEach } from 'vitest';
import { screen } from '@testing-library/react';
import { renderWithProviders } from '@/shared/test/testWrapper';

// Mock useConnectionForm to avoid needing the full state/service chain
vi.mock('../../state/useConnectionForm', () => ({
  useConnectionForm: () => ({
    form: {
      system: '',
      displayName: '',
      workday: {
        tenantUrl: '',
        tenantId: '',
        username: '',
        password: '',
        workerCountLimit: '200',
        responseGroup: '',
      },
      sageHr: {
        subdomain: '',
        apiToken: '',
        includeTeamHistory: false,
        includeEmploymentStatusHistory: false,
        includePositionHistory: false,
      },
      genericIngestion: { filename: '', columns: [], uploadStatus: 'idle' as const, uploadError: null, conversionStatus: null },
    },
    errors: {},
    isValid: false,
    config: {
      getActionConfig: vi.fn(),
      getDefaultState: vi.fn(),
      applyChange: vi.fn(),
      validate: vi.fn(() => ({})),
      isFormValid: vi.fn(() => false),
    },
    handleSystemSelect: vi.fn(),
    handleChange: () => vi.fn(),
    handleConnectorChange: vi.fn(),
    handleNext: vi.fn(),
    handleBack: vi.fn(),
    validateField: vi.fn(),
  }),
}));

/* ── Declarative Cases ──────────────────────────────────── */

interface Case {
  name: string;
  assert: (screen: typeof import('@testing-library/react').screen) => void;
}

const cases: Case[] = [
  {
    name: 'renders all HR system options',
    assert: (s) => {
      expect(s.getByText('Workday')).toBeInTheDocument();
      expect(s.getByText('Sage HR')).toBeInTheDocument();
      expect(s.getByText('Generic File Upload')).toBeInTheDocument();
    },
  },
  {
    name: 'renders back button',
    assert: (s) => {
      expect(s.getByRole('button', { name: /back/i })).toBeInTheDocument();
    },
  },
  {
    name: 'renders next button',
    assert: (s) => {
      expect(s.getByRole('button', { name: /next/i })).toBeInTheDocument();
    },
  },
  {
    name: 'next button is disabled when form is invalid',
    assert: (s) => {
      expect(s.getByRole('button', { name: /next/i })).toBeDisabled();
    },
  },
];

/* ── Tests ──────────────────────────────────────────────── */

describe('ConnectionDetailsScreen', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it.each(cases)('$name', async ({ assert: assertFn }) => {
    const { ConnectionDetailsScreen } = await import('./ConnectionDetailsScreen');
    renderWithProviders(<ConnectionDetailsScreen />);
    assertFn(screen);
  });
});
