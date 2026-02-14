export interface PipelineConfig {
  id: string;
  organizationId: string;
  name: string;
  description: string;
  sourceSystem: string;
  status: 'draft' | 'active' | 'paused' | 'error';
  version: number;
  pipeline: PipelineDefinition;
  createdAt: string;
  updatedAt: string;
}

export interface PipelineDefinition {
  ingestion: IngestionStage;
  transformations: TransformationStage[];
  egress: EgressStage;
}

export interface IngestionStage {
  type: string;
  source: string;
  config: Record<string, unknown>;
}

export interface TransformationStage {
  id: string;
  type: string;
  name: string;
  config: Record<string, unknown>;
  dependsOn: string[];
}

export interface EgressStage {
  type: string;
  destination: string;
  config: Record<string, unknown>;
}

export interface PipelineNode {
  id: string;
  type: 'ingestion' | 'transformation' | 'egress';
  data: {
    label: string;
    stageType: string;
    config: Record<string, unknown>;
    status?: string;
  };
  position: { x: number; y: number };
}

export interface ChatMessage {
  id: string;
  role: 'user' | 'assistant' | 'system';
  content: string;
  timestamp: string;
}

export interface User {
  id: string;
  email: string;
  name: string;
  organizationId: string;
  role: string;
}

export interface Organization {
  id: string;
  name: string;
  plan: string;
}

export type NotificationType = 'success' | 'error' | 'warning' | 'info';

export interface ApiErrorResponse {
  statusCode: number;
  message: string;
}
