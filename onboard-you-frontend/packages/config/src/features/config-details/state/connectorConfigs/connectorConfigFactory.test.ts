import { describe, it, expect } from 'vitest';
import { ConnectorConfigFactory, ConnectorType } from './connectorConfigFactory';
import { WorkdayConnectorConfig } from './workdayConnectorConfig';
import { SageHrConnectorConfig } from './sageHrConnectorConfig';
import { CsvConnectorConfig } from './csvConnectorConfig';

describe('ConnectorConfigFactory', () => {
  const factory = new ConnectorConfigFactory();

  interface Case { type: ConnectorType; expected: Function }
  const cases: Case[] = [
    { type: ConnectorType.Workday, expected: WorkdayConnectorConfig },
    { type: ConnectorType.SageHR, expected: SageHrConnectorConfig },
    { type: ConnectorType.Csv, expected: CsvConnectorConfig },
  ];

  it.each(cases)('getConfig($type) returns $expected.name instance', ({ type, expected }) => {
    expect(factory.getConfig(type)).toBeInstanceOf(expected);
  });

  it('returns the same instance on repeated calls', () => {
    expect(factory.getConfig(ConnectorType.Workday)).toBe(factory.getConfig(ConnectorType.Workday));
  });
});
