import { describe, it, expect, vi, beforeEach } from 'vitest';
import { screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { renderWithProviders } from '@/shared/test/testWrapper';
import { RunDetailsPanel } from './RunDetailsPanel';
import type { PipelineRun } from '@/generated/api';

const baseRun: PipelineRun = {
  id: 'run-1',
  organizationId: 'org-1',
  customerCompanyId: 'test-company',
  status: 'completed',
  startedAt: '2026-03-10T10:00:00Z',
  finishedAt: '2026-03-10T10:05:00Z',
  rowsProcessed: 42,
  warnings: [
    { action_id: 'cellphone_sanitizer', message: 'Invalid phone format', count: 3, detail: '+1-ABC' },
    { action_id: 'cellphone_sanitizer', message: 'Missing country code', count: 1 },
    { action_id: 'pii_masking', message: 'Empty value skipped', count: 5 },
  ],
};

const failedRun: PipelineRun = {
  ...baseRun,
  id: 'run-2',
  status: 'failed',
  errorMessage: 'Column "email" not found in input',
  errorActionId: 'api_dispatcher',
  errorRow: 15,
};

describe('RunDetailsPanel', () => {
  const onClose = vi.fn();

  beforeEach(() => vi.clearAllMocks());

  it('renders run details', () => {
    renderWithProviders(<RunDetailsPanel run={baseRun} onClose={onClose} />);
    expect(screen.getByTestId('run-details-panel')).toBeInTheDocument();
    expect(screen.getByText('completed')).toBeInTheDocument();
    expect(screen.getByText('42')).toBeInTheDocument();
  });

  it('groups warnings by action', () => {
    renderWithProviders(<RunDetailsPanel run={baseRun} onClose={onClose} />);
    // cellphone_sanitizer → "Clean Phone Numbers"
    expect(screen.getByText('Clean Phone Numbers')).toBeInTheDocument();
    // pii_masking → "Mask Sensitive Data"
    expect(screen.getByText('Mask Sensitive Data')).toBeInTheDocument();
  });

  it('shows warning messages', () => {
    renderWithProviders(<RunDetailsPanel run={baseRun} onClose={onClose} />);
    expect(screen.getByText('Invalid phone format')).toBeInTheDocument();
    expect(screen.getByText('Missing country code')).toBeInTheDocument();
    expect(screen.getByText('Empty value skipped')).toBeInTheDocument();
  });

  it('shows error section for failed runs', () => {
    renderWithProviders(<RunDetailsPanel run={failedRun} onClose={onClose} />);
    expect(screen.getByTestId('run-error-section')).toBeInTheDocument();
    expect(screen.getByText(/Column "email" not found/)).toBeInTheDocument();
    expect(screen.getByText(/Send to API/)).toBeInTheDocument();
    expect(screen.getByText(/row 15/)).toBeInTheDocument();
  });

  it('does not show error section for successful runs', () => {
    renderWithProviders(<RunDetailsPanel run={baseRun} onClose={onClose} />);
    expect(screen.queryByTestId('run-error-section')).not.toBeInTheDocument();
  });

  it('calls onClose when close button clicked', async () => {
    renderWithProviders(<RunDetailsPanel run={baseRun} onClose={onClose} />);
    await userEvent.click(screen.getByTestId('close-details'));
    expect(onClose).toHaveBeenCalledTimes(1);
  });

  it('shows no warnings message when empty', () => {
    renderWithProviders(
      <RunDetailsPanel run={{ ...baseRun, warnings: [] }} onClose={onClose} />,
    );
    expect(screen.getByText(/no warnings/i)).toBeInTheDocument();
  });

  it('has warning search input', () => {
    renderWithProviders(<RunDetailsPanel run={baseRun} onClose={onClose} />);
    expect(screen.getByTestId('warning-search')).toBeInTheDocument();
  });

  it('has resize handle', () => {
    renderWithProviders(<RunDetailsPanel run={baseRun} onClose={onClose} />);
    expect(screen.getByTestId('resize-handle')).toBeInTheDocument();
  });
});
