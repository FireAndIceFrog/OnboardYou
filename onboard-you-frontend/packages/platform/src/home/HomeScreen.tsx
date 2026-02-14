// ============================================================================
// OnboardYou — Home Screen
//
// Simple dashboard landing page. Shows a welcome message, quick stats cards,
// and a recent activity list (all placeholder data for now).
// ============================================================================

import { useAuth } from '@/auth/AuthContext';
import styles from './HomeScreen.module.scss';

// ---------------------------------------------------------------------------
// Placeholder data
// ---------------------------------------------------------------------------

interface StatCard {
  label: string;
  value: string;
  change: string;
  trend: 'up' | 'down' | 'neutral';
}

const STATS: StatCard[] = [
  { label: 'Active Configs', value: '12', change: '+3 this week', trend: 'up' },
  { label: 'Pending Reviews', value: '5', change: '2 urgent', trend: 'neutral' },
  { label: 'Team Members', value: '24', change: '+1 this month', trend: 'up' },
];

interface ActivityItem {
  id: string;
  action: string;
  actor: string;
  timestamp: string;
}

const RECENT_ACTIVITY: ActivityItem[] = [
  {
    id: '1',
    action: 'Created new onboarding config "Engineering Q1"',
    actor: 'Jane Cooper',
    timestamp: '2 hours ago',
  },
  {
    id: '2',
    action: 'Approved review for "Sales Onboarding v2"',
    actor: 'Alex Morgan',
    timestamp: '5 hours ago',
  },
  {
    id: '3',
    action: 'Updated field mapping on "Customer Success"',
    actor: 'Priya Sharma',
    timestamp: 'Yesterday',
  },
  {
    id: '4',
    action: 'Invited team member sarah@acme.com',
    actor: 'You',
    timestamp: 'Yesterday',
  },
  {
    id: '5',
    action: 'Deployed config "Marketing Onboarding" to production',
    actor: 'Chris Lee',
    timestamp: '3 days ago',
  },
];

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

export function HomeScreen() {
  const { user } = useAuth();

  const firstName = user?.name?.split(' ')[0] ?? 'there';

  return (
    <div className={styles.page}>
      {/* Welcome */}
      <section className={styles.welcome}>
        <h1 className={styles.heading}>
          Welcome back, {firstName} 👋
        </h1>
        <p className={styles.subtitle}>
          Here's what's happening with your onboarding configurations.
        </p>
      </section>

      {/* Stats cards */}
      <section className={styles.stats}>
        {STATS.map((stat) => (
          <div key={stat.label} className={styles.card}>
            <p className={styles.cardLabel}>{stat.label}</p>
            <p className={styles.cardValue}>{stat.value}</p>
            <p
              className={`${styles.cardChange} ${
                stat.trend === 'up'
                  ? styles['cardChange--up']
                  : stat.trend === 'down'
                    ? styles['cardChange--down']
                    : ''
              }`}
            >
              {stat.trend === 'up' && '↑ '}
              {stat.trend === 'down' && '↓ '}
              {stat.change}
            </p>
          </div>
        ))}
      </section>

      {/* Recent activity */}
      <section className={styles.activity}>
        <h2 className={styles.sectionTitle}>Recent Activity</h2>

        <ul className={styles.activityList}>
          {RECENT_ACTIVITY.map((item) => (
            <li key={item.id} className={styles.activityItem}>
              <div className={styles.activityDot} />
              <div className={styles.activityContent}>
                <p className={styles.activityAction}>{item.action}</p>
                <p className={styles.activityMeta}>
                  {item.actor} · {item.timestamp}
                </p>
              </div>
            </li>
          ))}
        </ul>
      </section>
    </div>
  );
}
