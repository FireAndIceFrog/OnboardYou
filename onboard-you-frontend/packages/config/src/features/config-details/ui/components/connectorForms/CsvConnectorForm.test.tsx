import { describe, it, expect, vi } from 'vitest';
import { screen, fireEvent } from '@testing-library/react';
import { renderWithProviders } from '@/shared/test/testWrapper';
import { CsvConnectorForm } from './CsvConnectorForm';
import type { ConnectorFormProps } from './types';

function baseProps(overrides: Partial<ConnectorFormProps> = {}): ConnectorFormProps {
  return {
    form: {
      system: 'csv' as any,
      displayName: 'Test',
      workday: { tenantUrl: '', tenantId: '', username: '', password: '', workerCountLimit: '200', responseGroup: '' },
      sageHr: { subdomain: '', apiToken: '', includeTeamHistory: false, includeEmploymentStatusHistory: false, includePositionHistory: false },
      csv: { filename: '', columns: [], uploadStatus: 'idle' as const, uploadError: null },
      genericIngestion: { filename: '', columns: [], uploadStatus: 'idle' as const, uploadError: null, conversionStatus: null },
    },
    errors: {},
    config: { getActionConfig: vi.fn(), getDefaultState: vi.fn(), applyChange: vi.fn() as any, validate: vi.fn(() => ({})), isFormValid: vi.fn(() => false) },
    onChange: vi.fn(),
    validateField: vi.fn(),
    ...overrides,
  };
}

describe('CsvConnectorForm', () => {
  it('renders choose file button', () => {
    renderWithProviders(<CsvConnectorForm {...baseProps()} />);
    expect(screen.getByRole('button', { name: /Choose CSV file/i })).toBeInTheDocument();
  });

  it('emits file event when file is selected', () => {
    const onChange = vi.fn();
    renderWithProviders(<CsvConnectorForm {...baseProps({ onChange })} />);
    const input = document.querySelector('input[type="file"]') as HTMLInputElement;
    const file = new File(['a,b'], 'test.csv', { type: 'text/csv' });
    fireEvent.change(input, { target: { files: [file] } });
    expect(onChange).toHaveBeenCalledWith({ type: 'file', file });
  });

  it('disables button while uploading', () => {
    const props = baseProps();
    props.form.csv.uploadStatus = 'uploading';
    renderWithProviders(<CsvConnectorForm {...props} />);
    const btn = screen.getByRole('button');
    expect(btn).toBeDisabled();
  });

  it('shows filename when file is set', () => {
    const props = baseProps();
    props.form.csv.filename = 'data.csv';
    renderWithProviders(<CsvConnectorForm {...props} />);
    expect(screen.getByText('data.csv')).toBeInTheDocument();
  });

  it('shows discovered columns when available', () => {
    const props = baseProps();
    props.form.csv = { filename: 'data.csv', columns: ['name', 'email'], uploadStatus: 'done', uploadError: null };
    renderWithProviders(<CsvConnectorForm {...props} />);
    expect(screen.getByText('name')).toBeInTheDocument();
    expect(screen.getByText('email')).toBeInTheDocument();
  });

  it('shows error when present', () => {
    const props = baseProps({ errors: { 'csv.filename': 'File too large' } });
    renderWithProviders(<CsvConnectorForm {...props} />);
    expect(screen.getByText('File too large')).toBeInTheDocument();
  });
});
