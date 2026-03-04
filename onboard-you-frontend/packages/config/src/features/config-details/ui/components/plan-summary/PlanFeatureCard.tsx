import { useCallback } from 'react';
import { Box, Flex, Text } from '@chakra-ui/react';
import type { PlanFeature } from '@/generated/api';

/** Map icon names from the AI response to emoji fallbacks */
const ICON_MAP: Record<string, string> = {
  calendar: '📅',
  users: '👥',
  shield: '🛡️',
  filter: '🔍',
  rename: '✏️',
  transform: '🔄',
  sync: '🔄',
  email: '📧',
  dedup: '🔒',
  drop: '🗑️',
  mask: '🎭',
};

interface PlanFeatureCardProps {
  feature: PlanFeature;
  /** Whether the feature is currently enabled (linked actions not disabled) */
  enabled: boolean;
  onToggle: (featureId: string) => void;
}

/**
 * Individual feature card with icon, label, description, and toggle switch.
 * White background, subtle border, rounded corners.
 */
export function PlanFeatureCard({ feature, enabled, onToggle }: PlanFeatureCardProps) {
  const handleToggle = useCallback(() => {
    onToggle(feature.id);
  }, [onToggle, feature.id]);

  const icon = ICON_MAP[feature.icon] ?? '⚙️';

  return (
    <Flex
      bg="white"
      border="1px solid"
      borderColor="gray.200"
      borderRadius="lg"
      p="4"
      gap="3"
      align="flex-start"
      justify="space-between"
      transition="border-color 0.2s"
      _hover={{ borderColor: 'gray.300' }}
      data-testid={`feature-card-${feature.id}`}
    >
      <Flex gap="3" align="flex-start" flex="1">
        <Text fontSize="xl" lineHeight="1.2">{icon}</Text>
        <Box>
          <Text fontWeight="600" fontSize="sm" color="gray.800">
            {feature.label}
          </Text>
          <Text fontSize="xs" color="gray.500" mt="0.5">
            {feature.description}
          </Text>
        </Box>
      </Flex>

      {/* Toggle switch */}
      <Box
        as="button"
        onClick={handleToggle}
        position="relative"
        flexShrink={0}
        w="40px"
        h="22px"
        borderRadius="full"
        bg={enabled ? 'purple.500' : 'gray.300'}
        cursor="pointer"
        transition="background 0.2s"
        role="switch"
        aria-checked={enabled}
        aria-label={`Toggle ${feature.label}`}
        data-testid={`feature-toggle-${feature.id}`}
      >
        <Box
          position="absolute"
          top="2px"
          left={enabled ? '20px' : '2px'}
          w="18px"
          h="18px"
          borderRadius="full"
          bg="white"
          boxShadow="sm"
          transition="left 0.2s"
        />
      </Box>
    </Flex>
  );
}
