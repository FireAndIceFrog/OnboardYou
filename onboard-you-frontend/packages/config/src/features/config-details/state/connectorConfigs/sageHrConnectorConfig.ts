import type { TFunction } from 'i18next';
import { ActionConfig } from "@/shared";
import { ConnectionForm, SageHrFields, ValidationErrors } from "../../domain";
import { IConnectorConfig, ConnectorChangeEvent, ConnectorChangeResult, ApplyChangeContext } from "./IConnectorConfig";

export class SageHrConnectorConfig implements IConnectorConfig {
    getActionConfig(form: ConnectionForm): ActionConfig {
        // Implement the logic to return the ActionConfig for CSV connector
        return {
            id: 'ingest',
            action_type: 'sage_hr_connector',
            config: {
                subdomain: form.sageHr.subdomain.trim(),
                api_token: form.sageHr.apiToken,
                include_team_history: form.sageHr.includeTeamHistory || undefined,
                include_employment_status_history: form.sageHr.includeEmploymentStatusHistory || undefined,
                include_position_history: form.sageHr.includePositionHistory || undefined,
            }
        };
    }

    getDefaultState(): Partial<ConnectionForm> {
        return {
            sageHr: {
                subdomain: '',
                apiToken: '',
                includeTeamHistory: false,
                includeEmploymentStatusHistory: false,
                includePositionHistory: false,
            },
        };
    }

    async *applyChange(event: ConnectorChangeEvent, form: ConnectionForm, ctx: ApplyChangeContext): AsyncGenerator<ConnectorChangeResult> {
        let next = form;
        if (event.type === 'field') {
            next = { ...form, sageHr: { ...form.sageHr, [event.key]: event.value } };
        } else if (event.type === 'toggle') {
            next = { ...form, sageHr: { ...form.sageHr, [event.key]: !form.sageHr[event.key as keyof SageHrFields] } };
        }
        yield { form: next, errors: ctx.validate ? this.validate(next, ctx.t) : {} };
    }

    validate(form: ConnectionForm, t: TFunction): ValidationErrors {
        const errs: ValidationErrors = {};
        const s = form.sageHr;
        if (!s.subdomain.trim()) errs['sageHr.subdomain'] = t('validation.required');
        if (!s.apiToken) {
            errs['sageHr.apiToken'] = t('validation.required');
        } else if (s.apiToken.length < 8) {
            errs['sageHr.apiToken'] = t('validation.minLength', { min: 8 });
        }
        return errs;
    }

    isFormValid(form: ConnectionForm): boolean {
        const s = form.sageHr;
        return !!(s.subdomain.trim() && s.apiToken && s.apiToken.length >= 8);
    }
}