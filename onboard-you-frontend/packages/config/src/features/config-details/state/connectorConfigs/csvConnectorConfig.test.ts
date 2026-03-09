import { describe, it, expect, vi, beforeEach } from 'vitest';
import type { ConnectionForm } from '../../domain/types';
import type { ApplyChangeContext, ConnectorChangeResult } from './IConnectorConfig';
import { CsvConnectorConfig } from './csvConnectorConfig';

vi.mock('../../services/csvUploadService', () => ({
  validateCsvFile: vi.fn(),
  uploadCsvAndDiscoverColumns: vi.fn(),
}));

import { validateCsvFile, uploadCsvAndDiscoverColumns } from '../../services/csvUploadService';

const t = (key: string) => key;
const ctx = (validate = false): ApplyChangeContext => ({ validate, t: t as any, companyId: 'co-1' });

async function collect(gen: AsyncGenerator<ConnectorChangeResult>): Promise<ConnectorChangeResult[]> {
  const results: ConnectorChangeResult[] = [];
  for await (const r of gen) results.push(r);
  return results;
}

function baseForm(): ConnectionForm {
  return {
    system: 'csv' as any,
    displayName: 'Test',
    workday: { tenantUrl: '', tenantId: '', username: '', password: '', workerCountLimit: '200', responseGroup: 'include_personal_information,include_employment_information' },
    sageHr: { subdomain: '', apiToken: '', includeTeamHistory: false, includeEmploymentStatusHistory: false, includePositionHistory: false },
    csv: { filename: '', columns: [], uploadStatus: 'idle' as const, uploadError: null },
  };
}

describe('CsvConnectorConfig', () => {
  const config = new CsvConnectorConfig();
  const file = new File(['a,b\n1,2'], 'data.csv', { type: 'text/csv' });

  beforeEach(() => {
    vi.mocked(validateCsvFile).mockReturnValue(null);
    vi.mocked(uploadCsvAndDiscoverColumns).mockResolvedValue({ filename: 'data.csv', columns: ['a', 'b'] });
  });

  describe('applyChange', () => {
    it('happy path: yields uploading then done', async () => {
      const results = await collect(config.applyChange({ type: 'file', file }, baseForm(), ctx(true)));

      expect(results).toHaveLength(2);
      expect(results[0].form.csv.uploadStatus).toBe('uploading');
      expect(results[0].form.csv.filename).toBe('data.csv');
      expect(results[1].form.csv.uploadStatus).toBe('done');
      expect(results[1].form.csv.columns).toEqual(['a', 'b']);
    });

    it('upload failure: yields uploading then error', async () => {
      vi.mocked(uploadCsvAndDiscoverColumns).mockRejectedValue(new Error('Network error'));

      const results = await collect(config.applyChange({ type: 'file', file }, baseForm(), ctx(true)));

      expect(results).toHaveLength(2);
      expect(results[0].form.csv.uploadStatus).toBe('uploading');
      expect(results[1].form.csv.uploadStatus).toBe('error');
      expect(results[1].errors['csv.filename']).toBe('Network error');
    });

    it('invalid file: yields single error without uploading', async () => {
      vi.mocked(validateCsvFile).mockReturnValue('File too large');

      const results = await collect(config.applyChange({ type: 'file', file }, baseForm(), ctx(true)));

      expect(results).toHaveLength(1);
      expect(results[0].form.csv.uploadStatus).toBe('error');
      expect(results[0].errors['csv.filename']).toBe('File too large');
    });

    it('non-file events yield nothing', async () => {
      const results = await collect(config.applyChange({ type: 'field', key: 'x', value: 'y' }, baseForm(), ctx()));
      expect(results).toHaveLength(0);
    });
  });

  describe('validate', () => {
    it('returns error when filename is empty', () => {
      const errs = config.validate(baseForm(), t as any);
      expect(errs['csv.filename']).toBeDefined();
    });

    it('returns no errors when filename is set and no upload error', () => {
      const form = baseForm();
      form.csv = { ...form.csv, filename: 'data.csv', columns: ['a'], uploadStatus: 'done' };
      const errs = config.validate(form, t as any);
      expect(Object.keys(errs)).toHaveLength(0);
    });
  });

  describe('isFormValid', () => {
    it('returns false when no file uploaded', () => {
      expect(config.isFormValid(baseForm())).toBe(false);
    });

    it('returns true when file uploaded and columns discovered', () => {
      const form = baseForm();
      form.csv = { filename: 'data.csv', columns: ['a', 'b'], uploadStatus: 'done', uploadError: null };
      expect(config.isFormValid(form)).toBe(true);
    });
  });
});
