import { useConfigList } from '../state';
import { ConfigListProvider } from '../state';
import { ConfigListItem } from './ConfigListItem';
import { Button } from '@/shared/ui/Button';
import { Spinner } from '@/shared/ui/Spinner';
import styles from './ConfigListScreen.module.scss';

function ConfigListScreenInner() {
  const { state, filteredConfigs, setSearchQuery } = useConfigList();

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
          placeholder="Search by name or company…"
          value={state.searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
        />
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
            {state.searchQuery
              ? 'Try adjusting your search criteria.'
              : 'Create your first ETL pipeline configuration to get started.'}
          </p>
        </div>
      )}

      {!state.isLoading && !state.error && filteredConfigs.length > 0 && (
        <div className={styles.configGrid}>
          {filteredConfigs.map((config) => (
            <ConfigListItem key={config.customerCompanyId} config={config} />
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
