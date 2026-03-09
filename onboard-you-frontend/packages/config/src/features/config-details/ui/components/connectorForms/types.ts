import type { ConnectionForm, ValidationErrors } from '../../../domain';
import type { IConnectorConfig, ConnectorChangeEvent } from '../../../state/connectorConfigs/IConnectorConfig';

export interface ConnectorFormProps {
  form: ConnectionForm;
  errors: ValidationErrors;
  config: IConnectorConfig;
  onChange: (event: ConnectorChangeEvent) => void;
  validateField: (name: string) => void;
}

export type { ConnectorChangeEvent };
