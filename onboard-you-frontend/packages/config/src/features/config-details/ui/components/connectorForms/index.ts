import type { ComponentType } from 'react';
import { ConnectorType } from '../../../state/connectorConfigs/connectorConfigFactory';
import type { ConnectorFormProps } from './types';
import { WorkdayConnectorForm } from './WorkdayConnectorForm';
import { SageHrConnectorForm } from './SageHrConnectorForm';
import { CsvConnectorForm } from './CsvConnectorForm';
import { GenericIngestionConnectorForm } from './GenericIngestionConnectorForm';

const CONNECTOR_FORM_REGISTRY: Record<ConnectorType, ComponentType<ConnectorFormProps>> = {
  [ConnectorType.Workday]: WorkdayConnectorForm,
  [ConnectorType.SageHR]: SageHrConnectorForm,
  [ConnectorType.Csv]: CsvConnectorForm,
  [ConnectorType.GenericIngestion]: GenericIngestionConnectorForm,
};

/**
 * Look up the form component for a given connector type.
 * Returns `null` if no connector is selected yet.
 */
export function getConnectorFormComponent(
  type: ConnectorType | null | undefined,
): ComponentType<ConnectorFormProps> | null {
  if (!type) return null;
  return CONNECTOR_FORM_REGISTRY[type] ?? null;
}

export type { ConnectorFormProps };
export { WorkdayConnectorForm, SageHrConnectorForm, CsvConnectorForm, GenericIngestionConnectorForm };
