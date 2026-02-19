import { useEffect } from 'react';
import { useTranslation } from 'react-i18next';
import { useNavigate } from 'react-router-dom';
import { Box, Flex, Heading, Text, Input, Spinner } from '@chakra-ui/react';
import { Button } from '@chakra-ui/react';
import { useAppDispatch, useAppSelector } from '@/store';
import { fetchConfigs, setSearchQuery, selectConfigList, selectFilteredConfigs } from '../../state/configListSlice';
import { ConfigListItem } from '../components';

function ConfigListScreenInner() {
  const { t } = useTranslation();
  const dispatch = useAppDispatch();
  const state = useAppSelector(selectConfigList);
  const filteredConfigs = useAppSelector(selectFilteredConfigs);
  const navigate = useNavigate();

  useEffect(() => {
    dispatch(fetchConfigs());
  }, [dispatch]);

  return (
    <Box maxW="1200px" mx="auto" py="8" px="6">
      <Box role="tabpanel" id="tabpanel-portfolio">
        {/* Header */}
        <Flex justify="space-between" align="center" mb="6">
          <Heading size="2xl">{t('configList.title')}</Heading>
          <Button colorPalette="blue" size="md" onClick={() => navigate('new')}>
            ＋ {t('configList.newConnection')}
          </Button>
        </Flex>

        {/* Search */}
        <Box position="relative" mb="6">
          <label htmlFor="config-search" style={{ position: 'absolute', width: 1, height: 1, overflow: 'hidden', clip: 'rect(0,0,0,0)', whiteSpace: 'nowrap' as const }}>{t('configList.search.placeholder')}</label>
          <Box position="absolute" left="3" top="50%" transform="translateY(-50%)" zIndex="1" aria-hidden="true">🔍</Box>
          <Input
            id="config-search"
            type="text"
            placeholder={t('configList.search.placeholder')}
            value={state.searchQuery}
            onChange={(e) => dispatch(setSearchQuery(e.target.value))}
            pl="10"
            bg="white"
            borderColor="gray.200"
            _focus={{ borderColor: 'blue.500', boxShadow: '0 0 0 1px var(--chakra-colors-blue-500)' }}
          />
        </Box>

        {/* Loading */}
        {state.isLoading && (
          <Flex direction="column" align="center" gap="3" py="12" data-testid="config-list-loading">
            <Spinner size="lg" />
            <Text color="gray.500">{t('configList.loading')}</Text>
          </Flex>
        )}

        {/* Error */}
        {state.error && !state.isLoading && (
          <Flex direction="column" align="center" gap="2" py="12" data-testid="config-list-error">
            <Text fontSize="2xl">⚠️</Text>
            <Text color="red.500">{state.error}</Text>
          </Flex>
        )}

        {/* Empty */}
        {!state.isLoading && !state.error && filteredConfigs.length === 0 && (
          <Flex direction="column" align="center" gap="3" py="16" data-testid="config-list-empty">
            <Text fontSize="3xl">🔗</Text>
            <Heading size="md" color="gray.700">{t('configList.empty.title')}</Heading>
            <Text color="gray.500" fontSize="sm">
              {state.searchQuery
                ? t('configList.empty.noResults')
                : t('configList.empty.noData')}
            </Text>
          </Flex>
        )}

        {/* Grid */}
        {!state.isLoading && !state.error && filteredConfigs.length > 0 && (
          <Box
            display="grid"
            gridTemplateColumns={{ base: '1fr', md: 'repeat(2, 1fr)', lg: 'repeat(3, 1fr)' }}
            gap="5"
            data-testid="config-list-grid"
          >
            {filteredConfigs.map((config) => (
              <ConfigListItem key={config.customerCompanyId} config={config} />
            ))}
          </Box>
        )}
      </Box>
    </Box>
  );
}

export function ConfigListScreen() {
  return <ConfigListScreenInner />;
}
