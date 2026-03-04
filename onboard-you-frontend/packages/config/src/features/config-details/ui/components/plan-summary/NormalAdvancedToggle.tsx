import { useCallback } from 'react';
import { Box, Flex, Text } from '@chakra-ui/react';
import type { ViewMode } from '../../../domain/types';

interface NormalAdvancedToggleProps {
  viewMode: ViewMode;
  onToggle: (mode: ViewMode) => void;
}

/**
 * Pill toggle that switches between Normal (plan summary) and Advanced (React Flow) views.
 */
export function NormalAdvancedToggle({ viewMode, onToggle }: NormalAdvancedToggleProps) {
  const handleNormal = useCallback(() => onToggle('normal'), [onToggle]);
  const handleAdvanced = useCallback(() => onToggle('advanced'), [onToggle]);

  return (
    <Flex
      display="inline-flex"
      bg="gray.100"
      borderRadius="full"
      p="1"
      gap="0"
      data-testid="normal-advanced-toggle"
    >
      <Box
        as="button"
        onClick={handleNormal}
        px="4"
        py="1.5"
        borderRadius="full"
        fontSize="sm"
        fontWeight="500"
        cursor="pointer"
        transition="all 0.2s"
        bg={viewMode === 'normal' ? 'purple.600' : 'transparent'}
        color={viewMode === 'normal' ? 'white' : 'gray.600'}
        _hover={viewMode === 'normal' ? {} : { color: 'gray.800' }}
        data-testid="toggle-normal"
      >
        <Text>Normal</Text>
      </Box>
      <Box
        as="button"
        onClick={handleAdvanced}
        px="4"
        py="1.5"
        borderRadius="full"
        fontSize="sm"
        fontWeight="500"
        cursor="pointer"
        transition="all 0.2s"
        bg={viewMode === 'advanced' ? 'purple.600' : 'transparent'}
        color={viewMode === 'advanced' ? 'white' : 'gray.600'}
        _hover={viewMode === 'advanced' ? {} : { color: 'gray.800' }}
        data-testid="toggle-advanced"
      >
        <Text>Advanced</Text>
      </Box>
    </Flex>
  );
}
