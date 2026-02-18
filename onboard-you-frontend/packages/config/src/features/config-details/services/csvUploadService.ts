import {
  csvPresignedUpload,
  csvColumns,
} from '@/generated/api';
import type { CsvColumnsResponse, PresignedUploadResponse } from '@/generated/api';

const MAX_CSV_SIZE = 50 * 1024 * 1024; // 50 MB
const ALLOWED_EXTENSIONS = ['.csv'];

/** Validate the selected file before uploading. */
export function validateCsvFile(file: File): string | null {
  const ext = file.name.slice(file.name.lastIndexOf('.')).toLowerCase();
  if (!ALLOWED_EXTENSIONS.includes(ext)) {
    return 'Only .csv files are supported';
  }
  if (file.size > MAX_CSV_SIZE) {
    return `File size must be under ${MAX_CSV_SIZE / 1024 / 1024}MB`;
  }
  return null;
}

/**
 * Step 1: Request a presigned PUT URL from the API.
 */
export async function getPresignedUploadUrl(
  customerCompanyId: string,
  filename: string,
): Promise<PresignedUploadResponse> {
  const { data } = await csvPresignedUpload({
    path: { customer_company_id: customerCompanyId },
    query: { filename },
    throwOnError: true,
  });
  return data;
}

/**
 * Step 2: Upload the CSV directly to S3 via the presigned URL.
 */
export async function uploadCsvToS3(
  uploadUrl: string,
  file: File,
): Promise<void> {
  const res = await fetch(uploadUrl, {
    method: 'PUT',
    headers: { 'Content-Type': 'text/csv' },
    body: file,
  });
  if (!res.ok) {
    throw new Error(`S3 upload failed: ${res.status} ${res.statusText}`);
  }
}

/**
 * Step 3: Ask the API to read the CSV headers and return column names.
 */
export async function fetchCsvColumns(
  customerCompanyId: string,
  filename: string,
): Promise<CsvColumnsResponse> {
  const { data } = await csvColumns({
    path: { customer_company_id: customerCompanyId },
    query: { filename },
    throwOnError: true,
  });
  return data;
}

/**
 * Orchestrates the full upload flow: presigned URL → S3 PUT → column discovery.
 * Returns the discovered columns on success.
 */
export async function uploadCsvAndDiscoverColumns(
  customerCompanyId: string,
  file: File,
): Promise<{ filename: string; columns: string[] }> {
  const { upload_url } = await getPresignedUploadUrl(customerCompanyId, file.name);
  await uploadCsvToS3(upload_url, file);
  const { columns } = await fetchCsvColumns(customerCompanyId, file.name);
  return { filename: file.name, columns };
}
