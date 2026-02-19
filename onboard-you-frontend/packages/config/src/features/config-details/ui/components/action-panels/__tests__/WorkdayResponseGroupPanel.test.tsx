import { describe, it, expect, vi } from 'vitest';
import { screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { renderWithProviders } from '@/shared/test/testWrapper';
import { WorkdayResponseGroupPanel } from '../WorkdayResponseGroupPanel';

function renderPanel(config: Record<string, unknown>) {
  const onChange = vi.fn();
  renderWithProviders(
    <WorkdayResponseGroupPanel config={config} onChange={onChange} availableColumns={[]} />,
  );
  return { onChange };
}

describe('WorkdayResponseGroupPanel', () => {
  it('renders all response group options', () => {
    renderPanel({ response_group: {} });
    expect(screen.getByText('Personal Information')).toBeInTheDocument();
    expect(screen.getByText('Employment Information')).toBeInTheDocument();
    expect(screen.getByText('Compensation')).toBeInTheDocument();
    expect(screen.getByText('Organizations')).toBeInTheDocument();
    expect(screen.getByText('Roles')).toBeInTheDocument();
  });

  it('toggles a response group on click', async () => {
    const user = userEvent.setup();
    const { onChange } = renderPanel({
      response_group: {
        include_personal_information: true,
        include_employment_information: false,
      },
    });
    await user.click(screen.getByText('Employment Information'));
    expect(onChange).toHaveBeenCalledWith('response_group', {
      include_personal_information: true,
      include_employment_information: true,
    });
  });

  it('toggles off a selected group', async () => {
    const user = userEvent.setup();
    const { onChange } = renderPanel({
      response_group: {
        include_personal_information: true,
      },
    });
    await user.click(screen.getByText('Personal Information'));
    expect(onChange).toHaveBeenCalledWith('response_group', {
      include_personal_information: false,
    });
  });

  it('handles missing response_group gracefully', () => {
    renderPanel({});
    expect(screen.getByTestId('workday-response-group-panel')).toBeInTheDocument();
  });
});
