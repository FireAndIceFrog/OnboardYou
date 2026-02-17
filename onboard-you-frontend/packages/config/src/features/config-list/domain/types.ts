import type { PipelineConfig } from '@/generated/api';

export interface ConfigListState {
  configs: PipelineConfig[];
  isLoading: boolean;
  error: string | null;
  searchQuery: string;
}
