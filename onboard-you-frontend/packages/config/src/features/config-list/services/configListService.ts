import type { ApiClient } from '@/shared/services';
import type { PipelineConfig } from '@/shared/domain/types';

export async function fetchConfigs(apiClient: ApiClient): Promise<PipelineConfig[]> {
  return apiClient.get<PipelineConfig[]>('/configs');
}

export async function deleteConfig(apiClient: ApiClient, id: string): Promise<void> {
  await apiClient.del(`/configs/${id}`);
}
