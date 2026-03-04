import {
  generatePlan as generatePlanApi,
  getConfig as getConfigApi,
} from '@/generated/api';
import type { PipelineConfig, GeneratePlanResponse } from '@/generated/api';

/**
 * Trigger async plan generation. Returns immediately with `{ status: "InProgress" }`.
 * The frontend should then poll `GET /config/{id}` until `planSummary.generationStatus`
 * flips to `"completed"` or `{ failed: "..." }`.
 */
export async function triggerPlanGeneration(
  customerCompanyId: string,
  sourceSystem: string,
): Promise<GeneratePlanResponse> {
  const { data } = await generatePlanApi({
    path: { customer_company_id: customerCompanyId },
    body: { sourceSystem },
    throwOnError: true,
  });
  return data;
}

/**
 * Poll the config until plan generation is complete.
 * Resolves with the updated PipelineConfig once generationStatus
 * is 'completed' or '{ failed: ... }'.
 */
export async function pollForPlanCompletion(
  customerCompanyId: string,
  intervalMs = 2500,
  maxAttempts = 60,
): Promise<PipelineConfig> {
  for (let attempt = 0; attempt < maxAttempts; attempt++) {
    const { data: config } = await getConfigApi({
      path: { customer_company_id: customerCompanyId },
      throwOnError: true,
    });

    const status = config.planSummary?.generationStatus;
    if (status === 'completed' || (typeof status === 'object' && status !== null && 'failed' in status)) {
      return config;
    }

    await new Promise((resolve) => setTimeout(resolve, intervalMs));
  }

  throw new Error('Plan generation timed out');
}
