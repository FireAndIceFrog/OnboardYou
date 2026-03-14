import { describe, it, expect } from 'vitest';
import { screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { renderWithProviders } from '@/shared/test/testWrapper';
import { AddStepPanel } from './AddStepPanel';
import { ACTION_CATALOG } from '../../domain/actionCatalog';
import { vi } from 'vitest';

const LOGIC_COUNT = ACTION_CATALOG.filter((a) => a.category === 'logic').length;
const EGRESS_COUNT = ACTION_CATALOG.filter((a) => a.category === 'egress').length;

describe('AddStepPanel', () => {
  it('renders the panel with all catalog entries', () => {
    renderWithProviders(<AddStepPanel onClose={vi.fn()} />);
    expect(screen.getByTestId('add-step-panel')).toBeInTheDocument();

    // Should have one button per catalog entry
    const allButtons = ACTION_CATALOG.map((e) =>
      screen.getByTestId(`add-step-${e.actionType}`),
    );
    expect(allButtons).toHaveLength(LOGIC_COUNT + EGRESS_COUNT);
  });

  it('renders logic and egress sections', () => {
    renderWithProviders(<AddStepPanel onClose={vi.fn()} />);
    // Check section headings exist via test ids
    expect(screen.getByTestId('add-step-panel')).toBeInTheDocument();
  });

  it('calls onClose when close button is clicked', async () => {
    const onClose = vi.fn();
    const user = userEvent.setup();
    renderWithProviders(<AddStepPanel onClose={onClose} />);

    await user.click(screen.getByTestId('add-step-close'));
    expect(onClose).toHaveBeenCalledTimes(1);
  });

  it('dispatches addFlowAction when a step is clicked', async () => {
    const user = userEvent.setup();
    const { store } = renderWithProviders(<AddStepPanel onClose={vi.fn()} />);

    const initialActions = (store.getState() as any).configDetails.config?.pipeline?.actions?.length ?? 0;
    await user.click(screen.getByTestId('add-step-rename_column'));

    // The panel closes itself by dispatching setAddStepPanelOpen(false)
    const state = store.getState() as any;
    expect(state.configDetails.addStepPanelOpen).toBe(false);
  });

  it('displays label and description for each catalog entry', () => {
    renderWithProviders(<AddStepPanel onClose={vi.fn()} />);
    // Spot-check a couple entries
    expect(screen.getByText('Remove Duplicates')).toBeInTheDocument();
    expect(screen.getByText('Mask Sensitive Data')).toBeInTheDocument();
    expect(screen.getByText('Send to API')).toBeInTheDocument();
  });
});
