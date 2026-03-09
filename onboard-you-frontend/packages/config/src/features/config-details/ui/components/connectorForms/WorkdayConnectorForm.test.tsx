import { describe, it, expect, vi } from 'vitest';
import { screen, fireEvent } from '@testing-library/react';
import { renderWithProviders } from '@/shared/test/testWrapper';
import { WorkdayConnectorForm } from './WorkdayConnectorForm';
import type { ConnectorFormProps } from './types';

function baseProps(overrides: Partial<ConnectorFormProps> = {}): ConnectorFormProps {
  return {
    form: {
      system: 'workday' as any,
      displayName: 'Test',
      workday: { tenantUrl: '', tenantId: '', username: '', password: '', workerCountLimit: '200', responseGroup: 'include_personal_information,include_employment_information' },
      sageHr: { subdomain: '', apiToken: '', includeTeamHistory: false, includeEmploymentStatusHistory: false, includePositionHistory: false },
      csv: { filename: '', columns: [], uploadStatus: 'idle' as const, uploadError: null },
    },
    errors: {},
    config: { getActionConfig: vi.fn(), getDefaultState: vi.fn(), applyChange: vi.fn() as any, validate: vi.fn(() => ({})), isFormValid: vi.fn(() => false) },
    onChange: vi.fn(),
    validateField: vi.fn(),
    ...overrides,
  };
}

describe('WorkdayConnectorForm', () => {
  it('renders credential fields', () => {
    renderWithProviders(<WorkdayConnectorForm {...baseProps()} />);
    expect(screen.getByLabelText(/Tenant URL/i)).toBeInTheDocument();
    expect(screen.getByLabelText(/Username/i)).toBeInTheDocument();
    expect(screen.getByLabelText(/Password/i)).toBeInTheDocument();
  });

  it('renders response group toggle buttons', () => {
    renderWithProviders(<WorkdayConnectorForm {...baseProps()} />);
    expect(screen.getByText('Personal Information')).toBeInTheDocument();
    expect(screen.getByText('Compensation')).toBeInTheDocument();
  });

  it('emits field change event on input', () => {
    const onChange = vi.fn();
    renderWithProviders(<WorkdayConnectorForm {...baseProps({ onChange })} />);
    fireEvent.change(screen.getByLabelText(/Tenant URL/i), { target: { value: 'https://test.com' } });
    expect(onChange).toHaveBeenCalledWith({ type: 'field', key: 'tenantUrl', value: 'https://test.com' });
  });

  it('emits toggle event on response group click', () => {
    const onChange = vi.fn();
    renderWithProviders(<WorkdayConnectorForm {...baseProps({ onChange })} />);
    fireEvent.click(screen.getByText('Compensation'));
    expect(onChange).toHaveBeenCalledWith({ type: 'toggle', key: 'include_compensation' });
  });

  it('calls validateField on blur', () => {
    const validateField = vi.fn();
    renderWithProviders(<WorkdayConnectorForm {...baseProps({ validateField })} />);
    fireEvent.blur(screen.getByLabelText(/Tenant URL/i));
    expect(validateField).toHaveBeenCalledWith('workday.tenantUrl');
  });

  it('shows active state for enabled response groups', () => {
    renderWithProviders(<WorkdayConnectorForm {...baseProps()} />);
    const personalBtn = screen.getByText('Personal Information');
    expect(personalBtn).toHaveAttribute('aria-pressed', 'true');
  });
});
