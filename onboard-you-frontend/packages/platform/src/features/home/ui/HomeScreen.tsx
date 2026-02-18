import { useState, useEffect } from 'react';
import { useTranslation } from 'react-i18next';
import { Box, Center, Heading, SimpleGrid, Spinner, Text } from '@chakra-ui/react';
import { useGlobal } from '@/shared/hooks/useGlobal';
import { MOCK_MODE } from '@/shared/domain/constants';
import { fetchDashboardStats, MOCK_STATS } from '@/features/home/services/homeService';
import { StatCard } from './StatCard';
import type { StatCardData } from '@/features/home/domain/types';

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
    return () => {
      cancelled = true;
    };
  }, [t]);

  const placeholderStats: StatCardData[] = [
    { label: t('home.stats.connectedSystems'), value: '—', icon: '🔗' },
    { label: t('home.stats.pendingReviews'), value: '—', icon: '📋' },
    { label: t('home.stats.teamMembers'), value: '—', icon: '👥' },
    { label: t('home.stats.activityToday'), value: '—', icon: '📊' },
  ];

  return (
    <Box as="section" maxW="1200px" mx="auto" aria-label="Dashboard overview">
      <Box mb={7}>
        <Heading as="h1" size="2xl" fontWeight="bold" mb={1}>
          {t('home.welcome', { name: userName })}
        </Heading>
        <Text fontSize="sm" color="fg.muted">
          {t('home.subtitle')}
        </Text>
      </Box>

      <Box as="dl">
        {loading ? (
          <Center py={8}>
            <Spinner size="md" />
          </Center>
        ) : (
          <SimpleGrid columns={{ base: 1, md: 2, lg: 4 }} gap={5}>
            {(stats ?? placeholderStats).map((stat) => (
              <StatCard key={stat.label} data={stat} />
            ))}
          </SimpleGrid>
        )}
      </Box>
    </Box>
  );
}
