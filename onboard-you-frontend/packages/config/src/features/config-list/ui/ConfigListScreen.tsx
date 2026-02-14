import { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { useConfigList } from '../state';
import { ConfigListProvider } from '../state';
import { ConfigListItem } from './ConfigListItem';
import { Button } from '@/shared/ui/Button';
import { Spinner } from '@/shared/ui/Spinner';
import styles from './ConfigListScreen.module.scss';

type TabId = 'portfolio' | 'systems';

function ConfigListScreenInner() {
  const { state, filteredConfigs, setSearchQuery } = useConfigList();
  const [activeTab, setActiveTab] = useState<TabId>('portfolio');
  const navigate = useNavigate();

  return (
    <div className={styles.configListScreen}>
      {/* Dual-tab header */}
      <div className={styles.tabBar}>
        <button
          className={`${styles.tab} ${activeTab === 'portfolio' ? styles.tabActive : ''}`}
          onClick={() => setActiveTab('portfolio')}
        >
          🏢 Client Portfolio
        </button>
        <button
          className={`${styles.tab} ${activeTab === 'systems' ? styles.tabActive : ''}`}
          onClick={() => setActiveTab('systems')}
        >
          ⚙️ My Systems
        </button>
      </div>

      {activeTab === 'portfolio' && (
        <>
          <div className={styles.listHeader}>
            <h1 className={styles.title}>Connected Systems</h1>
            <Button variant="primary" size="md" leftIcon={<span>＋</span>} onClick={() => navigate('new')}>
              New Connection
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
              <p>Loading connected systems…</p>
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
              <span className={styles.emptyIcon}>🔗</span>
              <h3 className={styles.emptyTitle}>No connected systems found</h3>
              <p className={styles.emptyDesc}>
                {state.searchQuery
                  ? 'Try adjusting your search criteria.'
                  : 'Connect your first HR system to get started.'}
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
        </>
      )}

      {activeTab === 'systems' && (
        <div className={styles.emptyState}>
          <span className={styles.emptyIcon}>⚙️</span>
          <h3 className={styles.emptyTitle}>My Systems</h3>
          <p className={styles.emptyDesc}>
            Manage your internal integration settings and API keys here.
          </p>
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
