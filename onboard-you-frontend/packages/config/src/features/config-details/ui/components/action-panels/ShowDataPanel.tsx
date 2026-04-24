import { useEffect, useState } from 'react';
import { useParams } from 'react-router-dom';
import { Box, Heading, Text, Table, Spinner, Alert } from '@chakra-ui/react';
import { useTranslation } from 'react-i18next';
import { useAppSelector } from '@/store';
import { selectSelectedNode } from '../../../state/configDetailsSlice';
import { getShowData } from '@/generated/api';
import type { ShowDataResponse } from '@/generated/api';
import type { ActionEditorProps } from './registry';

export function ShowDataPanel(_props: ActionEditorProps) {
  const { t } = useTranslation();
  const { customerCompanyId } = useParams<{ customerCompanyId: string }>();
  const selectedNode = useAppSelector(selectSelectedNode);
  const nodeData = selectedNode?.data as Record<string, unknown> | undefined;
  const actionId = (nodeData?.actionId as string) ?? selectedNode?.id ?? '';

  const [data, setData] = useState<ShowDataResponse | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!customerCompanyId || !actionId) return;

    setLoading(true);
    setError(null);

    getShowData({
      path: { customer_company_id: customerCompanyId, action_id: actionId },
    })
      .then((res: { data?: ShowDataResponse }) => {
        if (res.data) {
          setData(res.data);
        } else {
          setError(
            t('flow.showData.notFound', 'No output yet — run the pipeline first.'),
          );
        }
      })
      .catch(() => {
        setError(
          t('flow.showData.notFound', 'No output yet — run the pipeline first.'),
        );
      })
      .finally(() => setLoading(false));
  }, [customerCompanyId, actionId, t]);

  return (
    <Box as="section" data-testid="show-data-panel">
      <Heading as="h3" fontSize="md" fontWeight="600" mb="2">
        {t('flow.showData.title', 'Data Preview')}
      </Heading>
      <Text fontSize="xs" color="gray.500" mb="3">
        {t(
          'flow.showData.description',
          'Shows the data snapshot saved by this step after the last successful pipeline run.',
        )}
      </Text>

      {loading && (
        <Box display="flex" justifyContent="center" py="6">
          <Spinner size="sm" />
        </Box>
      )}

      {!loading && error && (
        <Alert.Root status="info" borderRadius="md">
          <Alert.Indicator />
          <Alert.Description fontSize="xs">{error}</Alert.Description>
        </Alert.Root>
      )}

      {!loading && data && data.rows.length === 0 && (
        <Text fontSize="sm" color="gray.500">
          {t('flow.showData.empty', 'The output file is empty.')}
        </Text>
      )}

      {!loading && data && data.rows.length > 0 && (
        <Box overflowX="auto" maxH="320px" overflowY="auto">
          <Table.Root size="sm" variant="outline" borderRadius="10px" overflow="hidden">
            <Table.Header>
              <Table.Row>
                {data.columns.map((col: string) => (
                  <Table.ColumnHeader key={col} fontSize="xs" fontWeight="600" color="gray.500" whiteSpace="nowrap">
                    {col}
                  </Table.ColumnHeader>
                ))}
              </Table.Row>
            </Table.Header>
            <Table.Body>
              {data.rows.slice(0, 50).map((row: Record<string, unknown>, i: number) => (
                <Table.Row key={i}>
                  {data.columns.map((col: string) => (
                    <Table.Cell key={col} fontSize="xs" whiteSpace="nowrap" maxW="160px" overflow="hidden" textOverflow="ellipsis">
                      {String(row[col] ?? '')}
                    </Table.Cell>
                  ))}
                </Table.Row>
              ))}
            </Table.Body>
          </Table.Root>
          {data.rows.length > 50 && (
            <Text fontSize="xs" color="gray.400" mt="2" textAlign="center">
              {t('flow.showData.truncated', `Showing first 50 of ${data.rows.length} rows`)}
            </Text>
          )}
        </Box>
      )}
    </Box>
  );
}
