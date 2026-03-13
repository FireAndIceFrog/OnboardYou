import {
  listRuns as listRunsApi,
  getRun as getRunApi,
} from '@/generated/api';
import type { PipelineRun, ListResponsePipelineRun } from '@/generated/api';

export async function fetchRuns(
  customerCompanyId: string,
  page = 1,
  countPerPage = 20,
): Promise<ListResponsePipelineRun> {
  const { data } = await listRunsApi({
    path: { customer_company_id: customerCompanyId },
    query: { page, count_per_page: countPerPage },
    throwOnError: true,
  });
  return data;
}

export async function fetchRun(
  customerCompanyId: string,
  runId: string,
): Promise<PipelineRun> {
  const { data } = await getRunApi({
    path: { customer_company_id: customerCompanyId, run_id: runId },
    throwOnError: true,
  });
  return data;
}
