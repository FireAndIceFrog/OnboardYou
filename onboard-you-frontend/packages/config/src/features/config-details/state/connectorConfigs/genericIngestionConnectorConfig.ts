import type { TFunction } from 'i18next';
import { ActionConfig } from "@/shared";
import {
  IConnectorConfig,
  ConnectorChangeEvent,
  ConnectorChangeResult,
  ApplyChangeContext,
} from "./IConnectorConfig";
import { ConnectionForm, ValidationErrors } from "../../domain/types";
import {
  validateGenericFile,
  uploadFileAndStartConversion,
} from '../../services/genericUploadService';

export class GenericIngestionConnectorConfig implements IConnectorConfig {
  getActionConfig(form: ConnectionForm): ActionConfig {
    return {
      id: 'ingest',
      action_type: 'generic_ingestion_connector',
      config: {
        filename: form.genericIngestion.filename,
        // Only include columns when the user has specified overrides.
        // For non-CSV files, columns will be auto-detected by Textract at
        // conversion time; sending an empty array would be incorrect.
        ...(form.genericIngestion.columns.length > 0
          ? { columns: form.genericIngestion.columns }
          : {}),
      },
    };
  }

  getDefaultState(): Partial<ConnectionForm> {
    return {
      genericIngestion: {
        filename: '',
        columns: [],
        uploadStatus: 'idle',
        uploadError: null,
        conversionStatus: null,
      },
    };
  }

  async *applyChange(
    event: ConnectorChangeEvent,
    form: ConnectionForm,
    ctx: ApplyChangeContext,
  ): AsyncGenerator<ConnectorChangeResult> {
    if (event.type === 'field') {
      const updated: ConnectionForm = {
        ...form,
        genericIngestion: { ...form.genericIngestion, [event.key]: event.value },
      };
      yield { form: updated, errors: {} };
      return;
    }

    if (event.type !== 'file') return;

    const validationError = validateGenericFile(event.file);
    if (validationError) {
      yield {
        form: {
          ...form,
          genericIngestion: { ...form.genericIngestion, uploadStatus: 'error', uploadError: validationError },
        },
        errors: { 'genericIngestion.filename': validationError },
      };
      return;
    }

    const uploadingForm: ConnectionForm = {
      ...form,
      genericIngestion: {
        filename: event.file.name,
        columns: [],
        uploadStatus: 'uploading',
        uploadError: null,
        conversionStatus: null,
      },
    };
    yield { form: uploadingForm, errors: { 'genericIngestion.filename': undefined } };

    try {
      const { filename, columns, conversionStatus } = await uploadFileAndStartConversion(
        ctx.companyId!,
        event.file,
      );

      yield {
        form: {
          ...uploadingForm,
          genericIngestion: {
            filename,
            columns,
            uploadStatus: 'done',
            uploadError: null,
            conversionStatus,
          },
        },
        errors: { 'genericIngestion.filename': undefined },
      };
    } catch (err) {
      const message = err instanceof Error ? err.message : 'Upload failed';
      yield {
        form: {
          ...uploadingForm,
          genericIngestion: { ...uploadingForm.genericIngestion, uploadStatus: 'error', uploadError: message },
        },
        errors: { 'genericIngestion.filename': message },
      };
    }
  }

  validate(form: ConnectionForm, t: TFunction): ValidationErrors {
    const errs: ValidationErrors = {};
    if (!form.genericIngestion.filename) {
      errs['genericIngestion.filename'] = t('validation.genericIngestionRequired');
    }
    if (form.genericIngestion.uploadError) {
      errs['genericIngestion.filename'] = form.genericIngestion.uploadError;
    }
    return errs;
  }

  isFormValid(form: ConnectionForm): boolean {
    const { filename, uploadStatus } = form.genericIngestion;
    return !!(filename && uploadStatus === 'done');
  }
}
