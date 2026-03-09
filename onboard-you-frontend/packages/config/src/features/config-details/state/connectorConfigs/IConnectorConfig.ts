import type { TFunction } from 'i18next';
import { ActionConfig } from "@/shared";
import { ConnectionForm, ValidationErrors } from "../../domain";

/** Generic event emitted by a connector form field. */
export type ConnectorChangeEvent =
  | { type: 'field'; key: string; value: string }
  | { type: 'toggle'; key: string }
  | { type: 'file'; file: File };

/** Snapshot yielded by applyChange at each stage. */
export interface ConnectorChangeResult {
    form: ConnectionForm;
    errors: ValidationErrors;
}

/** Context passed through to applyChange. */
export interface ApplyChangeContext {
    /** When true, run validation (and for async connectors like CSV, trigger the upload). */
    validate: boolean;
    t: TFunction;
    /** Company ID for connectors that need server-side processing (e.g. CSV upload). */
    companyId?: string;
}

export interface IConnectorConfig {
    getActionConfig(fields: ConnectionForm): ActionConfig;
    getDefaultState(): Partial<ConnectionForm>;
    /**
     * Async generator that applies a change event and yields `{ form, errors }` snapshots.
     * Synchronous connectors yield once. Async connectors (CSV upload) yield
     * at each stage (uploading → done/error).
     *
     * When `ctx.validate` is false, yields the updated form with empty errors
     * (used for keystroke-level changes). When true, also includes validation
     * errors and triggers async side effects.
     */
    applyChange(event: ConnectorChangeEvent, form: ConnectionForm, ctx: ApplyChangeContext): AsyncGenerator<ConnectorChangeResult>;
    /** Return field-level validation errors for this connector's fields. */
    validate(form: ConnectionForm, t: TFunction): ValidationErrors;
    /** True when all required fields for this connector are complete and valid. */
    isFormValid(form: ConnectionForm): boolean;
}