import { Box, Flex, Text, Grid, Badge } from '@chakra-ui/react';
import { Tooltip } from '@/shared/ui/Tooltip';
import type { PlanPreview } from '@/generated/api';

/** Sentinel value the AI places in `after` for unmapped destination fields. */
const NEEDS_MAPPING_SENTINEL = '__NEEDS_MAPPING__';

interface PlanPreviewCardProps {
  preview: PlanPreview;
}

function RecordCard({
  label,
  data,
  borderColorValue,
  warnings,
}: {
  label: string;
  data: Record<string, string>;
  borderColorValue: string;
  warnings?: Map<string, string>;
}) {
  return (
    <Box
      flex="1"
      border="2px solid"
      borderColor={borderColorValue}
      borderRadius="lg"
      p="4"
      bg="white"
    >
      <Text fontSize="xs" fontWeight="600" color="gray.500" mb="3" textTransform="uppercase">
        {label}
      </Text>
      {Object.entries(data).map(([key, value]) => {
        const isUnmapped = value === NEEDS_MAPPING_SENTINEL;
        const warningMessage = warnings?.get(key);

        return (
          <Flex key={key} justify="space-between" py="1.5" borderBottom="1px solid" borderColor={isUnmapped ? 'orange.200' : 'gray.100'} bg={isUnmapped ? 'orange.50' : undefined} px={isUnmapped ? '2' : undefined} borderRadius={isUnmapped ? 'md' : undefined}>
            <Text fontSize="sm" color={isUnmapped ? 'orange.700' : 'gray.500'}>{key}</Text>
            {isUnmapped ? (
              <Tooltip content={warningMessage ?? 'No source column maps to this field. Manual mapping required.'} positioning={{ placement: 'top' }}>
                <Badge colorPalette="orange" fontSize="xs" variant="subtle" cursor="help" data-testid={`warning-badge-${key}`}>
                  ⚠ Needs mapping
                </Badge>
              </Tooltip>
            ) : (
              <Text fontSize="sm" fontWeight="500" color="gray.800">{value}</Text>
            )}
          </Flex>
        );
      })}
    </Box>
  );
}

/**
 * Side-by-side before/after employee display with arrow between.
 * Left = grey border (source system), Right = purple border (your app).
 *
 * Destination fields with the `__NEEDS_MAPPING__` sentinel value are
 * highlighted with an amber warning badge to prompt user intervention.
 */
export function PlanPreviewCard({ preview }: PlanPreviewCardProps) {
  // Build a lookup from field name → warning message
  const warningMap = new Map(
    (preview.warnings ?? []).map((w) => [w.field, w.message]),
  );

  // Ensure every warned field appears in `after` with the sentinel value,
  // even if the AI omitted it or set a different value.
  const afterWithWarnings = { ...preview.after };
  for (const field of warningMap.keys()) {
    if (afterWithWarnings[field] !== NEEDS_MAPPING_SENTINEL) {
      afterWithWarnings[field] = NEEDS_MAPPING_SENTINEL;
    }
  }

  return (
    <Box data-testid="plan-preview-card">
      <Text fontWeight="600" fontSize="md" color="gray.800" mb="3">
        Preview
      </Text>
      <Grid templateColumns={{ base: '1fr', md: '1fr auto 1fr' }} gap="4" alignItems="center">
        <RecordCard
          label={preview.sourceLabel}
          data={preview.before}
          borderColorValue="gray.300"
        />

        {/* Arrow */}
        <Flex
          display={{ base: 'none', md: 'flex' }}
          direction="column"
          align="center"
          justify="center"
          color="purple.400"
          fontSize="2xl"
        >
          <Text>→</Text>
        </Flex>

        <RecordCard
          label={preview.targetLabel}
          data={afterWithWarnings}
          borderColorValue="purple.400"
          warnings={warningMap}
        />
      </Grid>

      {/* Summary warning banner when there are unmapped fields */}
      {warningMap.size > 0 && (
        <Box mt="4" p="3" bg="orange.50" border="1px solid" borderColor="orange.200" borderRadius="md" data-testid="preview-warnings-banner">
          <Text fontSize="sm" fontWeight="600" color="orange.800" mb="1">
            ⚠️ {warningMap.size} field{warningMap.size > 1 ? 's' : ''} could not be auto-mapped
          </Text>
          <Text fontSize="xs" color="orange.700">
            The highlighted fields in the destination have no matching source column.
            Switch to Advanced mode to configure manual mappings.
          </Text>
        </Box>
      )}
    </Box>
  );
}
