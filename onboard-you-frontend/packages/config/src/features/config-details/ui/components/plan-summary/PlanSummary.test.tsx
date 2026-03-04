import { describe, it, expect, vi } from 'vitest';
import { screen, fireEvent } from '@testing-library/react';
import { renderWithProviders, createTestStore } from '@/shared/test/testWrapper';
import type { PlanSummary, ActionConfig, ActionConfigPayload, PlanFeature, PlanPreview } from '@/generated/api';
import type { ConfigDetailsState } from '../../../domain/types';
import { PlanSummaryView } from '../plan-summary/PlanSummaryView';
import { PlanFeatureCard } from '../plan-summary/PlanFeatureCard';
import { PlanPreviewCard } from '../plan-summary/PlanPreviewCard';
import { NormalAdvancedToggle } from '../plan-summary/NormalAdvancedToggle';
import configDetailsReducer, {
  toggleFeature,
} from '../../../state/configDetailsSlice';

/* ── Mock data ──────────────────────────────────────────────── */

const mockPreview: PlanPreview = {
  sourceLabel: 'In Workday',
  targetLabel: 'In Your App',
  before: {
    name: 'Jane Doe',
    status: 'Active',
    email: 'jane.doe@example.com',
  },
  after: {
    name: 'Jane Doe',
    email: 'jane.doe@example.com',
    department: 'Northeast Sales',
  },
};

const mockFeatures: PlanFeature[] = [
  {
    id: 'sync_start_dates',
    icon: 'calendar',
    label: 'Sync Start Dates',
    description: "Use the employee's original start date.",
    actionIds: ['step_3'],
  },
  {
    id: 'active_only',
    icon: 'users',
    label: 'Active Employees Only',
    description: 'Only sync people currently employed.',
    actionIds: ['step_2'],
  },
  {
    id: 'prevent_duplicates',
    icon: 'shield',
    label: 'Prevent Duplicates',
    description: 'Match people by work email to avoid copies.',
    actionIds: ['step_4'],
  },
];

const mockPlanSummary: PlanSummary = {
  headline: "Here's the plan to connect Workday to your App.",
  description: "We've designed a simple sync for your active employees.",
  features: mockFeatures,
  preview: mockPreview,
  generationStatus: 'completed',
};

const mockActions: ActionConfig[] = [
  { id: 'step_1', action_type: 'workday_hris_connector', config: { tenant_url: '', tenant_id: '', username: '', password: '', response_group: { include_personal_information: true, include_employment_information: true, include_compensation: false, include_organizations: false, include_roles: false } } },
  { id: 'step_2', action_type: 'filter_by_value', config: { column: 'worker_status', value: 'Active' } as unknown as ActionConfigPayload, disabled: false },
  { id: 'step_3', action_type: 'rename_column', config: { mapping: { hire_date: 'startDate' } }, disabled: false },
  { id: 'step_4', action_type: 'identity_deduplicator', config: { match_column: 'work_email' } as unknown as ActionConfigPayload, disabled: false },
  { id: 'step_5', action_type: 'api_dispatcher', config: { endpoint_url: 'https://example.com/api', method: 'POST', schema: {}, body_path: '$.employees' } as unknown as ActionConfigPayload },
];

/* ── Component Tests ─────────────────────────────────────── */

describe('PlanSummaryView', () => {
  it('renders with mock PlanSummary data', () => {
    renderWithProviders(
      <PlanSummaryView
        planSummary={mockPlanSummary}
        actions={mockActions}
        isSaving={false}
        isGenerating={false}
        onToggleFeature={vi.fn()}
        onApplyPlan={vi.fn()}
        onMakeChanges={vi.fn()}
      />,
    );

    expect(screen.getByText(mockPlanSummary.headline)).toBeInTheDocument();
    expect(screen.getByText(mockPlanSummary.description)).toBeInTheDocument();
    expect(screen.getByText('How it will work')).toBeInTheDocument();
    expect(screen.getByText('Sync Start Dates')).toBeInTheDocument();
    expect(screen.getByText('Active Employees Only')).toBeInTheDocument();
    expect(screen.getByText('Prevent Duplicates')).toBeInTheDocument();
    expect(screen.getByText('Looks Good, Start Syncing')).toBeInTheDocument();
    expect(screen.getByText('Make Changes')).toBeInTheDocument();
  });

  it('shows loading state when generating', () => {
    renderWithProviders(
      <PlanSummaryView
        planSummary={mockPlanSummary}
        actions={mockActions}
        isSaving={false}
        isGenerating={true}
        onToggleFeature={vi.fn()}
        onApplyPlan={vi.fn()}
        onMakeChanges={vi.fn()}
      />,
    );

    expect(screen.getByText(/generating your plan/i)).toBeInTheDocument();
  });

  it('"Looks Good" triggers onApplyPlan', () => {
    const onApplyPlan = vi.fn();
    renderWithProviders(
      <PlanSummaryView
        planSummary={mockPlanSummary}
        actions={mockActions}
        isSaving={false}
        isGenerating={false}
        onToggleFeature={vi.fn()}
        onApplyPlan={onApplyPlan}
        onMakeChanges={vi.fn()}
      />,
    );

    fireEvent.click(screen.getByTestId('apply-plan-button'));
    expect(onApplyPlan).toHaveBeenCalledTimes(1);
  });

  it('"Make Changes" triggers onMakeChanges', () => {
    const onMakeChanges = vi.fn();
    renderWithProviders(
      <PlanSummaryView
        planSummary={mockPlanSummary}
        actions={mockActions}
        isSaving={false}
        isGenerating={false}
        onToggleFeature={vi.fn()}
        onApplyPlan={vi.fn()}
        onMakeChanges={onMakeChanges}
      />,
    );

    fireEvent.click(screen.getByTestId('make-changes-button'));
    expect(onMakeChanges).toHaveBeenCalledTimes(1);
  });
});

describe('PlanFeatureCard', () => {
  it('renders feature info and toggle', () => {
    renderWithProviders(
      <PlanFeatureCard
        feature={mockFeatures[0]}
        enabled={true}
        onToggle={vi.fn()}
      />,
    );

    expect(screen.getByText('Sync Start Dates')).toBeInTheDocument();
    expect(screen.getByText("Use the employee's original start date.")).toBeInTheDocument();
    const toggle = screen.getByTestId('feature-toggle-sync_start_dates');
    expect(toggle).toHaveAttribute('aria-checked', 'true');
  });

  it('calls onToggle with feature id when clicked', () => {
    const onToggle = vi.fn();
    renderWithProviders(
      <PlanFeatureCard
        feature={mockFeatures[1]}
        enabled={true}
        onToggle={onToggle}
      />,
    );

    fireEvent.click(screen.getByTestId('feature-toggle-active_only'));
    expect(onToggle).toHaveBeenCalledWith('active_only');
  });
});

describe('PlanPreviewCard', () => {
  it('renders before/after preview data', () => {
    renderWithProviders(<PlanPreviewCard preview={mockPreview} />);

    expect(screen.getByText('In Workday')).toBeInTheDocument();
    expect(screen.getByText('In Your App')).toBeInTheDocument();
    // Jane Doe appears in both before and after cards
    expect(screen.getAllByText('Jane Doe')).toHaveLength(2);
    expect(screen.getByText('Northeast Sales')).toBeInTheDocument();
  });
});

describe('NormalAdvancedToggle', () => {
  it('renders Normal and Advanced options', () => {
    renderWithProviders(
      <NormalAdvancedToggle viewMode="normal" onToggle={vi.fn()} />,
    );

    expect(screen.getByText('Normal')).toBeInTheDocument();
    expect(screen.getByText('Advanced')).toBeInTheDocument();
  });

  it('switches to advanced when clicked', () => {
    const onToggle = vi.fn();
    renderWithProviders(
      <NormalAdvancedToggle viewMode="normal" onToggle={onToggle} />,
    );

    fireEvent.click(screen.getByTestId('toggle-advanced'));
    expect(onToggle).toHaveBeenCalledWith('advanced');
  });

  it('switches to normal when clicked', () => {
    const onToggle = vi.fn();
    renderWithProviders(
      <NormalAdvancedToggle viewMode="advanced" onToggle={onToggle} />,
    );

    fireEvent.click(screen.getByTestId('toggle-normal'));
    expect(onToggle).toHaveBeenCalledWith('normal');
  });
});

/* ── Reducer Tests ─────────────────────────────────────────── */

describe('toggleFeature reducer', () => {
  function makeConfigState(): ConfigDetailsState {
    return {
      config: {
        name: 'Test',
        cron: 'rate(1 day)',
        organizationId: 'org-1',
        customerCompanyId: 'co-1',
        pipeline: {
          version: '1.0',
          actions: structuredClone(mockActions),
        },
      },
      nodes: [],
      edges: [],
      selectedNode: null,
      isLoading: false,
      isSaving: false,
      isDeleting: false,
      isValidating: false,
      error: null,
      chatOpen: false,
      addStepPanelOpen: false,
      validationResult: null,
      planSummary: mockPlanSummary,
      isGeneratingPlan: false,
      viewMode: 'normal',
    };
  }

  it('sets disabled: true on linked actions when toggling off', () => {
    const state = makeConfigState();
    const result = configDetailsReducer(state, toggleFeature('active_only'));

    // step_2 should now be disabled
    const step2 = result.config!.pipeline.actions.find((a) => a.id === 'step_2');
    expect(step2?.disabled).toBe(true);

    // Other actions remain unchanged
    const step3 = result.config!.pipeline.actions.find((a) => a.id === 'step_3');
    expect(step3?.disabled).toBe(false);
  });

  it('sets disabled: false on linked actions when toggling back on', () => {
    const state = makeConfigState();
    // First disable
    const disabled = configDetailsReducer(state, toggleFeature('active_only'));
    // Then re-enable
    const result = configDetailsReducer(disabled, toggleFeature('active_only'));

    const step2 = result.config!.pipeline.actions.find((a) => a.id === 'step_2');
    expect(step2?.disabled).toBe(false);
  });

  it('does not add or remove actions', () => {
    const state = makeConfigState();
    const before = state.config!.pipeline.actions.length;
    const result = configDetailsReducer(state, toggleFeature('prevent_duplicates'));
    expect(result.config!.pipeline.actions.length).toBe(before);
  });

  it('handles unknown feature id gracefully', () => {
    const state = makeConfigState();
    const result = configDetailsReducer(state, toggleFeature('nonexistent'));
    expect(result.config!.pipeline.actions).toEqual(state.config!.pipeline.actions);
  });
});
