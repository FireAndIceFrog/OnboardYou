import { listConfigs } from '@/generated/api';
import type { PipelineConfig } from '@/generated/api';

export async function fetchConfigs(): Promise<PipelineConfig[]> {
  const { data } = await listConfigs({ throwOnError: true });
  return data;
}

// Note: The backend API does not currently expose a DELETE endpoint.
// Configs are managed via POST (create) and PUT (update) only.
