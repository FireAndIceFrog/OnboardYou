import { ApiClient } from '@/shared/services';
import type { PipelineConfig } from '@/shared/domain/types';

interface ConfigResponse {
  data: PipelineConfig;
}

export async function fetchConfig(
  apiClient: ApiClient,
  configId: string,
): Promise<PipelineConfig> {
  const response = await apiClient.get<ConfigResponse>(`/configs/${configId}`);
  return response.data;
}

export async function saveConfig(
  apiClient: ApiClient,
  configId: string,
  data: Partial<PipelineConfig>,
): Promise<PipelineConfig> {
  const response = await apiClient.put<ConfigResponse>(`/configs/${configId}`, data);
  return response.data;
}
