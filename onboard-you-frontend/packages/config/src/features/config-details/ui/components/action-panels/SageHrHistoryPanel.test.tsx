import { describe, it, expect, vi } from 'vitest';
import { screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { renderWithProviders } from '@/shared/test/testWrapper';
import { SageHrHistoryPanel } from './SageHrHistoryPanel';

/* ── Helpers ────────────────────────────────────────────── */

function renderPanel(config: Record<string, unknown>) {
  const onChange = vi.fn();
  renderWithProviders(
    <SageHrHistoryPanel config={config} onChange={onChange} availableColumns={[]} />,
  );
  return { onChange };
}

/* ── Declarative Cases ──────────────────────────────────── */

interface RenderCase {
  name: string;
  config: Record<string, unknown>;
  expectedTexts: string[];
  expectedTestId?: string;
}

const renderCases: RenderCase[] = [
  {
    name: 'renders all history toggle options',
    config: {},
    expectedTexts: ['Team History', 'Employment Status History', 'Position History'],
  },
  {
    name: 'renders panel heading',
    config: {},
    expectedTexts: ['History Options'],
    expectedTestId: 'sage-hr-history-panel',
  },
  {
    name: 'handles missing config gracefully',
    config: {},
    expectedTestId: 'sage-hr-history-panel',
    expectedTexts: [],
  },
];

interface ToggleCase {
  name: string;
  config: Record<string, unknown>;
  clickText: string;
  expectedKey: string;
  expectedValue: boolean;
}

const toggleCases: ToggleCase[] = [
  {
    name: 'toggle on team history',
    config: { include_team_history: false, include_employment_status_history: false, include_position_history: false },
    clickText: 'Team History',
    expectedKey: 'include_team_history',
    expectedValue: true,
  },
  {
    name: 'toggle off team history',
    config: { include_team_history: true },
    clickText: 'Team History',
    expectedKey: 'include_team_history',
    expectedValue: false,
  },
  {
    name: 'toggle on employment status history',
    config: { include_employment_status_history: false },
    clickText: 'Employment Status History',
    expectedKey: 'include_employment_status_history',
    expectedValue: true,
  },
  {
    name: 'toggle on position history',
    config: { include_position_history: false },
    clickText: 'Position History',
    expectedKey: 'include_position_history',
    expectedValue: true,
  },
];

/* ── Tests ──────────────────────────────────────────────── */

describe('SageHrHistoryPanel', () => {
  it.each(renderCases)('$name', ({ config, expectedTexts, expectedTestId }) => {
    renderPanel(config);
    for (const text of expectedTexts) {
      expect(screen.getByText(text)).toBeInTheDocument();
    }
    if (expectedTestId) {
      expect(screen.getByTestId(expectedTestId)).toBeInTheDocument();
    }
  });

  it.each(toggleCases)('$name', async ({ config, clickText, expectedKey, expectedValue }) => {
    const user = userEvent.setup();
    const { onChange } = renderPanel(config);
    await user.click(screen.getByText(clickText));
    expect(onChange).toHaveBeenCalledWith(expectedKey, expectedValue);
  });
});
