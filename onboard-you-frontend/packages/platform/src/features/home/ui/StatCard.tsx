import { Card, Flex, Text } from '@chakra-ui/react';
import type { StatCardData } from '@/features/home/domain/types';

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

  return (
    <Card.Root
      variant="outline"
      cursor="pointer"
      _hover={{ shadow: 'md', transform: 'translateY(-2px)' }}
      transition="all 0.15s ease"
    >
      <Card.Body>
        <Flex justifyContent="space-between" alignItems="center" mb={3}>
          <Text fontSize="2xl">{data.icon}</Text>
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
        <Text as="dd" fontSize="3xl" fontWeight="bold" mb={1}>
          {data.value}
        </Text>
        <Text as="dt" fontSize="sm" color="fg.muted">
          {data.label}
        </Text>
      </Card.Body>
    </Card.Root>
  );
}
