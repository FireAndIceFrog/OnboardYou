import type { TFunction } from 'i18next';
import { ActionConfig } from "@/shared";
import { ConnectionForm, ValidationErrors } from "../../domain";
import { IConnectorConfig, ConnectorChangeEvent, ConnectorChangeResult, ApplyChangeContext } from "./IConnectorConfig";

const URL_RE = /^https?:\/\/.+/i;

export class WorkdayConnectorConfig implements IConnectorConfig{
    getActionConfig(form: ConnectionForm): ActionConfig {
        // Implement the logic to return the ActionConfig for Workday connector
        const active = new Set(form.workday.responseGroup.split(',').filter(Boolean));
        
        return {
            id: 'ingest',
            action_type: 'workday_hris_connector',
            config: {
                tenant_url: form.workday.tenantUrl,
                tenant_id: form.workday.tenantId,
                username: form.workday.username,
                password: form.workday.password,
                worker_count_limit: Number(form.workday.workerCountLimit) || 200,
                response_group: {
                    include_personal_information: active.has('include_personal_information'),
                    include_employment_information: active.has('include_employment_information'),
                    include_compensation: active.has('include_compensation'),
                    include_organizations: active.has('include_organizations'),
                    include_roles: active.has('include_roles'),
                },
            },
        };
    }

    getDefaultState(): Partial<ConnectionForm> {
        return {
            workday: {
                tenantUrl: '',
                tenantId: '',
                username: '',
                password: '',
                workerCountLimit: '200',
                responseGroup: 'include_personal_information,include_employment_information',
            }
        };
    }

    async *applyChange(event: ConnectorChangeEvent, form: ConnectionForm, ctx: ApplyChangeContext): AsyncGenerator<ConnectorChangeResult> {
        let next = form;
        if (event.type === 'field') {
            next = { ...form, workday: { ...form.workday, [event.key]: event.value } };
        } else if (event.type === 'toggle') {
            const current = form.workday.responseGroup.split(',').filter(Boolean);
            const items = current.includes(event.key)
                ? current.filter(v => v !== event.key)
                : [...current, event.key];
            next = { ...form, workday: { ...form.workday, responseGroup: items.join(',') } };
        }
        yield { form: next, errors: ctx.validate ? this.validate(next, ctx.t) : {} };
    }

    validate(form: ConnectionForm, t: TFunction): ValidationErrors {
        const errs: ValidationErrors = {};
        const w = form.workday;
        if (!w.tenantUrl.trim()) {
            errs['workday.tenantUrl'] = t('validation.required');
        } else if (!URL_RE.test(w.tenantUrl.trim())) {
            errs['workday.tenantUrl'] = t('validation.invalidUrl');
        }
        if (!w.username.trim()) errs['workday.username'] = t('validation.required');
        if (!w.password) {
            errs['workday.password'] = t('validation.required');
        } else if (w.password.length < 8) {
            errs['workday.password'] = t('validation.minLength', { min: 8 });
        }
        if (!w.tenantId.trim()) errs['workday.tenantId'] = t('validation.required');
        const groups = w.responseGroup.split(',').filter(Boolean);
        if (groups.length === 0) errs['workday.responseGroup'] = t('validation.selectResponseGroup');
        return errs;
    }

    isFormValid(form: ConnectionForm): boolean {
        const w = form.workday;
        const groups = w.responseGroup.split(',').filter(Boolean);
        return !!(
            w.tenantUrl && URL_RE.test(w.tenantUrl.trim()) &&
            w.tenantId && w.username &&
            w.password && w.password.length >= 8 &&
            groups.length > 0
        );
    }
}