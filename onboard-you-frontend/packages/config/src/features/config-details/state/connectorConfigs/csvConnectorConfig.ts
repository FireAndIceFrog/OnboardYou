import type { TFunction } from 'i18next';
import { ActionConfig } from "@/shared";
import { IConnectorConfig, ConnectorChangeEvent, ConnectorChangeResult, ApplyChangeContext } from "./IConnectorConfig";
import { ConnectionForm, ValidationErrors } from "../../domain/types";
import { validateCsvFile, uploadCsvAndDiscoverColumns } from '../../services/csvUploadService';

export class CsvConnectorConfig implements IConnectorConfig {
    getActionConfig(form: ConnectionForm): ActionConfig {
        return {
            id: 'ingest',
            action_type: 'csv_hris_connector',
            config: {
                filename: form.csv.filename,
                columns: form.csv.columns,
            },
        };
    }

    getDefaultState(): Partial<ConnectionForm> {
        return {
            csv: {
                filename: '',
                columns: [],
                uploadStatus: 'idle',
                uploadError: null,
            },
        };
    }

    async *applyChange(event: ConnectorChangeEvent, form: ConnectionForm, ctx: ApplyChangeContext): AsyncGenerator<ConnectorChangeResult> {
        if (event.type !== 'file') return;

        const validationError = validateCsvFile(event.file);
        if (validationError) {
            yield {
                form: { ...form, csv: { ...form.csv, uploadStatus: 'error', uploadError: validationError } },
                errors: { 'csv.filename': validationError },
            };
            return;
        }

        const uploadingForm: ConnectionForm = {
            ...form,
            csv: { filename: event.file.name, columns: [], uploadStatus: 'uploading', uploadError: null },
        };
        yield { form: uploadingForm, errors: { 'csv.filename': undefined } };

        try {
            const { filename, columns } = await uploadCsvAndDiscoverColumns(ctx.companyId!, event.file);
            yield {
                form: { ...uploadingForm, csv: { filename, columns, uploadStatus: 'done', uploadError: null } },
                errors: { 'csv.filename': undefined },
            };
        } catch (err) {
            const message = err instanceof Error ? err.message : 'Upload failed';
            yield {
                form: { ...uploadingForm, csv: { ...uploadingForm.csv, uploadStatus: 'error', uploadError: message } },
                errors: { 'csv.filename': message },
            };
        }
    }

    validate(form: ConnectionForm, t: TFunction): ValidationErrors {
        const errs: ValidationErrors = {};
        if (!form.csv.filename) errs['csv.filename'] = t('validation.csvRequired');
        if (form.csv.uploadError) errs['csv.filename'] = form.csv.uploadError;
        return errs;
    }

    isFormValid(form: ConnectionForm): boolean {
        return !!(
            form.csv.filename &&
            form.csv.columns.length > 0 &&
            form.csv.uploadStatus === 'done'
        );
    }
}