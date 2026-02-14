import type { ApiClient } from '@/shared/services';
import type { PipelineConfig } from '@/shared/domain/types';

export async function fetchConfigs(apiClient: ApiClient): Promise<PipelineConfig[]> {
  return apiClient.get<PipelineConfig[]>('/config');
}

// Note: The backend API does not currently expose a DELETE endpoint.
// Configs are managed via POST (create) and PUT (update) only.
