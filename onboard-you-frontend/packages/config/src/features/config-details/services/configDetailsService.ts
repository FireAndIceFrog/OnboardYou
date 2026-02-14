import type { ApiClient } from '@/shared/services';
import type { PipelineConfig, ValidationResult } from '@/shared/domain/types';

export async function fetchConfig(
  apiClient: ApiClient,
  customerCompanyId: string,
): Promise<PipelineConfig> {
  return apiClient.get<PipelineConfig>(`/config/${customerCompanyId}`);
}

export async function createConfig(
  apiClient: ApiClient,
  customerCompanyId: string,
  data: PipelineConfig,
): Promise<PipelineConfig> {
  return apiClient.post<PipelineConfig>(`/config/${customerCompanyId}`, data);
}

export async function saveConfig(
  apiClient: ApiClient,
  customerCompanyId: string,
  data: PipelineConfig,
): Promise<PipelineConfig> {
  return apiClient.put<PipelineConfig>(`/config/${customerCompanyId}`, data);
}

export async function validateConfig(
  apiClient: ApiClient,
  customerCompanyId: string,
  data: PipelineConfig,
): Promise<ValidationResult> {
  return apiClient.post<ValidationResult>(`/config/${customerCompanyId}/validate`, data);
}
