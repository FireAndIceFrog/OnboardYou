import { describe, it, expect, vi, beforeEach } from 'vitest';
import { screen, fireEvent, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { renderWithProviders } from '@/shared/test/testWrapper';
import { CsvConnectorPanel } from './CsvConnectorPanel';

/* ── Mocks ─────────────────────────────────────────────────── */

vi.mock('react-router-dom', async () => {
  const actual = await vi.importActual<typeof import('react-router-dom')>('react-router-dom');
  return {
    ...actual,
    useParams: () => ({ customerCompanyId: 'test-company' }),
  };
});

vi.mock('../../../services/csvUploadService', () => ({
  validateCsvFile: vi.fn(),
  uploadCsvAndDiscoverColumns: vi.fn(),
}));

import {
  validateCsvFile,
  uploadCsvAndDiscoverColumns,
} from '../../../services/csvUploadService';

const mockValidate = vi.mocked(validateCsvFile);
const mockUpload = vi.mocked(uploadCsvAndDiscoverColumns);

/* ── Helpers ───────────────────────────────────────────────── */

function csvFile(name = 'data.csv') {
  return new File(['col1,col2\na,b'], name, { type: 'text/csv' });
}

function renderPanel(config: Record<string, unknown> = {}) {
  const onChange = vi.fn();
  renderWithProviders(
    <CsvConnectorPanel config={config} onChange={onChange} availableColumns={[]} />,
  );
  return { onChange };
}

/* ── Tests ─────────────────────────────────────────────────── */

beforeEach(() => {
  vi.clearAllMocks();
  mockValidate.mockReturnValue(null);
  mockUpload.mockResolvedValue({ filename: 'data.csv', columns: ['col1', 'col2'] });
});

describe('CsvConnectorPanel', () => {
  it('renders the panel container', () => {
    renderPanel();
    expect(screen.getByTestId('csv-connector-panel')).toBeInTheDocument();
  });

  /* ── Current file display ────────────────────────────────── */

  describe('current file info', () => {
    it.each([
      ['does not show filename when config has none', {}, false],
      ['shows filename from config', { filename: 'employees.csv' }, true],
    ])('%s', (_label, config, visible) => {
      renderPanel(config);
      if (visible) {
        expect(screen.getByText('employees.csv')).toBeInTheDocument();
      } else {
        expect(screen.queryByText('employees.csv')).not.toBeInTheDocument();
      }
    });

    it('shows discovered column chips', () => {
      const columns = ['name', 'email', 'role'];
      renderPanel({ filename: 'staff.csv', columns });
      columns.forEach((col) => {
        expect(screen.getByText(col)).toBeInTheDocument();
      });
    });

    it('shows no column chips when columns array is empty', () => {
      renderPanel({ filename: 'empty.csv', columns: [] });
      expect(screen.getByText('empty.csv')).toBeInTheDocument();
    });
  });

  /* ── Button label ────────────────────────────────────────── */

  describe('action button label', () => {
    it.each([
      ['Choose CSV file', {}],
      ['Replace CSV file', { filename: 'data.csv', columns: ['a'] }],
    ])('shows "%s" when config is %o', (expectedText, config) => {
      renderPanel(config);
      expect(screen.getByRole('button', { name: expectedText })).toBeInTheDocument();
    });
  });

  /* ── File input interaction ──────────────────────────────── */

  describe('file input upload', () => {
    it('calls onChange with filename and columns on successful upload', async () => {
      const { onChange } = renderPanel();
      const input = document.querySelector('input[type="file"]') as HTMLInputElement;

      fireEvent.change(input, { target: { files: [csvFile()] } });

      await waitFor(() => {
        expect(mockUpload).toHaveBeenCalledWith('test-company', expect.any(File));
      });
      await waitFor(() => {
        expect(onChange).toHaveBeenCalledWith('filename', 'data.csv');
        expect(onChange).toHaveBeenCalledWith('columns', ['col1', 'col2']);
      });
    });

    it('shows success feedback after upload', async () => {
      renderPanel();
      const input = document.querySelector('input[type="file"]') as HTMLInputElement;

      fireEvent.change(input, { target: { files: [csvFile()] } });

      await waitFor(() => {
        expect(screen.getByTestId('csv-upload-success')).toBeInTheDocument();
      });
    });
  });

  /* ── Validation errors ───────────────────────────────────── */

  describe('client-side validation', () => {
    it.each([
      ['invalid extension', 'Only .csv files are supported'],
      ['file too large', 'File size must be under 50MB'],
    ])('shows error for %s', async (_label, errorMessage) => {
      mockValidate.mockReturnValue(errorMessage);
      renderPanel();
      const input = document.querySelector('input[type="file"]') as HTMLInputElement;

      fireEvent.change(input, { target: { files: [csvFile()] } });

      await waitFor(() => {
        expect(screen.getByTestId('csv-upload-error')).toHaveTextContent(errorMessage);
      });
      expect(mockUpload).not.toHaveBeenCalled();
    });
  });

  /* ── Upload failure ──────────────────────────────────────── */

  describe('upload failure', () => {
    it.each([
      ['Error instance', new Error('S3 upload failed: 403'), 'S3 upload failed: 403'],
      ['non-Error throw', 'something broke', 'Upload failed'],
    ])('shows error message for %s', async (_label, rejection, expected) => {
      mockUpload.mockRejectedValue(rejection);
      const { onChange } = renderPanel();
      const input = document.querySelector('input[type="file"]') as HTMLInputElement;

      fireEvent.change(input, { target: { files: [csvFile()] } });

      await waitFor(() => {
        expect(screen.getByTestId('csv-upload-error')).toHaveTextContent(expected);
      });
      expect(onChange).not.toHaveBeenCalled();
    });
  });

  /* ── Drag and drop ───────────────────────────────────────── */

  describe('drag and drop', () => {
    it('uploads file on drop', async () => {
      const { onChange } = renderPanel();
      const dropZone = screen.getByText('📂').parentElement!;
      const file = csvFile();

      fireEvent.drop(dropZone, {
        dataTransfer: { files: [file] },
      });

      await waitFor(() => {
        expect(mockUpload).toHaveBeenCalledWith('test-company', file);
      });
      await waitFor(() => {
        expect(onChange).toHaveBeenCalledWith('filename', 'data.csv');
        expect(onChange).toHaveBeenCalledWith('columns', ['col1', 'col2']);
      });
    });
  });

  /* ── Does not call onChange when no file selected ─────────── */

  it('does nothing when input change has no files', () => {
    const { onChange } = renderPanel();
    const input = document.querySelector('input[type="file"]') as HTMLInputElement;

    fireEvent.change(input, { target: { files: [] } });

    expect(mockUpload).not.toHaveBeenCalled();
    expect(onChange).not.toHaveBeenCalled();
  });
});
