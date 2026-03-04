import { useCallback } from 'react';
import { Box, Flex, Heading, Text, Button, Grid, Spinner } from '@chakra-ui/react';
import type { PlanSummary, ActionConfig } from '@/generated/api';
import { PlanFeatureCard } from './PlanFeatureCard';
import { PlanPreviewCard } from './PlanPreviewCard';

interface PlanSummaryViewProps {
  planSummary: PlanSummary | null;
  /** The manifest actions — used to derive feature enabled state */
  actions: ActionConfig[];
  /** Whether the plan is currently being saved */
  isSaving: boolean;
  /** Whether the plan is currently being generated */
  isGenerating: boolean;
  /** Whether the plan is out of sync with the advanced config */
  isStale: boolean;
  /** Whether this is a brand-new unsaved config */
  isNewConfig: boolean;
  onToggleFeature: (featureId: string) => void;
  onApplyPlan: () => void;
  onMakeChanges: () => void;
  onGeneratePlan: () => void;
}

const FUNNY_LOADING_MESSAGES = [
  '🥧 Assembling the pie…',
  '🔮 Reading the data tea leaves…',
  '🤖 Teaching robots to be helpful…',
  '🧩 Fitting the puzzle pieces together…',
  '🚀 Warming up the engines…',
];

function getRandomLoadingMessage(): string {
  return FUNNY_LOADING_MESSAGES[Math.floor(Math.random() * FUNNY_LOADING_MESSAGES.length)];
}

/**
 * Main plan summary container.
 *
 * Three states:
 * 1. No plan + not generating → empty state with "Generate Plan" button
 * 2. Generating → loading spinner with funny message
 * 3. Plan exists → full plan view (with optional "Regenerate" if stale)
 */
export function PlanSummaryView({
  planSummary,
  actions,
  isSaving,
  isGenerating,
  isStale,
  isNewConfig,
  onToggleFeature,
  onApplyPlan,
  onMakeChanges,
  onGeneratePlan,
}: PlanSummaryViewProps) {
  const handleApply = useCallback(() => onApplyPlan(), [onApplyPlan]);
  const handleChanges = useCallback(() => onMakeChanges(), [onMakeChanges]);
  const handleGenerate = useCallback(() => onGeneratePlan(), [onGeneratePlan]);

  // ── State 2: Generating ──────────────────────────────────
  if (isGenerating) {
    return (
      <Flex direction="column" align="center" justify="center" h="100%" gap="4" p="8" data-testid="plan-generating">
        <Spinner size="xl" color="purple.500" />
        <Heading size="md" color="gray.700">
          {getRandomLoadingMessage()}
        </Heading>
        <Text color="gray.500" textAlign="center">
          Our AI is analysing your data source and building the best pipeline for your needs.
          This usually takes a few seconds.
        </Text>
      </Flex>
    );
  }

  // ── State 1: No plan yet ─────────────────────────────────
  if (!planSummary) {
    return (
      <Flex
        direction="column"
        align="center"
        justify="center"
        h="100%"
        gap="5"
        p="8"
        data-testid="plan-empty-state"
      >
        <Text fontSize="5xl">🥧</Text>
        <Heading size="lg" color="gray.800" textAlign="center">
          No plan yet
        </Heading>
        <Text color="gray.500" textAlign="center" maxW="400px">
          {isNewConfig
            ? 'Save your configuration first, then generate a plan to see a simple summary of how your pipeline will work.'
            : 'Generate a plan to see a simple summary of how your pipeline will work. Our AI will analyse your data source and build the best flow for you.'}
        </Text>
        <Button
          size="lg"
          bg="purple.600"
          color="white"
          fontWeight="600"
          borderRadius="lg"
          _hover={{ bg: 'purple.700' }}
          onClick={handleGenerate}
          disabled={isNewConfig}
          data-testid="generate-plan-button"
        >
          ✨ Generate Plan
        </Button>
      </Flex>
    );
  }

  // ── State 3: Plan exists ─────────────────────────────────

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
        {/* Stale banner */}
        {isStale && (
          <Flex
            align="center"
            justify="space-between"
            bg="orange.50"
            border="1px solid"
            borderColor="orange.200"
            borderRadius="lg"
            p="4"
            mb="6"
            data-testid="plan-stale-banner"
          >
            <Text color="orange.800" fontSize="sm" fontWeight="500">
              ⚠️ The pipeline has been modified. This summary may be out of date.
            </Text>
            <Button
              size="sm"
              bg="orange.500"
              color="white"
              fontWeight="600"
              borderRadius="md"
              _hover={{ bg: 'orange.600' }}
              onClick={handleGenerate}
              data-testid="regenerate-plan-button"
            >
              🔄 Regenerate
            </Button>
          </Flex>
        )}

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
