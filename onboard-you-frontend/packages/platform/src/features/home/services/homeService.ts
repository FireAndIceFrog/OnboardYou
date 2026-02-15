import type { TFunction } from 'i18next';
import type { ApiClient } from '@/shared/services/apiClient';
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
export async function fetchDashboardStats(
  apiClient: ApiClient,
): Promise<StatCardData[]> {
  const res = await apiClient.get<DashboardStatsResponse>('/dashboard/stats');

  return [
    {
      label: 'Connected Systems',
      value: res.connectedSystems,
      change: res.changes?.connectedSystems?.text,
      trend: res.changes?.connectedSystems?.trend,
      icon: '🔗',
    },
    {
      label: 'Pending Reviews',
      value: res.pendingReviews,
      change: res.changes?.pendingReviews?.text,
      trend: res.changes?.pendingReviews?.trend,
      icon: '📋',
    },
    {
      label: 'Team Members',
      value: res.teamMembers,
      change: res.changes?.teamMembers?.text,
      trend: res.changes?.teamMembers?.trend,
      icon: '👥',
    },
    {
      label: 'Activity Today',
      value: res.activityToday,
      change: res.changes?.activityToday?.text,
      trend: res.changes?.activityToday?.trend,
      icon: '📊',
    },
  ];
}

/**
 * Mock stats used only when MOCK_MODE is true.
 * Accepts the i18next `t` function so labels stay localised.
 */
export function MOCK_STATS(t: TFunction): StatCardData[] {
  return [
    { label: t('home.stats.connectedSystems'), value: 3, change: t('home.stats.plusOneThisWeek'), trend: 'up', icon: '🔗' },
    { label: t('home.stats.pendingReviews'), value: 7, change: t('home.stats.plusThreeToday'), trend: 'up', icon: '📋' },
    { label: t('home.stats.teamMembers'), value: 12, change: t('home.stats.noChange'), trend: 'neutral', icon: '👥' },
    { label: t('home.stats.activityToday'), value: 156, change: t('home.stats.plusTwelvePercent'), trend: 'up', icon: '📊' },
  ];
}
