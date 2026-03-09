import { describe, it, expect, vi } from 'vitest';
import { renderHook, act } from '@testing-library/react';

vi.mock('react-router-dom', () => ({
  useNavigate: () => vi.fn(),
  useParams: () => ({ customerCompanyId: 'test-company' }),
}));

vi.mock('react-i18next', () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));

vi.mock('../services/csvUploadService', () => ({
  validateCsvFile: vi.fn(),
  uploadCsvAndDiscoverColumns: vi.fn(),
}));

import { useConnectionForm } from './useConnectionForm';
import { ConnectorType } from './connectorConfigs/connectorConfigFactory';

/* ── Helpers ─────────────────────────────────────────────── */

type HookResult = ReturnType<typeof useConnectionForm>;
const fakeEvent = (value: string) => ({ target: { value } }) as React.ChangeEvent<HTMLInputElement>;

async function applyFields(
  result: { current: HookResult },
  system: ConnectorType,
  displayName: string,
  fields: Record<string, string>,
) {
  act(() => result.current.handleSystemSelect(system));
  act(() => result.current.handleChange('displayName')(fakeEvent(displayName)));

  for (const [key, value] of Object.entries(fields)) {
    await act(async () => {
      result.current.handleConnectorChange({ type: 'field', key, value });
    });
  }
}

/* ── Cases ───────────────────────────────────────────────── */

interface ValidityCase {
  name: string;
  system: ConnectorType;
  displayName: string;
  fields: Record<string, string>;
  expectedValid: boolean;
}

const validityCases: ValidityCase[] = [
  {
    name: 'sage_hr: valid when subdomain and token >= 8 chars',
    system: ConnectorType.SageHR,
    displayName: 'My SageHR',
    fields: { subdomain: 'acme', apiToken: 'longenoughtoken' },
    expectedValid: true,
  },
  {
    name: 'sage_hr: invalid when subdomain missing',
    system: ConnectorType.SageHR,
    displayName: 'Sage Test',
    fields: { subdomain: '', apiToken: 'longenoughtoken' },
    expectedValid: false,
  },
  {
    name: 'sage_hr: invalid when token too short',
    system: ConnectorType.SageHR,
    displayName: 'Sage Test',
    fields: { subdomain: 'acme', apiToken: 'short' },
    expectedValid: false,
  },
  {
    name: 'sage_hr: invalid when token missing',
    system: ConnectorType.SageHR,
    displayName: 'Sage Test',
    fields: { subdomain: 'acme', apiToken: '' },
    expectedValid: false,
  },
  {
    name: 'workday: valid with all required fields',
    system: ConnectorType.Workday,
    displayName: 'My Workday',
    fields: {
      tenantUrl: 'https://example.workday.com',
      tenantId: 'v40.0',
      username: 'admin',
      password: 'securepass',
    },
    expectedValid: true,
  },
  {
    name: 'workday: invalid when url not https',
    system: ConnectorType.Workday,
    displayName: 'My Workday',
    fields: {
      tenantUrl: 'not-a-url',
      tenantId: 'v40.0',
      username: 'admin',
      password: 'securepass',
    },
    expectedValid: false,
  },
  {
    name: 'workday: invalid when password too short',
    system: ConnectorType.Workday,
    displayName: 'My Workday',
    fields: {
      tenantUrl: 'https://example.workday.com',
      tenantId: 'v40.0',
      username: 'admin',
      password: 'short',
    },
    expectedValid: false,
  },
  {
    name: 'any system: invalid when displayName empty',
    system: ConnectorType.SageHR,
    displayName: '',
    fields: { subdomain: 'acme', apiToken: 'longenoughtoken' },
    expectedValid: false,
  },
];

/* ── Tests ───────────────────────────────────────────────── */

describe('useConnectionForm', () => {
  it('initial state is invalid', () => {
    const { result } = renderHook(() => useConnectionForm());
    expect(result.current.isValid).toBe(false);
  });

  it.each(validityCases)('$name', async ({ system, displayName, fields, expectedValid }) => {
    const { result } = renderHook(() => useConnectionForm());
    await applyFields(result, system, displayName, fields);
    expect(result.current.isValid).toBe(expectedValid);
  });

  it('exposes no connector-specific handlers', () => {
    const source = useConnectionForm.toString();
    const leakedHandlers = [
      'handleWorkdayChange',
      'handleSageHrChange',
      'handleSageHrHistoryToggle',
      'handleCsvFileSelect',
      'handleResponseGroupToggle',
      'activeGroups',
    ];
    for (const name of leakedHandlers) {
      expect(source).not.toContain(name);
    }
  });
});
