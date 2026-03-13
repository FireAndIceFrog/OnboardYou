import {
  getConfig as getConfigApi,
  createConfig as createConfigApi,
  updateConfig as updateConfigApi,
  deleteConfig as deleteConfigApi,
  validateConfig as validateConfigApi,
  getSettings as getSettingsApi,
} from '@/generated/api';
import type { PipelineConfig, ValidationResult, ConfigRequest, OrgSettings } from '@/generated/api';

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

export async function deleteConfig(
  customerCompanyId: string,
): Promise<void> {
  await deleteConfigApi({
    path: { customer_company_id: customerCompanyId },
    throwOnError: true,
  });
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

export async function fetchSettings(): Promise<OrgSettings> {
  const { data } = await getSettingsApi({ throwOnError: true });
  return data;
}
