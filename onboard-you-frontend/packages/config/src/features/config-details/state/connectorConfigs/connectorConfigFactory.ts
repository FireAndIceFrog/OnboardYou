import { CsvConnectorConfig } from "./csvConnectorConfig";
import { IConnectorConfig } from "./IConnectorConfig";
import { SageHrConnectorConfig } from "./sageHrConnectorConfig";
import { WorkdayConnectorConfig } from "./workdayConnectorConfig";

export enum ConnectorType {
    Workday = "workday",
    SageHR = "sage_hr",
    Csv = "csv"
}

export class ConnectorConfigFactory {
    configs: {[key in ConnectorType]: IConnectorConfig};
    constructor() {
        this.configs = {
            [ConnectorType.Workday]: new WorkdayConnectorConfig(),
            [ConnectorType.SageHR]: new SageHrConnectorConfig(),
            [ConnectorType.Csv]: new CsvConnectorConfig() 
        }
    }

    getConfig(type: ConnectorType): IConnectorConfig {
        return this.configs[type];
    }
}