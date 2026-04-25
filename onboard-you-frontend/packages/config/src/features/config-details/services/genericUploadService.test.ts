import { describe, it, expect, vi, beforeEach } from 'vitest';
import { validateGenericFile, isCsvFile, uploadFileAndStartConversion } from './genericUploadService';

// ---------------------------------------------------------------------------
// validateGenericFile — table-driven
// ---------------------------------------------------------------------------

describe('validateGenericFile', () => {
  const MAX_MB = 50;
  const OVER_LIMIT = (MAX_MB + 1) * 1024 * 1024;

  interface ValidCase { name: string; size: number }
  const valid: ValidCase[] = [
    { name: 'roster.csv',  size: 100 },
    { name: 'report.pdf',  size: 1024 },
    { name: 'data.xml',    size: 512 },
    { name: 'image.png',   size: 2048 },
    { name: 'sheet.xlsx',  size: 4096 },
  ];

  it.each(valid)('accepts "$name"', ({ name, size }) => {
    const file = new File(['x'.repeat(size)], name);
    expect(validateGenericFile(file)).toBeNull();
  });

  interface InvalidCase { name: string; size: number; reason: string }
  const invalid: InvalidCase[] = [
    { name: 'data.txt',  size: 100,       reason: 'unsupported extension' },
    { name: 'run.exe',   size: 100,       reason: 'unsupported extension' },
    { name: 'file.csv',  size: OVER_LIMIT, reason: 'exceeds size limit' },
  ];

  it.each(invalid)('rejects "$name" ($reason)', ({ name, size }) => {
    const file = new File(['x'.repeat(Math.min(size, 100))], name);
    Object.defineProperty(file, 'size', { value: size });
    expect(validateGenericFile(file)).not.toBeNull();
  });
});

// ---------------------------------------------------------------------------
// isCsvFile — table-driven
// ---------------------------------------------------------------------------

describe('isCsvFile', () => {
  interface Case { filename: string; expected: boolean }
  const cases: Case[] = [
    { filename: 'employees.csv',  expected: true },
    { filename: 'DATA.CSV',       expected: true },
    { filename: 'report.pdf',     expected: false },
    { filename: 'data.xml',       expected: false },
    { filename: 'noextension',    expected: false },
  ];

  it.each(cases)('isCsvFile("$filename") → $expected', ({ filename, expected }) => {
    expect(isCsvFile(filename)).toBe(expected);
  });
});

// ---------------------------------------------------------------------------
// uploadFileAndStartConversion — uses server-returned filename
// ---------------------------------------------------------------------------

// Mock the generated API client and internal fetch
vi.mock('@/generated/api', () => ({
  csvPresignedUpload: vi.fn(),
}));

describe('uploadFileAndStartConversion', () => {
  interface Case {
    localName: string;
    serverFilename: string;
    conversionStatus: 'not_needed' | 'converted';
    columns: string[];
  }

  const cases: Case[] = [
    {
      localName: 'employees.pdf',
      serverFilename: 'employees_20260425T143000Z.pdf',
      conversionStatus: 'converted',
      columns: ['id', 'name'],
    },
    {
      localName: 'roster.csv',
      serverFilename: 'roster_20260425T143000Z.csv',
      conversionStatus: 'not_needed',
      columns: ['emp_id', 'email'],
    },
    {
      localName: 'data.xml',
      serverFilename: 'data_20260425T143000Z.xml',
      conversionStatus: 'converted',
      columns: [],
    },
  ];

  beforeEach(() => {
    vi.restoreAllMocks();
  });

  it.each(cases)(
    'uses server filename "$serverFilename" (not local "$localName")',
    async ({ localName, serverFilename, conversionStatus, columns }) => {
      const { csvPresignedUpload } = await import('@/generated/api');
      vi.mocked(csvPresignedUpload).mockResolvedValue({
        data: { upload_url: 'https://s3.example.com/put', filename: serverFilename, key: `org/co/${serverFilename}` },
      } as any);

      // Mock fetch for S3 PUT
      global.fetch = vi.fn().mockResolvedValue({ ok: true } as Response);

      // Mock the start-conversion call via the client
      const { client } = await import('@/generated/api/client.gen');
      vi.spyOn(client as any, 'post').mockResolvedValue({
        data: { status: conversionStatus, columns },
      });

      const file = new File(['content'], localName);
      const result = await uploadFileAndStartConversion('company-1', file);

      // Critical: the returned filename must be the server-assigned one
      expect(result.filename).toBe(serverFilename);
      expect(result.conversionStatus).toBe(conversionStatus);
      expect(result.columns).toEqual(columns);
    },
  );
});
