import { useState, useEffect, useCallback, useMemo } from 'react';
import { useGlobal } from '@/hooks/useGlobal';
import type { PipelineConfig } from '@/types';

interface UseConfigListReturn {
  configs: PipelineConfig[];
  filteredConfigs: PipelineConfig[];
  isLoading: boolean;
  error: string | null;
  searchQuery: string;
  setSearchQuery: (q: string) => void;
  fetchConfigs: () => Promise<void>;
  deleteConfig: (id: string) => Promise<void>;
}

export function useConfigList(): UseConfigListReturn {
  const { apiClient, showNotification } = useGlobal();
  const [configs, setConfigs] = useState<PipelineConfig[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [searchQuery, setSearchQuery] = useState('');

  const fetchConfigs = useCallback(async () => {
    setIsLoading(true);
    setError(null);
    try {
      const data = await apiClient.get<PipelineConfig[]>('/configs');
      setConfigs(data);
    } catch (err) {
      const message = err instanceof Error ? err.message : 'Failed to load configurations';
      setError(message);
      showNotification(message, 'error');
    } finally {
      setIsLoading(false);
    }
  }, [apiClient, showNotification]);

  const deleteConfig = useCallback(
    async (id: string) => {
      try {
        await apiClient.delete(`/configs/${id}`);
        setConfigs((prev) => prev.filter((c) => c.id !== id));
        showNotification('Configuration deleted', 'success');
      } catch (err) {
        const message = err instanceof Error ? err.message : 'Failed to delete configuration';
        showNotification(message, 'error');
      }
    },
    [apiClient, showNotification],
  );

  const filteredConfigs = useMemo(() => {
    if (!searchQuery.trim()) return configs;
    const q = searchQuery.toLowerCase();
    return configs.filter(
      (c) =>
        c.name.toLowerCase().includes(q) ||
        c.description.toLowerCase().includes(q) ||
        c.sourceSystem.toLowerCase().includes(q),
    );
  }, [configs, searchQuery]);

  useEffect(() => {
    fetchConfigs();
  }, [fetchConfigs]);

  return {
    configs,
    filteredConfigs,
    isLoading,
    error,
    searchQuery,
    setSearchQuery,
    fetchConfigs,
    deleteConfig,
  };
}
