import { csvPresignedUpload } from '@/generated/api';
import { client } from '@/generated/api/client.gen';

const MAX_FILE_SIZE = 50 * 1024 * 1024; // 50 MB

/** Accepted MIME types and extensions for generic ingestion. */
const ACCEPTED_EXTENSIONS = [
  '.csv', '.pdf', '.xml', '.json',
  '.xlsx', '.xls', '.png', '.jpg', '.jpeg', '.tiff', '.tif',
];

export interface StartConversionResponse {
  /** `"not_needed"` — file was already a CSV; columns returned inline.
   *  `"converted"` — file was converted to CSV synchronously; columns returned. */
  status: 'not_needed' | 'converted';
  /** Column names from the converted (or original CSV) file. */
  columns?: string[];
}

/** Validate a file before upload. Returns an error string or null. */
export function validateGenericFile(file: File): string | null {
  const ext = file.name.slice(file.name.lastIndexOf('.')).toLowerCase();
  if (!ACCEPTED_EXTENSIONS.includes(ext)) {
    return `Unsupported file type: ${ext}. Supported types: ${ACCEPTED_EXTENSIONS.join(', ')}`;
  }
  if (file.size > MAX_FILE_SIZE) {
    return `File size must be under ${MAX_FILE_SIZE / 1024 / 1024} MB`;
  }
  return null;
}

/** Returns true if the file is already a CSV (no Textract needed). */
export function isCsvFile(filename: string): boolean {
  return filename.toLowerCase().endsWith('.csv');
}

/**
 * Step 1: Request a presigned PUT URL from the API.
 * Reuses the existing csv-upload endpoint — it accepts any filename.
 */
async function getPresignedUploadUrl(
  customerCompanyId: string,
  filename: string,
): Promise<string> {
  const { data } = await csvPresignedUpload({
    path: { customer_company_id: customerCompanyId },
    query: { filename },
    throwOnError: true,
  });
  return data.upload_url;
}

/**
 * Step 2: Upload the file directly to S3 via the presigned URL.
 * Content-Type is set based on the file's MIME type (or octet-stream fallback).
 */
async function uploadFileToS3(uploadUrl: string, file: File): Promise<void> {
  const contentType = file.type || 'application/octet-stream';
  const res = await fetch(uploadUrl, {
    method: 'PUT',
    headers: { 'Content-Type': contentType },
    body: file,
  });
  if (!res.ok) {
    throw new Error(`S3 upload failed: ${res.status} ${res.statusText}`);
  }
}

/**
 * Step 3: Notify the API that the upload is complete and request conversion.
 *
 * - CSV files: columns returned inline immediately (`status = "not_needed"`).
 * - All other types: Textract job queued (`status = "queued"`).
 */
async function startConversion(
  customerCompanyId: string,
  filename: string,
  tableIndex?: number,
): Promise<StartConversionResponse> {
  const response = await client.post<StartConversionResponse>({
    url: `/config/${customerCompanyId}/start-conversion`,
    body: {
      filename,
      ...(tableIndex != null ? { table_index: tableIndex } : {}),
    },
    security: [{ scheme: 'bearer', type: 'http' }],
  });

  if (!response.data) {
    throw new Error('start-conversion returned no data');
  }
  return response.data;
}

/**
 * Full upload + conversion flow for any file type.
 *
 * Returns:
 * - `{ filename, columns, conversionStatus: 'not_needed' }` for CSVs
 * - `{ filename, columns: [], conversionStatus: 'queued' }` for non-CSV files
 */
export async function uploadFileAndStartConversion(
  customerCompanyId: string,
  file: File,
  tableIndex?: number,
): Promise<{ filename: string; columns: string[]; conversionStatus: StartConversionResponse['status'] }> {
  const uploadUrl = await getPresignedUploadUrl(customerCompanyId, file.name);
  await uploadFileToS3(uploadUrl, file);
  const conversion = await startConversion(customerCompanyId, file.name, tableIndex);

  return {
    filename: file.name,
    columns: conversion.columns ?? [],
    conversionStatus: conversion.status,
  };
}
