import type { TFunction } from 'i18next';
import { client } from '@/generated/api/client.gen';
import type { StatCardData } from '@/features/home/domain/types';

/** Shape returned by the dashboard stats API endpoint. */
interface DashboardStatsResponse {
  connectedSystems: number;
  pendingReviews: number;
  teamMembers: number;
  activityToday: number;
  changes?: {
    connectedSystems?: { text: string; trend: 'up' | 'down' | 'neutral' };
    pendingReviews?: { text: string; trend: 'up' | 'down' | 'neutral' };
    teamMembers?: { text: string; trend: 'up' | 'down' | 'neutral' };
    activityToday?: { text: string; trend: 'up' | 'down' | 'neutral' };
  };
}

/**
 * Fetch dashboard stats from the API and map them to StatCardData[].
 */
export async function fetchDashboardStats(): Promise<StatCardData[]> {
  const { data: res } = await client.get<DashboardStatsResponse>({
    url: '/dashboard/stats',
  });

  if (!res) {
    throw new Error('Dashboard stats response was empty');
  }

  return [
    {
      label: 'Connected Systems',
      value: res.connectedSystems,
      change: res.changes?.connectedSystems?.text,
      trend: res.changes?.connectedSystems?.trend,
      iconName: 'link',
    },
    {
      label: 'Pending Reviews',
      value: res.pendingReviews,
      change: res.changes?.pendingReviews?.text,
      trend: res.changes?.pendingReviews?.trend,
      iconName: 'clipboard',
    },
    {
      label: 'Team Members',
      value: res.teamMembers,
      change: res.changes?.teamMembers?.text,
      trend: res.changes?.teamMembers?.trend,
      iconName: 'users',
    },
    {
      label: 'Activity Today',
      value: res.activityToday,
      change: res.changes?.activityToday?.text,
      trend: res.changes?.activityToday?.trend,
      iconName: 'chart',
    },
  ];
}

/**
 * Mock stats used only when MOCK_MODE is true.
 * Accepts the i18next `t` function so labels stay localised.
 */
export function MOCK_STATS(t: TFunction): StatCardData[] {
  return [
    { label: t('home.stats.connectedSystems'), value: 3, change: t('home.stats.plusOneThisWeek'), trend: 'up', iconName: 'link' },
    { label: t('home.stats.pendingReviews'), value: 7, change: t('home.stats.plusThreeToday'), trend: 'up', iconName: 'clipboard' },
    { label: t('home.stats.teamMembers'), value: 12, change: t('home.stats.noChange'), trend: 'neutral', iconName: 'users' },
    { label: t('home.stats.activityToday'), value: 156, change: t('home.stats.plusTwelvePercent'), trend: 'up', iconName: 'chart' },
  ];
}
