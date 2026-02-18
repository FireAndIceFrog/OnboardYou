import { useNavigate } from 'react-router-dom';
import { Box, Flex, Heading, Text, Badge } from '@chakra-ui/react';
import type { PipelineConfig } from '@/shared/domain/types';
import { humanFrequency, deriveStatus, STATUS_DISPLAY } from '@/shared/domain/types';

interface ConfigListItemProps {
  config: PipelineConfig;
}

function relativeTime(dateStr: string): string {
  if (!dateStr) return '';
  const now = Date.now();
  const then = new Date(dateStr).getTime();
  const diffMs = now - then;
  const diffMins = Math.floor(diffMs / 60_000);

  if (diffMins < 1) return 'just now';
  if (diffMins < 60) return `${diffMins}m ago`;

  const diffHours = Math.floor(diffMins / 60);
  if (diffHours < 24) return `${diffHours}h ago`;

  const diffDays = Math.floor(diffHours / 24);
  if (diffDays < 30) return `${diffDays}d ago`;

  const diffMonths = Math.floor(diffDays / 30);
  if (diffMonths < 12) return `${diffMonths}mo ago`;

  return `${Math.floor(diffMonths / 12)}y ago`;
}

function fullDate(dateStr: string): string {
  if (!dateStr) return '';
  return new Date(dateStr).toLocaleString(undefined, {
    dateStyle: 'medium',
    timeStyle: 'short',
  });
}

const VARIANT_MAP: Record<string, 'green' | 'gray' | 'yellow' | 'red' | 'blue'> = {
  active: 'green',
  draft: 'gray',
  paused: 'yellow',
  error: 'red',
  info: 'blue',
};

export function ConfigListItem({ config }: ConfigListItemProps) {
  const navigate = useNavigate();
  const status = deriveStatus(config);
  const statusInfo = STATUS_DISPLAY[status];
  const frequency = humanFrequency(config.cron);

  return (
    <Box
      bg="white"
      borderRadius="lg"
      border="1px solid"
      borderColor="gray.200"
      p="5"
      cursor="pointer"
      transition="all 0.15s ease"
      _hover={{ borderColor: 'blue.300', shadow: 'md' }}
      onClick={() => navigate(config.customerCompanyId)}
      role="link"
      tabIndex={0}
      onKeyDown={(e) => {
        if (e.key === 'Enter' || e.key === ' ') {
          e.preventDefault();
          navigate(config.customerCompanyId);
        }
      }}
    >
      <Heading size="sm" mb="1" truncate>{config.name}</Heading>
      <Text fontSize="xs" color="gray.500" mb="4" truncate>{config.customerCompanyId}</Text>

      <Flex justify="space-between" align="center">
        <Text fontSize="xs" color="gray.500" title={`Cron: ${config.cron}`}>
          🔄 {frequency}
        </Text>
        <Flex align="center" gap="2">
          <Badge colorPalette={VARIANT_MAP[statusInfo.variant] ?? 'gray'} size="sm">
            {statusInfo.label}
          </Badge>
          <Text
            fontSize="xs"
            color="gray.400"
            title={`Last edited: ${fullDate(config.lastEdited ?? '')}`}
          >
            {relativeTime(config.lastEdited ?? '')}
          </Text>
        </Flex>
      </Flex>
    </Box>
  );
}
