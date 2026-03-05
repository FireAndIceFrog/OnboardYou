import { Handle, Position, type NodeProps } from '@xyflow/react';
import { useTranslation } from 'react-i18next';
import { Box, Flex, Text } from '@chakra-ui/react';
import { useAppSelector } from '@/store';
import type { RootState } from '@/store';
import { businessLabel } from '@/shared/domain/types';

/* ── Style lookup by category ──────────────────────────────── */

interface CategoryStyle {
  color: string;
  badgeBg: string;
  badgeColor: string;
  icon: string;
  i18nKey: string;
}

const CATEGORY_STYLES: Record<string, CategoryStyle> = {
  ingestion: {
    color: 'green.400',
    badgeBg: 'green.50',
    badgeColor: 'green.700',
    icon: '📥',
    i18nKey: 'configDetails.nodes.dataSource',
  },
  logic: {
    color: 'blue.400',
    badgeBg: 'blue.50',
    badgeColor: 'blue.700',
    icon: '⚙️',
    i18nKey: 'configDetails.nodes.businessRule',
  },
  egress: {
    color: 'orange.400',
    badgeBg: 'orange.50',
    badgeColor: 'orange.700',
    icon: '📤',
    i18nKey: 'configDetails.nodes.destination',
  },
};

const DEFAULT_STYLE: CategoryStyle = {
  color: 'gray.400',
  badgeBg: 'gray.50',
  badgeColor: 'gray.700',
  icon: '🔧',
  i18nKey: 'configDetails.nodes.step',
};

/**
 * Unified React Flow node component.
 * Replaces the former IngestionNode, TransformationNode, and EgressNode
 * with a single data-driven component whose appearance is controlled
 * by the `category` field in the node's data.
 */
export function PipelineNode({ data }: NodeProps) {
  const { t } = useTranslation();
  const category = (data.category as string) ?? 'logic';
  const style = CATEGORY_STYLES[category] ?? DEFAULT_STYLE;
  const friendly = businessLabel(data.actionType as string);
  const actionId = (data.actionId as string) ?? '';

  // Select only this action's error (primitive string|undefined) to avoid
  // re-renders when the validationErrors object reference changes.
  const error = useAppSelector(
    (state: RootState) => state.configDetails.validationErrors[actionId],
  );

  return (
    <Box
      bg={error ? 'red.50' : 'white'}
      border="1px solid"
      borderColor={error ? 'red.300' : 'gray.200'}
      borderRadius="lg"
      borderLeft="4px solid"
      borderLeftColor={error ? 'red.500' : style.color}
      shadow={error ? 'md' : 'sm'}
      minW="180px"
      p="3"
      data-testid={`pipeline-node-${category}`}
    >
      <Flex align="center" gap="2" mb="1">
        <Text fontSize="md">{style.icon}</Text>
        <Text
          fontSize="xs"
          fontWeight="600"
          color="gray.500"
          textTransform="uppercase"
          letterSpacing="wide"
        >
          {t(style.i18nKey)}
        </Text>
      </Flex>
      <Text fontSize="sm" fontWeight="500" mb="2">
        {data.label as string}
      </Text>
      <Box
        px="2"
        py="0.5"
        borderRadius="full"
        bg={style.badgeBg}
        color={style.badgeColor}
        fontSize="xs"
        fontWeight="500"
        display="inline-block"
      >
        {friendly}
      </Box>
      {error && (
        <Text fontSize="xs" color="red.600" mt="2" lineClamp={2}>
          ⚠️ {error}
        </Text>
      )}
      <Handle type="target" position={Position.Left} />
      <Handle type="source" position={Position.Right} />
    </Box>
  );
}
