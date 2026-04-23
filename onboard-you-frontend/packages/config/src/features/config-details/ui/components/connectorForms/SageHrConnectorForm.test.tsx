import { describe, it, expect, vi } from 'vitest';
import { screen, fireEvent } from '@testing-library/react';
import { renderWithProviders } from '@/shared/test/testWrapper';
import { SageHrConnectorForm } from './SageHrConnectorForm';
import type { ConnectorFormProps } from './types';

function baseProps(overrides: Partial<ConnectorFormProps> = {}): ConnectorFormProps {
  return {
    form: {
      system: 'sage_hr' as any,
      displayName: 'Test',
      workday: { tenantUrl: '', tenantId: '', username: '', password: '', workerCountLimit: '200', responseGroup: '' },
      sageHr: { subdomain: '', apiToken: '', includeTeamHistory: false, includeEmploymentStatusHistory: false, includePositionHistory: false },
      genericIngestion: { filename: '', columns: [], uploadStatus: 'idle' as const, uploadError: null, conversionStatus: null },
    },
    errors: {},
    config: { getActionConfig: vi.fn(), getDefaultState: vi.fn(), applyChange: vi.fn() as any, validate: vi.fn(() => ({})), isFormValid: vi.fn(() => false) },
    onChange: vi.fn(),
    validateField: vi.fn(),
    ...overrides,
  };
}

describe('SageHrConnectorForm', () => {
  it('renders credential fields', () => {
    renderWithProviders(<SageHrConnectorForm {...baseProps()} />);
    expect(screen.getByLabelText(/Subdomain/i)).toBeInTheDocument();
    expect(screen.getByLabelText(/API Token/i)).toBeInTheDocument();
  });

  it('renders history toggle buttons', () => {
    renderWithProviders(<SageHrConnectorForm {...baseProps()} />);
    expect(screen.getByText('Team History')).toBeInTheDocument();
  });

  it('emits field change event on input', () => {
    const onChange = vi.fn();
    renderWithProviders(<SageHrConnectorForm {...baseProps({ onChange })} />);
    fireEvent.change(screen.getByLabelText(/Subdomain/i), { target: { value: 'acme' } });
    expect(onChange).toHaveBeenCalledWith({ type: 'field', key: 'subdomain', value: 'acme' });
  });

  it('emits toggle event on history option click', () => {
    const onChange = vi.fn();
    renderWithProviders(<SageHrConnectorForm {...baseProps({ onChange })} />);
    fireEvent.click(screen.getByText('Team History'));
    expect(onChange).toHaveBeenCalledWith({ type: 'toggle', key: 'includeTeamHistory' });
  });

  it('calls validateField on blur', () => {
    const validateField = vi.fn();
    renderWithProviders(<SageHrConnectorForm {...baseProps({ validateField })} />);
    fireEvent.blur(screen.getByLabelText(/Subdomain/i));
    expect(validateField).toHaveBeenCalledWith('sageHr.subdomain');
  });

  it('shows active state for enabled history toggle', () => {
    const props = baseProps();
    props.form.sageHr.includeTeamHistory = true;
    renderWithProviders(<SageHrConnectorForm {...props} />);
    const teamHistoryBtn = screen.getByText('Team History');
    expect(teamHistoryBtn).toHaveAttribute('aria-pressed', 'true');
  });
});
