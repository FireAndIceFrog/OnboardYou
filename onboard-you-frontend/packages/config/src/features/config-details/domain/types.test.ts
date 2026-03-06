import { describe, it, expect } from 'vitest';
import { buildResponseGroup, buildSageHrConfig } from './types';
import type { SageHrFields } from './types';

/* ── buildResponseGroup ──────────────────────────────────── */

interface ResponseGroupCase {
  name: string;
  csv: string;
  expected: Record<string, boolean>;
}

const responseGroupCases: ResponseGroupCase[] = [
  {
    name: 'all groups active',
    csv: 'include_personal_information,include_employment_information,include_compensation,include_organizations,include_roles',
    expected: {
      include_personal_information: true,
      include_employment_information: true,
      include_compensation: true,
      include_organizations: true,
      include_roles: true,
    },
  },
  {
    name: 'empty string produces all false',
    csv: '',
    expected: {
      include_personal_information: false,
      include_employment_information: false,
      include_compensation: false,
      include_organizations: false,
      include_roles: false,
    },
  },
  {
    name: 'single group active',
    csv: 'include_compensation',
    expected: {
      include_personal_information: false,
      include_employment_information: false,
      include_compensation: true,
      include_organizations: false,
      include_roles: false,
    },
  },
  {
    name: 'two groups active',
    csv: 'include_personal_information,include_roles',
    expected: {
      include_personal_information: true,
      include_employment_information: false,
      include_compensation: false,
      include_organizations: false,
      include_roles: true,
    },
  },
];

describe('buildResponseGroup', () => {
  it.each(responseGroupCases)('$name', ({ csv, expected }) => {
    expect(buildResponseGroup(csv)).toEqual(expected);
  });
});

/* ── buildSageHrConfig ───────────────────────────────────── */

interface SageHrCase {
  name: string;
  fields: SageHrFields;
  expected: Record<string, unknown>;
}

const sageHrCases: SageHrCase[] = [
  {
    name: 'all history flags off omits them',
    fields: {
      subdomain: 'acme',
      apiToken: 'tok-123',
      includeTeamHistory: false,
      includeEmploymentStatusHistory: false,
      includePositionHistory: false,
    },
    expected: {
      subdomain: 'acme',
      api_token: 'tok-123',
      include_team_history: undefined,
      include_employment_status_history: undefined,
      include_position_history: undefined,
    },
  },
  {
    name: 'all history flags on',
    fields: {
      subdomain: 'globex',
      apiToken: 'secret-token',
      includeTeamHistory: true,
      includeEmploymentStatusHistory: true,
      includePositionHistory: true,
    },
    expected: {
      subdomain: 'globex',
      api_token: 'secret-token',
      include_team_history: true,
      include_employment_status_history: true,
      include_position_history: true,
    },
  },
  {
    name: 'mixed history flags',
    fields: {
      subdomain: 'initech',
      apiToken: 'api-key-xyz',
      includeTeamHistory: true,
      includeEmploymentStatusHistory: false,
      includePositionHistory: true,
    },
    expected: {
      subdomain: 'initech',
      api_token: 'api-key-xyz',
      include_team_history: true,
      include_employment_status_history: undefined,
      include_position_history: true,
    },
  },
  {
    name: 'trims whitespace from subdomain',
    fields: {
      subdomain: '  padded  ',
      apiToken: 'tok',
      includeTeamHistory: false,
      includeEmploymentStatusHistory: false,
      includePositionHistory: false,
    },
    expected: {
      subdomain: 'padded',
      api_token: 'tok',
      include_team_history: undefined,
      include_employment_status_history: undefined,
      include_position_history: undefined,
    },
  },
];

describe('buildSageHrConfig', () => {
  it.each(sageHrCases)('$name', ({ fields, expected }) => {
    expect(buildSageHrConfig(fields)).toEqual(expected);
  });
});
