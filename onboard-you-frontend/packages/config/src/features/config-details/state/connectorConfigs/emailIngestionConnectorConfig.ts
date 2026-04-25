import type { TFunction } from 'i18next';
import { ActionConfig } from "@/shared";
import { ConnectionForm, ValidationErrors } from "../../domain/types";
import { IConnectorConfig, ConnectorChangeEvent, ConnectorChangeResult, ApplyChangeContext } from "./IConnectorConfig";

export class EmailIngestionConnectorConfig implements IConnectorConfig {
    getActionConfig(form: ConnectionForm): ActionConfig {
        const allowedSenders = form.emailIngestion.allowedSenders
            .split(',')
            .map((s) => s.trim())
            .filter(Boolean);

        return {
            id: 'ingest',
            action_type: 'email_ingestion_connector',
            config: {
                allowed_senders: allowedSenders,
                ...(form.emailIngestion.subjectFilter.trim()
                    ? { subject_filter: form.emailIngestion.subjectFilter.trim() }
                    : {}),
            },
        };
    }

    getDefaultState(): Partial<ConnectionForm> {
        return {
            emailIngestion: {
                allowedSenders: '',
                subjectFilter: '',
            },
        };
    }

    async *applyChange(
        event: ConnectorChangeEvent,
        form: ConnectionForm,
        ctx: ApplyChangeContext,
    ): AsyncGenerator<ConnectorChangeResult> {
        if (event.type !== 'field') return;

        const updated: ConnectionForm = {
            ...form,
            emailIngestion: { ...form.emailIngestion, [event.key]: event.value },
        };
        yield { form: updated, errors: ctx.validate ? this.validate(updated, ctx.t) : {} };
    }

    validate(form: ConnectionForm, t: TFunction): ValidationErrors {
        const errs: ValidationErrors = {};
        const raw = form.emailIngestion.allowedSenders.trim();

        if (!raw) {
            errs['emailIngestion.allowedSenders'] = t('validation.required');
            return errs;
        }

        const entries = raw.split(',').map((s) => s.trim()).filter(Boolean);
        if (entries.length === 0) {
            errs['emailIngestion.allowedSenders'] = t('validation.required');
        } else {
            for (const entry of entries) {
                // Must be a valid email address or a domain glob (@domain.com)
                const isEmail = /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(entry);
                const isDomainGlob = /^@[^\s@]+\.[^\s@]+$/.test(entry);
                if (!isEmail && !isDomainGlob) {
                    errs['emailIngestion.allowedSenders'] = t('validation.invalidSender', {
                        sender: entry,
                        defaultValue: `Invalid sender: "${entry}". Use an email address or @domain.com`,
                    });
                    break;
                }
            }
        }

        return errs;
    }

    isFormValid(form: ConnectionForm): boolean {
        const raw = form.emailIngestion.allowedSenders.trim();
        if (!raw) return false;
        const entries = raw.split(',').map((s) => s.trim()).filter(Boolean);
        if (entries.length === 0) return false;
        return entries.every((entry) => {
            const isEmail = /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(entry);
            const isDomainGlob = /^@[^\s@]+\.[^\s@]+$/.test(entry);
            return isEmail || isDomainGlob;
        });
    }
}
