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
 * Returns both the presigned URL and the **server-assigned** timestamped
 * filename (e.g. `"employees_20260425T143000Z.pdf"`).  The caller must use
 * this canonical name — not the local `File.name` — for subsequent calls.
 */
async function getPresignedUploadUrl(
  customerCompanyId: string,
  filename: string,
): Promise<{ uploadUrl: string; serverFilename: string }> {
  const { data } = await csvPresignedUpload({
    path: { customer_company_id: customerCompanyId },
    query: { filename },
    throwOnError: true,
  });
  return { uploadUrl: data.upload_url, serverFilename: data.filename };
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
 *
 * Can also be called independently to re-convert an already-uploaded file
 * (e.g., before triggering a pipeline run to ensure the CSV is present).
 */
export async function startConversion(
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
 * - `{ filename, columns: [], conversionStatus: 'converted' }` for non-CSV files
 *
 * `filename` is always the **server-assigned** timestamped name — not the
 * original local filename.  Store this in the manifest config so the ETL
 * pipeline reads the correct S3 key.
 */
export async function uploadFileAndStartConversion(
  customerCompanyId: string,
  file: File,
  tableIndex?: number,
): Promise<{ filename: string; columns: string[]; conversionStatus: StartConversionResponse['status'] }> {
  const { uploadUrl, serverFilename } = await getPresignedUploadUrl(customerCompanyId, file.name);
  await uploadFileToS3(uploadUrl, file);
  const conversion = await startConversion(customerCompanyId, serverFilename, tableIndex);

  return {
    filename: serverFilename,
    columns: conversion.columns ?? [],
    conversionStatus: conversion.status,
  };
}
