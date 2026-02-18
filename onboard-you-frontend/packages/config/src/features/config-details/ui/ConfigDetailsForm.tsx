import { useTranslation } from 'react-i18next';
import { Box, Flex, Heading, Text, Code } from '@chakra-ui/react';
import { useAppDispatch, useAppSelector } from '@/store';
import { selectSelectedNode, deselectNode } from '../state/configDetailsSlice';
import { businessLabel } from '@/shared/domain/types';
import type { ActionConfigPayload } from '@/generated/api';

const CATEGORY_ICONS: Record<string, string> = {
  ingestion: '📥',
  logic: '⚙️',
  egress: '📤',
};

export function ConfigDetailsForm() {
  const { t } = useTranslation();
  const dispatch = useAppDispatch();
  const selectedNode = useAppSelector(selectSelectedNode);

  if (!selectedNode) return null;

  const nodeData = selectedNode.data as Record<string, unknown>;
  const category = (nodeData.category as string) ?? 'logic';
  const actionType = (nodeData.actionType as string) ?? '';
  const config = nodeData.config as ActionConfigPayload | undefined;

  const handleClose = () => {
    dispatch(deselectNode());
  };

  const configEntries = config && typeof config === 'object' ? Object.entries(config) : [];

  return (
    <Box
      position="absolute"
      top="4"
      right="4"
      w="340px"
      bg="white"
      borderRadius="lg"
      border="1px solid"
      borderColor="gray.200"
      shadow="lg"
      zIndex="10"
      overflow="hidden"
    >
      {/* Header */}
      <Flex align="center" justify="space-between" px="4" py="3" borderBottom="1px solid" borderColor="gray.100" bg="gray.50">
        <Flex align="center" gap="2">
          <Text>{CATEGORY_ICONS[category] ?? '🔧'}</Text>
          <Text fontWeight="600" fontSize="sm">
            {(nodeData.label as string) ?? t(`configDetails.form.categoryLabels.${category}`, t('configDetails.form.nodeDetails'))}
          </Text>
        </Flex>
        <Box
          as="button"
          onClick={handleClose}
          aria-label={t('configDetails.form.close')}
          cursor="pointer"
          fontSize="lg"
          color="gray.400"
          _hover={{ color: 'gray.600' }}
          bg="transparent"
          border="none"
          p="0"
        >
          ×
        </Box>
      </Flex>

      {/* Body */}
      <Box as="dl" p="4" display="flex" flexDirection="column" gap="3">
        {/* Action type */}
        <Box>
          <Text as="dt" fontSize="xs" fontWeight="600" color="gray.500" textTransform="uppercase" letterSpacing="wide">
            {t('configDetails.form.stepType')}
          </Text>
          <Text as="dd" fontSize="sm" mt="0.5">{businessLabel(actionType)}</Text>
        </Box>

        {/* Category */}
        <Box>
          <Text as="dt" fontSize="xs" fontWeight="600" color="gray.500" textTransform="uppercase" letterSpacing="wide">
            {t('configDetails.form.category')}
          </Text>
          <Text as="dd" fontSize="sm" mt="0.5">{t(`configDetails.form.categoryLabels.${category}`, category)}</Text>
        </Box>

        {/* Config key-value pairs */}
        {configEntries.length > 0 ? (
          configEntries.map(([key, value]) => (
            <Box key={key}>
              <Text as="dt" fontSize="xs" fontWeight="600" color="gray.500" textTransform="uppercase" letterSpacing="wide">
                {key}
              </Text>
              <Text as="dd" fontSize="sm" mt="0.5">
                {typeof value === 'string' || typeof value === 'number' || typeof value === 'boolean'
                  ? String(value)
                  : JSON.stringify(value)}
              </Text>
            </Box>
          ))
        ) : (
          <Box>
            <Text as="dt" fontSize="xs" fontWeight="600" color="gray.500" textTransform="uppercase" letterSpacing="wide">
              {t('configDetails.form.configuration')}
            </Text>
            <Text as="dd" fontSize="sm" mt="0.5">{t('configDetails.form.noConfigData')}</Text>
          </Box>
        )}

        {/* JSON fallback */}
        {config && Object.keys(config).length > 0 && (
          <Box mt="2" bg="gray.50" borderRadius="md" p="3" overflow="auto" maxH="200px">
            <Code as="pre" fontSize="xs" whiteSpace="pre-wrap">
              {JSON.stringify(config, null, 2)}
            </Code>
          </Box>
        )}
      </Box>
    </Box>
  );
}
