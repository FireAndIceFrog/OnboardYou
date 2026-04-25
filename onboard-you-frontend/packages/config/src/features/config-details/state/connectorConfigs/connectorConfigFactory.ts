import { EmailIngestionConnectorConfig } from "./emailIngestionConnectorConfig";
import { GenericIngestionConnectorConfig } from "./genericIngestionConnectorConfig";
import { IConnectorConfig } from "./IConnectorConfig";
import { SageHrConnectorConfig } from "./sageHrConnectorConfig";
import { WorkdayConnectorConfig } from "./workdayConnectorConfig";

export enum ConnectorType {
    Workday = "workday",
    SageHR = "sage_hr",
    GenericIngestion = "generic_ingestion",
    EmailIngestion = "email_ingestion",
}

export class ConnectorConfigFactory {
    configs: {[key in ConnectorType]: IConnectorConfig};
    constructor() {
        this.configs = {
            [ConnectorType.Workday]: new WorkdayConnectorConfig(),
            [ConnectorType.SageHR]: new SageHrConnectorConfig(),
            [ConnectorType.GenericIngestion]: new GenericIngestionConnectorConfig(),
            [ConnectorType.EmailIngestion]: new EmailIngestionConnectorConfig(),
        }
    }

    getConfig(type: ConnectorType): IConnectorConfig {
        return this.configs[type];
    }
}