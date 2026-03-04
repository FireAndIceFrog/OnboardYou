import { useCallback } from 'react';
import { Box, Flex, Heading, Text, Button, Grid, Spinner } from '@chakra-ui/react';
import type { PlanSummary, ActionConfig } from '@/generated/api';
import { PlanFeatureCard } from './PlanFeatureCard';
import { PlanPreviewCard } from './PlanPreviewCard';

interface PlanSummaryViewProps {
  planSummary: PlanSummary;
  /** The manifest actions — used to derive feature enabled state */
  actions: ActionConfig[];
  /** Whether the plan is currently being saved */
  isSaving: boolean;
  /** Whether the plan is currently being generated */
  isGenerating: boolean;
  onToggleFeature: (featureId: string) => void;
  onApplyPlan: () => void;
  onMakeChanges: () => void;
}

/**
 * Main plan summary container matching the screenshot layout.
 * Shows headline, description, feature cards, preview, and action buttons.
 */
export function PlanSummaryView({
  planSummary,
  actions,
  isSaving,
  isGenerating,
  onToggleFeature,
  onApplyPlan,
  onMakeChanges,
}: PlanSummaryViewProps) {
  const handleApply = useCallback(() => onApplyPlan(), [onApplyPlan]);
  const handleChanges = useCallback(() => onMakeChanges(), [onMakeChanges]);

  if (isGenerating) {
    return (
      <Flex direction="column" align="center" justify="center" h="100%" gap="4" p="8">
        <Spinner size="xl" color="purple.500" />
        <Heading size="md" color="gray.700">Generating your plan…</Heading>
        <Text color="gray.500" textAlign="center">
          Our AI is analysing your data source and building the best pipeline for your needs.
          This usually takes a few seconds.
        </Text>
      </Flex>
    );
  }

  /**
   * Determine if a feature is enabled by checking whether ALL its linked
   * manifest actions have `disabled !== true`.
   */
  function isFeatureEnabled(featureActionIds: string[]): boolean {
    const linked = actions.filter((a) => featureActionIds.includes(a.id));
    return linked.length > 0 && linked.every((a) => !a.disabled);
  }

  return (
    <Flex
      direction="column"
      h="100%"
      overflowY="auto"
      bg="gray.50"
      data-testid="plan-summary-view"
    >
      <Box maxW="720px" mx="auto" w="100%" p="8">
        {/* Header */}
        <Box mb="6">
          <Heading size="lg" fontWeight="700" color="gray.900" mb="2">
            {planSummary.headline}
          </Heading>
          <Text color="gray.600" fontSize="md">
            {planSummary.description}
          </Text>
        </Box>

        {/* Feature cards */}
        <Box mb="8">
          <Text fontWeight="600" fontSize="md" color="gray.800" mb="3">
            How it will work
          </Text>
          <Grid templateColumns={{ base: '1fr', md: '1fr 1fr' }} gap="3">
            {planSummary.features.map((feature) => (
              <PlanFeatureCard
                key={feature.id}
                feature={feature}
                enabled={isFeatureEnabled(feature.actionIds)}
                onToggle={onToggleFeature}
              />
            ))}
          </Grid>
        </Box>

        {/* Preview */}
        <Box mb="8">
          <PlanPreviewCard preview={planSummary.preview} />
        </Box>

        {/* Action buttons */}
        <Flex direction="column" gap="3">
          <Button
            size="lg"
            w="100%"
            bg="purple.600"
            color="white"
            fontWeight="600"
            borderRadius="lg"
            _hover={{ bg: 'purple.700' }}
            onClick={handleApply}
            disabled={isSaving}
            data-testid="apply-plan-button"
          >
            {isSaving ? 'Saving…' : 'Looks Good, Start Syncing'}
          </Button>
          <Button
            size="lg"
            w="100%"
            variant="outline"
            borderColor="gray.300"
            color="gray.700"
            fontWeight="600"
            borderRadius="lg"
            _hover={{ bg: 'gray.100' }}
            onClick={handleChanges}
            data-testid="make-changes-button"
          >
            Make Changes
          </Button>
        </Flex>
      </Box>
    </Flex>
  );
}
