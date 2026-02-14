import { useConfigList } from '../state';
import { ConfigListProvider } from '../state';
import { ConfigListItem } from './ConfigListItem';
import { Button } from '@/shared/ui/Button';
import { Spinner } from '@/shared/ui/Spinner';
import styles from './ConfigListScreen.module.scss';

const STATUS_TABS = [
  { label: 'All', value: null },
  { label: 'Active', value: 'active' },
  { label: 'Draft', value: 'draft' },
  { label: 'Paused', value: 'paused' },
  { label: 'Error', value: 'error' },
] as const;

function ConfigListScreenInner() {
  const { state, filteredConfigs, setSearchQuery, setStatusFilter } = useConfigList();

  return (
    <div className={styles.configListScreen}>
      <div className={styles.listHeader}>
        <h1 className={styles.title}>Configurations</h1>
        <Button variant="primary" size="md" leftIcon={<span>＋</span>}>
          New Config
        </Button>
      </div>

      <div className={styles.searchBar}>
        <span className={styles.searchIcon}>🔍</span>
        <input
          type="text"
          className={styles.searchInput}
          placeholder="Search configurations..."
          value={state.searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
        />
      </div>

      <div className={styles.filterTabs}>
        {STATUS_TABS.map((tab) => (
          <button
            key={tab.label}
            className={`${styles.filterTab} ${
              state.statusFilter === tab.value ? styles['filterTab--active'] : ''
            }`}
            onClick={() => setStatusFilter(tab.value)}
          >
            {tab.label}
          </button>
        ))}
      </div>

      {state.isLoading && (
        <div className={styles.loadingState}>
          <Spinner size="lg" />
          <p>Loading configurations…</p>
        </div>
      )}

      {state.error && !state.isLoading && (
        <div className={styles.errorState}>
          <span className={styles.errorIcon}>⚠️</span>
          <p>{state.error}</p>
        </div>
      )}

      {!state.isLoading && !state.error && filteredConfigs.length === 0 && (
        <div className={styles.emptyState}>
          <span className={styles.emptyIcon}>📋</span>
          <h3 className={styles.emptyTitle}>No configurations found</h3>
          <p className={styles.emptyDesc}>
            {state.searchQuery || state.statusFilter
              ? 'Try adjusting your search or filter criteria.'
              : 'Create your first ETL pipeline configuration to get started.'}
          </p>
        </div>
      )}

      {!state.isLoading && !state.error && filteredConfigs.length > 0 && (
        <div className={styles.configGrid}>
          {filteredConfigs.map((config) => (
            <ConfigListItem key={config.id} config={config} />
          ))}
        </div>
      )}
    </div>
  );
}

export function ConfigListScreen() {
  return (
    <ConfigListProvider>
      <ConfigListScreenInner />
    </ConfigListProvider>
  );
}
