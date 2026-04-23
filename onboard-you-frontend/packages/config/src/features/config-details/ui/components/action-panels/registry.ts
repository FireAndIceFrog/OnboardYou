import type { ComponentType } from 'react';
import type { ActionType } from '@/generated/api';
import { CsvConnectorPanel } from './CsvConnectorPanel';
import { GenericIngestionConnectorPanel } from './GenericIngestionConnectorPanel';
import { PiiMaskingPanel } from './PiiMaskingPanel';
import { WorkdayResponseGroupPanel } from './WorkdayResponseGroupPanel';
import { SageHrHistoryPanel } from './SageHrHistoryPanel';
import { ApiDispatcherPanel } from './ApiDispatcherPanel';

/**
 * Props that every custom action panel receives.
 * Panels registered here get full control of the edit area for their action type.
 */
export interface ActionEditorProps {
  config: Record<string, unknown>;
  onChange: (key: string, value: unknown) => void;
  availableColumns: string[];
}

/**
 * Registry of custom per-action editor panels.
 *
 * Most actions use the generic field-schema-driven renderer (`FieldEditor`).
 * Actions that need a richer editing experience register a custom panel here.
 *
 * To add a new custom panel:
 * 1. Create a component that implements `ActionEditorProps`
 * 2. Place it in this directory (`action-panels/`)
 * 3. Add one line to `ACTION_PANEL_REGISTRY` below
 */
const ACTION_PANEL_REGISTRY: Partial<Record<ActionType, ComponentType<ActionEditorProps>>> = {
  csv_hris_connector: CsvConnectorPanel,
  generic_ingestion_connector: GenericIngestionConnectorPanel,
  pii_masking: PiiMaskingPanel,
  workday_hris_connector: WorkdayResponseGroupPanel,
  sage_hr_connector: SageHrHistoryPanel,
  api_dispatcher: ApiDispatcherPanel,
};

/**
 * Look up a custom panel for the given action type.
 * Returns `null` if the action should use the generic field-schema renderer.
 */
export function getActionPanel(
  actionType: ActionType,
): ComponentType<ActionEditorProps> | null {
  return ACTION_PANEL_REGISTRY[actionType] ?? null;
}
