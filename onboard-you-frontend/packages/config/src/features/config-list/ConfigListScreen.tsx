import { useNavigate } from 'react-router-dom';
import { useConfigList } from './useConfigList';
import { ConfigListItem } from './ConfigListItem';
import styles from './ConfigListScreen.module.scss';

function SkeletonCard() {
  return (
    <div className={styles.skeletonCard}>
      <div className={styles.skeletonLine} style={{ width: '60%', height: 20 }} />
      <div className={styles.skeletonLine} style={{ width: '100%', height: 14, marginTop: 12 }} />
      <div className={styles.skeletonLine} style={{ width: '80%', height: 14, marginTop: 6 }} />
      <div className={styles.skeletonLine} style={{ width: '40%', height: 14, marginTop: 16 }} />
    </div>
  );
}

export function ConfigListScreen() {
  const navigate = useNavigate();
  const { filteredConfigs, isLoading, error, searchQuery, setSearchQuery, deleteConfig } =
    useConfigList();

  return (
    <div className={styles.container}>
      <header className={styles.header}>
        <div>
          <h1 className={styles.title}>Configurations</h1>
          <p className={styles.subtitle}>Manage your ETL pipeline configurations</p>
        </div>
        <button className={styles.newBtn} onClick={() => navigate('/new')}>
          + New Config
        </button>
      </header>

      <div className={styles.toolbar}>
        <div className={styles.searchWrapper}>
          <span className={styles.searchIcon}>🔍</span>
          <input
            className={styles.searchInput}
            type="text"
            placeholder="Search by name, source system, or description…"
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
          />
          {searchQuery && (
            <button className={styles.clearBtn} onClick={() => setSearchQuery('')}>
              ×
            </button>
          )}
        </div>
      </div>

      {error && (
        <div className={styles.errorBanner}>
          <span>⚠️ {error}</span>
        </div>
      )}

      {isLoading ? (
        <div className={styles.grid}>
          <SkeletonCard />
          <SkeletonCard />
          <SkeletonCard />
        </div>
      ) : filteredConfigs.length === 0 ? (
        <div className={styles.emptyState}>
          <span className={styles.emptyIcon}>📋</span>
          <h2 className={styles.emptyTitle}>
            {searchQuery ? 'No matching configurations' : 'No configurations yet'}
          </h2>
          <p className={styles.emptyText}>
            {searchQuery
              ? `No results for "${searchQuery}". Try a different search term.`
              : 'Create your first ETL pipeline configuration to get started.'}
          </p>
          {!searchQuery && (
            <button className={styles.newBtn} onClick={() => navigate('/new')}>
              + Create Configuration
            </button>
          )}
        </div>
      ) : (
        <div className={styles.grid}>
          {filteredConfigs.map((config) => (
            <ConfigListItem key={config.id} config={config} onDelete={deleteConfig} />
          ))}
        </div>
      )}
    </div>
  );
}
