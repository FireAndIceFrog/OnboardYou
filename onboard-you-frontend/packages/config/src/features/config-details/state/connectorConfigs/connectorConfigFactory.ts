import { CsvConnectorConfig } from "./csvConnectorConfig";
import { GenericIngestionConnectorConfig } from "./genericIngestionConnectorConfig";
import { IConnectorConfig } from "./IConnectorConfig";
import { SageHrConnectorConfig } from "./sageHrConnectorConfig";
import { WorkdayConnectorConfig } from "./workdayConnectorConfig";

export enum ConnectorType {
    Workday = "workday",
    SageHR = "sage_hr",
    Csv = "csv",
    GenericIngestion = "generic_ingestion"
}

export class ConnectorConfigFactory {
    configs: {[key in ConnectorType]: IConnectorConfig};
    constructor() {
        this.configs = {
            [ConnectorType.Workday]: new WorkdayConnectorConfig(),
            [ConnectorType.SageHR]: new SageHrConnectorConfig(),
            [ConnectorType.Csv]: new CsvConnectorConfig(),
            [ConnectorType.GenericIngestion]: new GenericIngestionConnectorConfig(),
        }
    }

    getConfig(type: ConnectorType): IConnectorConfig {
        return this.configs[type];
    }
}