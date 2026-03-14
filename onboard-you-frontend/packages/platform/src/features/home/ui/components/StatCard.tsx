import { Card, Flex, Text, Box } from '@chakra-ui/react';
import type { StatCardData } from '@/features/home/domain/types';
import { LinkIcon, ClipboardIcon, UsersIcon, BarChartIcon } from '@/shared/ui';

const ICON_MAP: Record<string, React.ComponentType<{ size?: number | string }>> = {
  link: LinkIcon,
  clipboard: ClipboardIcon,
  users: UsersIcon,
  chart: BarChartIcon,
};

interface StatCardProps {
  data: StatCardData;
}

export function StatCard({ data }: StatCardProps) {
  const trendIcon =
    data.trend === 'up' ? '↑' : data.trend === 'down' ? '↓' : '→';

  const trendColor =
    data.trend === 'up'
      ? 'green'
      : data.trend === 'down'
        ? 'red'
        : 'gray';

  const Icon = ICON_MAP[data.iconName];

  return (
    <Card.Root
      variant="outline"
      cursor="pointer"
      borderColor="tertiary.200"
      _hover={{ shadow: 'md', transform: 'translateY(-2px)', borderColor: 'secondary.300' }}
      transition="all 0.15s ease"
    >
      <Card.Body>
        <Flex justifyContent="space-between" alignItems="center" mb={3}>
          <Box color="secondary.500">
            {Icon ? <Icon size="1.5em" /> : null}
          </Box>
          {data.change && (
            <Text
              fontSize="xs"
              fontWeight="medium"
              px={2}
              py={1}
              borderRadius="full"
              color={`${trendColor}.fg`}
              bg={`${trendColor}.subtle`}
            >
              {trendIcon} {data.change}
            </Text>
          )}
        </Flex>
        <Text as="dd" fontSize="3xl" fontWeight="bold" mb={1} color="primary.500">
          {data.value}
        </Text>
        <Text as="dt" fontSize="sm" color="tertiary.500">
          {data.label}
        </Text>
      </Card.Body>
    </Card.Root>
  );
}
