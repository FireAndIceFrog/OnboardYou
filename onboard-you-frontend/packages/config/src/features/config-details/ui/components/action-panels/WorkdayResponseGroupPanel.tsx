import { Box, Flex, Text } from '@chakra-ui/react';
import { RESPONSE_GROUP_OPTIONS } from '../../../domain/types';
import type { ActionEditorProps } from './registry';

export function WorkdayResponseGroupPanel({ config, onChange }: ActionEditorProps) {
  const group = (typeof config.response_group === 'object' && config.response_group !== null
    ? config.response_group
    : {}) as Record<string, boolean>;

  const toggle = (field: string) => {
    onChange('response_group', { ...group, [field]: !group[field] });
  };

  return (
    <Box data-testid="workday-response-group-panel">
      <Text fontSize="sm" fontWeight="600" mb="1">
        Response Groups
      </Text>
      <Text fontSize="xs" color="gray.500" mb="2">
        Data sections to include in the Workday response
      </Text>
      <Flex wrap="wrap" gap="2">
        {RESPONSE_GROUP_OPTIONS.map((opt) => (
          <Box
            as="label"
            key={opt.value}
            display="flex"
            alignItems="center"
            gap="1.5"
            px="2.5"
            py="1"
            borderRadius="full"
            border="1px solid"
            borderColor={group[opt.value] ? 'blue.400' : 'gray.200'}
            bg={group[opt.value] ? 'blue.50' : 'white'}
            cursor="pointer"
            fontSize="xs"
            transition="all 0.15s"
            _hover={{ borderColor: 'blue.300' }}
          >
            <input
              type="checkbox"
              checked={!!group[opt.value]}
              onChange={() => toggle(opt.value)}
              style={{ display: 'none' }}
              data-testid={`response-group-${opt.value}`}
            />
            <Text>{opt.label}</Text>
          </Box>
        ))}
      </Flex>
    </Box>
  );
}
