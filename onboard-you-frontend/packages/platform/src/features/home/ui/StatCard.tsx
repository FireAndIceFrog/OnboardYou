import { Card } from '@/shared/ui/Card';
import type { StatCardData } from '@/features/home/domain/types';
import styles from './HomeScreen.module.scss';

interface StatCardProps {
  data: StatCardData;
}

export function StatCard({ data }: StatCardProps) {
  const trendIcon =
    data.trend === 'up' ? '↑' : data.trend === 'down' ? '↓' : '→';
  const trendClass =
    data.trend === 'up'
      ? styles['trend--up']
      : data.trend === 'down'
        ? styles['trend--down']
        : styles['trend--neutral'];

  return (
    <Card hoverable className={styles['stat-card']}>
      <div className={styles['stat-header']}>
        <span className={styles['stat-icon']}>{data.icon}</span>
        {data.change && (
          <span className={[styles['stat-change'], trendClass].join(' ')}>
            {trendIcon} {data.change}
          </span>
        )}
      </div>
      <dd className={styles['stat-value']}>{data.value}</dd>
      <dt className={styles['stat-label']}>{data.label}</dt>
    </Card>
  );
}
