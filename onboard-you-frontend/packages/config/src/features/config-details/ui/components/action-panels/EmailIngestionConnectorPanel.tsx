import { Box, Flex, Heading, Text, Textarea, Input } from '@chakra-ui/react';
import { EnvelopeIcon } from '@/shared/ui';
import type { ActionEditorProps } from './registry';

/**
 * Custom editor panel for `email_ingestion_connector` actions.
 *
 * Displayed inline in the pipeline canvas when an email ingestion step is
 * selected.  Allows the user to manage the allowed senders allowlist and
 * the optional subject filter directly in the canvas without leaving the
 * pipeline builder flow.
 */
export function EmailIngestionConnectorPanel({ config, onChange }: ActionEditorProps) {
  const allowedSenders: string = Array.isArray(config.allowed_senders)
    ? (config.allowed_senders as string[]).join(', ')
    : typeof config.allowed_senders === 'string'
      ? config.allowed_senders
      : '';
  const subjectFilter: string = typeof config.subject_filter === 'string' ? config.subject_filter : '';

  return (
    <Box>
      <Flex align="center" gap="2" mb="4">
        <EnvelopeIcon size="1em" />
        <Heading size="sm" fontWeight="700">Email Ingestion</Heading>
      </Flex>

      {/* Allowed senders */}
      <Box mb="4">
        <Text fontSize="sm" fontWeight="600" mb="1">
          Allowed Senders <Text as="span" color="red.500" aria-hidden>*</Text>
        </Text>
        <Textarea
          value={allowedSenders}
          placeholder="hr@acme.com, @partner.com"
          fontSize="sm"
          rows={3}
          onChange={(e) => {
            const senders = e.target.value
              .split(',')
              .map((s) => s.trim())
              .filter(Boolean);
            onChange('allowed_senders', senders);
          }}
        />
        <Text fontSize="xs" color="gray.400" mt="1">
          Comma-separated email addresses or domain globs.
        </Text>
      </Box>

      {/* Subject filter */}
      <Box mb="2">
        <Text fontSize="sm" fontWeight="600" mb="1">
          Subject Filter{' '}
          <Text as="span" fontSize="xs" fontWeight="400" color="gray.400">(optional)</Text>
        </Text>
        <Input
          value={subjectFilter}
          placeholder="e.g. Monthly Roster"
          fontSize="sm"
          onChange={(e) => onChange('subject_filter', e.target.value || null)}
        />
        <Text fontSize="xs" color="gray.400" mt="1">
          Only process emails whose subject contains this text (case-insensitive).
        </Text>
      </Box>
    </Box>
  );
}
