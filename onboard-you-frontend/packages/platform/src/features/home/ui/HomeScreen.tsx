import { useContext } from 'react';
import { AuthContext } from '@/features/auth/state/AuthContext';
import { StatCard } from './StatCard';
import type { StatCardData } from '@/features/home/domain/types';
import styles from './HomeScreen.module.scss';

const STATS: StatCardData[] = [
  { label: 'Connected Systems', value: 3, change: '+1 this week', trend: 'up', icon: '🔗' },
  { label: 'Pending Reviews', value: 7, change: '+3 today', trend: 'up', icon: '📋' },
  { label: 'Team Members', value: 12, change: 'No change', trend: 'neutral', icon: '👥' },
  { label: 'Activity Today', value: 156, change: '+12%', trend: 'up', icon: '📊' },
];

export function HomeScreen() {
  const authCtx = useContext(AuthContext);
  const userName = authCtx?.state.user?.name ?? 'there';

  return (
    <div className={styles['home-screen']}>
      <section className={styles['welcome-section']}>
        <h1 className={styles['welcome-title']}>Welcome back, {userName}</h1>
        <p className={styles['welcome-subtitle']}>
          Here&apos;s an overview of your client portfolio and connected systems.
        </p>
      </section>

      <section className={styles['stats-grid']}>
        {STATS.map((stat) => (
          <StatCard key={stat.label} data={stat} />
        ))}
      </section>
    </div>
  );
}
