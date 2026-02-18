import { Handle, Position, type NodeProps } from '@xyflow/react';
import { useTranslation } from 'react-i18next';
import { Box, Flex, Text } from '@chakra-ui/react';
import { businessLabel } from '@/shared/domain/types';

export function IngestionNode({ data }: NodeProps) {
  const { t } = useTranslation();
  const friendly = businessLabel(data.actionType as string);

  return (
    <Box bg="white" border="1px solid" borderColor="gray.200" borderRadius="lg" borderLeft="4px solid" borderLeftColor="green.400" shadow="sm" minW="180px" p="3">
      <Flex align="center" gap="2" mb="1">
        <Text fontSize="md">📥</Text>
        <Text fontSize="xs" fontWeight="600" color="gray.500" textTransform="uppercase" letterSpacing="wide">{t('configDetails.nodes.dataSource')}</Text>
      </Flex>
      <Text fontSize="sm" fontWeight="500" mb="2">{data.label as string}</Text>
      <Box px="2" py="0.5" borderRadius="full" bg="green.50" color="green.700" fontSize="xs" fontWeight="500" display="inline-block">{friendly}</Box>
      <Handle type="target" position={Position.Left} />
      <Handle type="source" position={Position.Right} />
    </Box>
  );
}
