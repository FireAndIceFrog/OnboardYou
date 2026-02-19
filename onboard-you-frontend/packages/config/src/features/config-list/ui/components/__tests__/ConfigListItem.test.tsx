import { describe, it, expect, vi, beforeEach } from 'vitest';
import { screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { renderWithProviders } from '@/shared/test/testWrapper';
import { ConfigListItem, relativeTime, fullDate } from '../ConfigListItem';
import type { PipelineConfig } from '@/shared/domain/types';

// Mock react-router-dom to track navigation
const mockNavigate = vi.fn();
vi.mock('react-router-dom', async () => {
  const actual = await vi.importActual<typeof import('react-router-dom')>('react-router-dom');
  return { ...actual, useNavigate: () => mockNavigate };
});

function makeConfig(overrides: Partial<PipelineConfig> = {}): PipelineConfig {
  return {
    customerCompanyId: 'acme-corp',
    name: 'Acme Integration',
    cron: 'rate(1 day)',
    organizationId: 'org-1',
    pipeline: { version: '1.0', actions: [{ id: 'a1', action_type: 'rename_column' as any, config: { mappings: {} } as any }] },
    lastEdited: new Date(Date.now() - 3_600_000 * 2).toISOString(), // 2h ago
    ...overrides,
  };
}

describe('ConfigListItem', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders the config name and company id', () => {
    renderWithProviders(<ConfigListItem config={makeConfig()} />);
    expect(screen.getByText('Acme Integration')).toBeInTheDocument();
    expect(screen.getByText('acme-corp')).toBeInTheDocument();
  });

  it('displays the status badge', () => {
    renderWithProviders(<ConfigListItem config={makeConfig()} />);
    // 2h ago config with actions → healthy → "Healthy"
    expect(screen.getByText('Healthy')).toBeInTheDocument();
  });

  it('navigates on click', async () => {
    const user = userEvent.setup();
    renderWithProviders(<ConfigListItem config={makeConfig()} />);
    const card = screen.getByTestId('config-list-item-acme-corp');
    await user.click(card);
    expect(mockNavigate).toHaveBeenCalledWith('acme-corp');
  });

  it('navigates on Enter key', async () => {
    const user = userEvent.setup();
    renderWithProviders(<ConfigListItem config={makeConfig()} />);
    const card = screen.getByTestId('config-list-item-acme-corp');
    card.focus();
    await user.keyboard('{Enter}');
    expect(mockNavigate).toHaveBeenCalledWith('acme-corp');
  });
});

describe('relativeTime', () => {
  it('returns empty string for empty input', () => {
    expect(relativeTime('')).toBe('');
  });

  it('returns "just now" for very recent dates', () => {
    expect(relativeTime(new Date().toISOString())).toBe('just now');
  });

  it('returns hours format for a few hours ago', () => {
    const twoHoursAgo = new Date(Date.now() - 2 * 60 * 60_000).toISOString();
    expect(relativeTime(twoHoursAgo)).toBe('2h ago');
  });

  it('returns days format', () => {
    const fiveDaysAgo = new Date(Date.now() - 5 * 24 * 60 * 60_000).toISOString();
    expect(relativeTime(fiveDaysAgo)).toBe('5d ago');
  });
});

describe('fullDate', () => {
  it('returns empty string for empty input', () => {
    expect(fullDate('')).toBe('');
  });

  it('returns a formatted date string', () => {
    const result = fullDate('2025-06-15T10:30:00Z');
    // Just verify it produces something non-empty and contains the year
    expect(result).toBeTruthy();
    expect(result).toContain('2025');
  });
});
