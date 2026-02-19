import { describe, it, expect, vi, beforeEach } from 'vitest';
import { screen } from '@testing-library/react';
import { renderWithProviders } from '@/shared/test/testWrapper';

// Mock useConnectionForm to avoid needing the full state/service chain
vi.mock('../../../state/useConnectionForm', () => ({
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
      csv: { filename: '', columns: [], uploadStatus: 'idle', uploadError: null },
    },
    errors: {},
    isValid: false,
    activeGroups: new Set<string>(),
    handleSystemSelect: vi.fn(),
    handleChange: () => vi.fn(),
    handleWorkdayChange: () => vi.fn(),
    handleCsvFileSelect: vi.fn(),
    handleResponseGroupToggle: vi.fn(),
    handleNext: vi.fn(),
    handleBack: vi.fn(),
    validateField: vi.fn(),
  }),
}));

describe('ConnectionDetailsScreen', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders the form with system selector', async () => {
    const { ConnectionDetailsScreen } = await import('../ConnectionDetailsScreen');
    renderWithProviders(<ConnectionDetailsScreen />);
    // Should show the two HR system options
    expect(screen.getByText('Workday')).toBeInTheDocument();
    expect(screen.getByText('CSV File Upload')).toBeInTheDocument();
  });

  it('renders back and next buttons', async () => {
    const { ConnectionDetailsScreen } = await import('../ConnectionDetailsScreen');
    renderWithProviders(<ConnectionDetailsScreen />);
    expect(screen.getByRole('button', { name: /back/i })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /next/i })).toBeInTheDocument();
  });

  it('disables next button when form is invalid', async () => {
    const { ConnectionDetailsScreen } = await import('../ConnectionDetailsScreen');
    renderWithProviders(<ConnectionDetailsScreen />);
    const nextBtn = screen.getByRole('button', { name: /next/i });
    expect(nextBtn).toBeDisabled();
  });
});
