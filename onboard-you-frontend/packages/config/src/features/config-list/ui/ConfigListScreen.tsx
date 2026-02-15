import { useState, useEffect } from 'react';
import { useTranslation } from 'react-i18next';
import { useNavigate } from 'react-router-dom';
import { useAppDispatch, useAppSelector } from '@/store';
import { fetchConfigs, setSearchQuery, selectConfigList, selectFilteredConfigs } from '../state/configListSlice';
import { ConfigListItem } from './ConfigListItem';
import { Button } from '@/shared/ui/Button';
import { Spinner } from '@/shared/ui/Spinner';
import styles from './ConfigListScreen.module.scss';

function ConfigListScreenInner() {
  const { t } = useTranslation();
  const dispatch = useAppDispatch();
  const state = useAppSelector(selectConfigList);
  const filteredConfigs = useAppSelector(selectFilteredConfigs);
  const navigate = useNavigate();

  useEffect(() => {
    dispatch(fetchConfigs());
  }, [dispatch]);

  return (
    <div className={styles.configListScreen}>
        <div role="tabpanel" id="tabpanel-portfolio">
          <div className={styles.listHeader}>
            <h1 className={styles.title}>{t('configList.title')}</h1>
            <Button variant="primary" size="md" leftIcon={<span>＋</span>} onClick={() => navigate('new')}>
              {t('configList.newConnection')}
            </Button>
          </div>

          <div className={styles.searchBar}>
            <label htmlFor="config-search" style={{ position: 'absolute', width: 1, height: 1, overflow: 'hidden', clip: 'rect(0,0,0,0)', whiteSpace: 'nowrap' as const }}>{t('configList.search.placeholder')}</label>
            <span className={styles.searchIcon} aria-hidden="true">🔍</span>
            <input
              id="config-search"
              type="text"
              className={styles.searchInput}
              placeholder={t('configList.search.placeholder')}
              value={state.searchQuery}
              onChange={(e) => dispatch(setSearchQuery(e.target.value))}
            />
          </div>

          {state.isLoading && (
            <div className={styles.loadingState}>
              <Spinner size="lg" />
              <p>{t('configList.loading')}</p>
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
              <h3 className={styles.emptyTitle}>{t('configList.empty.title')}</h3>
              <p className={styles.emptyDesc}>
                {state.searchQuery
                  ? t('configList.empty.noResults')
                  : t('configList.empty.noData')}
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
    </div>
  );
}

export function ConfigListScreen() {
  return <ConfigListScreenInner />;
}
