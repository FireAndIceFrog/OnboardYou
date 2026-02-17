import { useState, useEffect } from 'react';
import { useTranslation } from 'react-i18next';
import { useGlobal } from '@/shared/hooks/useGlobal';
import { Spinner } from '@/shared/ui/Spinner';
import { MOCK_MODE } from '@/shared/domain/constants';
import { fetchDashboardStats, MOCK_STATS } from '@/features/home/services/homeService';
import { StatCard } from './StatCard';
import type { StatCardData } from '@/features/home/domain/types';
import styles from './HomeScreen.module.scss';

export function HomeScreen() {
  const { t } = useTranslation();
  const { user } = useGlobal();
  const userName = user?.name ?? 'there';

  const [stats, setStats] = useState<StatCardData[] | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    let cancelled = false;

    async function loadStats() {
      if (MOCK_MODE) {
        setStats(MOCK_STATS(t));
        setLoading(false);
        return;
      }

      try {
        const data = await fetchDashboardStats();
        if (!cancelled) setStats(data);
      } catch {
        if (!cancelled) setStats(null);
      } finally {
        if (!cancelled) setLoading(false);
      }
    }

    loadStats();
    return () => { cancelled = true; };
  }, [t]);

  const placeholderStats: StatCardData[] = [
    { label: t('home.stats.connectedSystems'), value: '—', icon: '🔗' },
    { label: t('home.stats.pendingReviews'), value: '—', icon: '📋' },
    { label: t('home.stats.teamMembers'), value: '—', icon: '👥' },
    { label: t('home.stats.activityToday'), value: '—', icon: '📊' },
  ];

  return (
    <section className={styles['home-screen']} aria-label="Dashboard overview">
      <section className={styles['welcome-section']}>
        <h1 className={styles['welcome-title']}>{t('home.welcome', { name: userName })}</h1>
        <p className={styles['welcome-subtitle']}>
          {t('home.subtitle')}
        </p>
      </section>

      <dl className={styles['stats-grid']}>
        {loading ? (
          <div className={styles['stats-loading']}>
            <Spinner size="md" />
          </div>
        ) : (
          (stats ?? placeholderStats).map((stat) => (
            <StatCard key={stat.label} data={stat} />
          ))
        )}
      </dl>
    </section>
  );
}
