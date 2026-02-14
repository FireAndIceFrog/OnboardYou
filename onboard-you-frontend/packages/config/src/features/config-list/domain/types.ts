import type { PipelineConfig } from '@/shared/domain/types';

export interface ConfigListState {
  configs: PipelineConfig[];
  isLoading: boolean;
  error: string | null;
  searchQuery: string;
}
