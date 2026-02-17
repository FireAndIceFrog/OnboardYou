import {
  getConfig as getConfigApi,
  createConfig as createConfigApi,
  updateConfig as updateConfigApi,
  validateConfig as validateConfigApi,
} from '@/generated/api';
import type { PipelineConfig, ValidationResult, ConfigRequest } from '@/generated/api';

export async function fetchConfig(
  customerCompanyId: string,
): Promise<PipelineConfig> {
  const { data } = await getConfigApi({
    path: { customer_company_id: customerCompanyId },
    throwOnError: true,
  });
  return data;
}

export async function createConfig(
  customerCompanyId: string,
  body: ConfigRequest,
): Promise<PipelineConfig> {
  const { data } = await createConfigApi({
    path: { customer_company_id: customerCompanyId },
    body,
    throwOnError: true,
  });
  return data;
}

export async function saveConfig(
  customerCompanyId: string,
  body: ConfigRequest,
): Promise<PipelineConfig> {
  const { data } = await updateConfigApi({
    path: { customer_company_id: customerCompanyId },
    body,
    throwOnError: true,
  });
  return data;
}

export async function validateConfig(
  customerCompanyId: string,
  body: ConfigRequest,
): Promise<ValidationResult> {
  const { data } = await validateConfigApi({
    path: { customer_company_id: customerCompanyId },
    body,
    throwOnError: true,
  });
  return data;
}
