import { useEffect, useRef } from 'react';
import { useAppSelector } from '@/store';
import { selectIsAuthenticated } from '@/features/auth/state/authSlice';
import { API_BASE_URL } from '@/shared/domain/constants';
import '@mcp-b/global';

/* ── Helpers ──────────────────────────────────────────────── */

function authHeaders(): HeadersInit {
  const token = sessionStorage.getItem('oy_access_token');
  return token ? { Authorization: `Bearer ${token}`, 'Content-Type': 'application/json' } : {};
}

async function apiFetch(path: string, init?: RequestInit) {
  const res = await fetch(`${API_BASE_URL}${path}`, {
    ...init,
    headers: { ...authHeaders(), ...init?.headers },
  });
  if (!res.ok) {
    const body = await res.json().catch(() => ({}));
    throw new Error((body as { error?: string }).error ?? `API ${res.status}`);
  }
  if (res.status === 204) return null;
  return res.json();
}

function textResult(text: string) {
  return { content: [{ type: 'text' as const, text }] };
}

/* ── Tool definitions ─────────────────────────────────────── */

const TOOL_NAMES = [
  'list_configs',
  'get_config',
  'create_config',
  'update_config',
  'delete_config',
  'validate_config',
  'generate_plan',
  'get_settings',
  'update_settings',
  'csv_columns',
] as const;

function registerTools() {
  const mc = navigator.modelContext;

  mc.registerTool({
    name: 'list_configs',
    description: 'List all pipeline configurations for the current organization',
    inputSchema: { type: 'object', properties: {} },
    annotations: { readOnlyHint: true },
    async execute() {
      const data = await apiFetch('/config');
      return textResult(JSON.stringify(data, null, 2));
    },
  });

  mc.registerTool({
    name: 'get_config',
    description: 'Get a specific pipeline configuration by customer company ID',
    inputSchema: {
      type: 'object',
      properties: {
        customer_company_id: { type: 'string', description: 'Customer company identifier' },
      },
      required: ['customer_company_id'],
    },
    annotations: { readOnlyHint: true },
    async execute(args: Record<string, unknown>) {
      const id = args.customer_company_id as string;
      const data = await apiFetch(`/config/${encodeURIComponent(id)}`);
      return textResult(JSON.stringify(data, null, 2));
    },
  });

  mc.registerTool({
    name: 'create_config',
    description: 'Create a new pipeline configuration',
    inputSchema: {
      type: 'object',
      properties: {
        customer_company_id: { type: 'string', description: 'Customer company identifier' },
        config: { type: 'object', description: 'Pipeline configuration object (PipelineConfig)' },
      },
      required: ['customer_company_id', 'config'],
    },
    async execute(args: Record<string, unknown>) {
      const id = args.customer_company_id as string;
      const data = await apiFetch(`/config/${encodeURIComponent(id)}`, {
        method: 'POST',
        body: JSON.stringify(args.config),
      });
      return textResult(JSON.stringify(data, null, 2));
    },
  });

  mc.registerTool({
    name: 'update_config',
    description: 'Update an existing pipeline configuration',
    inputSchema: {
      type: 'object',
      properties: {
        customer_company_id: { type: 'string', description: 'Customer company identifier' },
        config: { type: 'object', description: 'Updated pipeline configuration object (PipelineConfig)' },
      },
      required: ['customer_company_id', 'config'],
    },
    async execute(args: Record<string, unknown>) {
      const id = args.customer_company_id as string;
      const data = await apiFetch(`/config/${encodeURIComponent(id)}`, {
        method: 'PUT',
        body: JSON.stringify(args.config),
      });
      return textResult(JSON.stringify(data, null, 2));
    },
  });

  mc.registerTool({
    name: 'delete_config',
    description: 'Delete a pipeline configuration',
    inputSchema: {
      type: 'object',
      properties: {
        customer_company_id: { type: 'string', description: 'Customer company identifier' },
      },
      required: ['customer_company_id'],
    },
    async execute(args: Record<string, unknown>) {
      const id = args.customer_company_id as string;
      await apiFetch(`/config/${encodeURIComponent(id)}`, { method: 'DELETE' });
      return textResult(`Configuration "${id}" deleted successfully.`);
    },
  });

  mc.registerTool({
    name: 'validate_config',
    description: 'Dry-run validate a pipeline configuration (column propagation check)',
    inputSchema: {
      type: 'object',
      properties: {
        customer_company_id: { type: 'string', description: 'Customer company identifier' },
        config: { type: 'object', description: 'Pipeline configuration to validate' },
      },
      required: ['customer_company_id', 'config'],
    },
    annotations: { readOnlyHint: true },
    async execute(args: Record<string, unknown>) {
      const id = args.customer_company_id as string;
      const data = await apiFetch(`/config/${encodeURIComponent(id)}/validate`, {
        method: 'POST',
        body: JSON.stringify(args.config),
      });
      return textResult(JSON.stringify(data, null, 2));
    },
  });

  mc.registerTool({
    name: 'generate_plan',
    description: 'Trigger async AI plan generation for a pipeline configuration (returns 202)',
    inputSchema: {
      type: 'object',
      properties: {
        customer_company_id: { type: 'string', description: 'Customer company identifier' },
      },
      required: ['customer_company_id'],
    },
    async execute(args: Record<string, unknown>) {
      const id = args.customer_company_id as string;
      const data = await apiFetch(`/config/${encodeURIComponent(id)}/generate-plan`, {
        method: 'POST',
        body: JSON.stringify({}),
      });
      return textResult(JSON.stringify(data, null, 2));
    },
  });

  mc.registerTool({
    name: 'get_settings',
    description: 'Get organization settings',
    inputSchema: { type: 'object', properties: {} },
    annotations: { readOnlyHint: true },
    async execute() {
      const data = await apiFetch('/settings');
      return textResult(JSON.stringify(data, null, 2));
    },
  });

  mc.registerTool({
    name: 'update_settings',
    description: 'Create or update organization settings',
    inputSchema: {
      type: 'object',
      properties: {
        settings: { type: 'object', description: 'Organization settings object (OrgSettings)' },
      },
      required: ['settings'],
    },
    async execute(args: Record<string, unknown>) {
      const data = await apiFetch('/settings', {
        method: 'PUT',
        body: JSON.stringify(args.settings),
      });
      return textResult(JSON.stringify(data, null, 2));
    },
  });

  mc.registerTool({
    name: 'csv_columns',
    description: 'Get column headers of an uploaded CSV file',
    inputSchema: {
      type: 'object',
      properties: {
        customer_company_id: { type: 'string', description: 'Customer company identifier' },
        filename: { type: 'string', description: 'CSV filename (e.g. "employees.csv")' },
      },
      required: ['customer_company_id', 'filename'],
    },
    annotations: { readOnlyHint: true },
    async execute(args: Record<string, unknown>) {
      const id = args.customer_company_id as string;
      const filename = args.filename as string;
      const data = await apiFetch(
        `/config/${encodeURIComponent(id)}/csv-columns?filename=${encodeURIComponent(filename)}`,
      );
      return textResult(JSON.stringify(data, null, 2));
    },
  });
}

function unregisterTools() {
  for (const name of TOOL_NAMES) {
    try {
      navigator.modelContext.unregisterTool(name);
    } catch {
      // tool may not have been registered
    }
  }
}

/* ── React hook ───────────────────────────────────────────── */

export function useWebMCP() {
  const isAuthenticated = useAppSelector(selectIsAuthenticated);
  const registered = useRef(false);

  useEffect(() => {
    if (!isAuthenticated) {
      if (registered.current) {
        unregisterTools();
        registered.current = false;
      }
      return;
    }

    if (registered.current) return;
    registerTools();
    registered.current = true;

    return () => {
      unregisterTools();
      registered.current = false;
    };
  }, [isAuthenticated]);
}
