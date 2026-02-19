import { useCallback } from 'react';
import { useTranslation } from 'react-i18next';
import { Box, Flex, Heading, Text } from '@chakra-ui/react';
import { useAppDispatch } from '@/store';
import type { ActionConfig } from '@/generated/api';
import { ACTION_CATALOG, type ActionCatalogEntry } from '../../domain/actionCatalog';
import { addFlowAction, setAddStepPanelOpen } from '../../state/configDetailsSlice';

const LOGIC_STEPS = ACTION_CATALOG.filter((a) => a.category === 'logic');
const EGRESS_STEPS = ACTION_CATALOG.filter((a) => a.category === 'egress');

interface AddStepPanelProps {
  onClose: () => void;
}

export function AddStepPanel({ onClose }: AddStepPanelProps) {
  const { t } = useTranslation();
  const dispatch = useAppDispatch();

  const handleAdd = useCallback(
    (entry: ActionCatalogEntry) => {
      const action: ActionConfig = {
        id: `step-${Date.now()}`,
        action_type: entry.actionType,
        config: structuredClone(entry.defaultConfig),
      };
      dispatch(addFlowAction(action));
      dispatch(setAddStepPanelOpen(false));
    },
    [dispatch],
  );

  return (
    <Box
      as="aside"
      position="absolute"
      top="4"
      right="4"
      w="320px"
      bg="white"
      borderRadius="lg"
      border="1px solid"
      borderColor="gray.200"
      shadow="lg"
      zIndex="10"
      overflow="hidden"
      aria-label={t('flow.addStep.title')}
      data-testid="add-step-panel"
    >
      {/* Header */}
      <Flex align="center" justify="space-between" px="4" py="3" borderBottom="1px solid" borderColor="gray.100" bg="gray.50">
        <Heading size="sm">{t('flow.addStep.title')}</Heading>
        <Box
          as="button"
          onClick={onClose}
          aria-label={t('common.close')}
          cursor="pointer"
          fontSize="lg"
          color="gray.400"
          _hover={{ color: 'gray.600' }}
          bg="transparent"
          border="none"
          p="0"
          data-testid="add-step-close"
        >
          ✕
        </Box>
      </Flex>

      <Text fontSize="sm" color="gray.500" px="4" pt="3">
        {t('flow.addStep.subtitle')}
      </Text>

      <Box p="4" display="flex" flexDirection="column" gap="5" overflowY="auto" maxH="420px">
        {/* Logic / Transform */}
        <Box as="section">
          <Flex align="center" gap="2" mb="2">
            <Text>⚙️</Text>
            <Heading size="xs" textTransform="uppercase" letterSpacing="wide" color="gray.600">
              {t('flow.addStep.sections.transform')}
            </Heading>
          </Flex>
          <Flex as="ul" direction="column" gap="1" role="list" listStyleType="none">
            {LOGIC_STEPS.map((entry) => (
              <Box as="li" key={entry.actionType}>
                <Flex
                  as="button"
                  align="flex-start"
                  gap="3"
                  w="full"
                  p="2.5"
                  borderRadius="md"
                  cursor="pointer"
                  bg="transparent"
                  border="none"
                  _hover={{ bg: 'gray.50' }}
                  transition="background 0.15s"
                  onClick={() => handleAdd(entry)}
                  textAlign="left"
                  data-testid={`add-step-${entry.actionType}`}
                >
                  <Text fontSize="lg" mt="0.5">{entry.icon}</Text>
                  <Box>
                    <Text fontSize="sm" fontWeight="500">{entry.label}</Text>
                    <Text fontSize="xs" color="gray.500" lineHeight="1.4">{entry.description}</Text>
                  </Box>
                </Flex>
              </Box>
            ))}
          </Flex>
        </Box>

        {/* Egress / Destinations */}
        <Box as="section">
          <Flex align="center" gap="2" mb="2">
            <Text>📤</Text>
            <Heading size="xs" textTransform="uppercase" letterSpacing="wide" color="gray.600">
              {t('flow.addStep.sections.destination')}
            </Heading>
          </Flex>
          <Flex as="ul" direction="column" gap="1" role="list" listStyleType="none">
            {EGRESS_STEPS.map((entry) => (
              <Box as="li" key={entry.actionType}>
                <Flex
                  as="button"
                  align="flex-start"
                  gap="3"
                  w="full"
                  p="2.5"
                  borderRadius="md"
                  cursor="pointer"
                  bg="transparent"
                  border="none"
                  _hover={{ bg: 'gray.50' }}
                  transition="background 0.15s"
                  onClick={() => handleAdd(entry)}
                  textAlign="left"
                  data-testid={`add-step-${entry.actionType}`}
                >
                  <Text fontSize="lg" mt="0.5">{entry.icon}</Text>
                  <Box>
                    <Text fontSize="sm" fontWeight="500">{entry.label}</Text>
                    <Text fontSize="xs" color="gray.500" lineHeight="1.4">{entry.description}</Text>
                  </Box>
                </Flex>
              </Box>
            ))}
          </Flex>
        </Box>
      </Box>
    </Box>
  );
}
