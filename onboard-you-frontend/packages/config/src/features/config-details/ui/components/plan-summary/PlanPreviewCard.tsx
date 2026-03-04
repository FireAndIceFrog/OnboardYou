import { Box, Flex, Text, Grid } from '@chakra-ui/react';
import type { PlanPreview } from '@/generated/api';

interface PlanPreviewCardProps {
  preview: PlanPreview;
}

function RecordCard({
  label,
  data,
  borderColorValue,
}: {
  label: string;
  data: Record<string, string>;
  borderColorValue: string;
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
      {Object.entries(data).map(([key, value]) => (
        <Flex key={key} justify="space-between" py="1.5" borderBottom="1px solid" borderColor="gray.100">
          <Text fontSize="sm" color="gray.500">{key}</Text>
          <Text fontSize="sm" fontWeight="500" color="gray.800">{value}</Text>
        </Flex>
      ))}
    </Box>
  );
}

/**
 * Side-by-side before/after employee display with arrow between.
 * Left = grey border (source system), Right = purple border (your app).
 */
export function PlanPreviewCard({ preview }: PlanPreviewCardProps) {
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
          data={preview.after}
          borderColorValue="purple.400"
        />
      </Grid>
    </Box>
  );
}
