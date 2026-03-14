import { useTranslation } from 'react-i18next';
import { Box, Flex, Heading, Text, Button, chakra } from '@chakra-ui/react';
import { HR_SYSTEMS } from '../../domain/types';
import { useConnectionForm } from '../../state/useConnectionForm';
import { FieldError } from '../components';
import { FormField } from '@/shared/ui';
import { getConnectorFormComponent } from '../components/connectorForms';

const StyledButton = chakra('button');

/* ── Component ──────────────────────────────────────────── */

export function ConnectionDetailsScreen() {
  const {
    form,
    errors,
    isValid,
    config,
    handleSystemSelect,
    handleChange,
    handleConnectorChange,
    handleNext,
    handleBack,
    validateField,
  } = useConnectionForm();
  const { t } = useTranslation();

  const ConnectorForm = getConnectorFormComponent(form.system);

  return (
    <Box maxW="680px" mx="auto" py="8" px="6">
      <Box as="form" bg="white" borderRadius="lg" border="1px solid" borderColor="tertiary.200" p="6" shadow="sm" onSubmit={(e: React.FormEvent) => e.preventDefault()}>
        <Heading size="lg" mb="1" color="primary.500">{t('configDetails.connection.title')}</Heading>
        <Text fontSize="sm" color="tertiary.500" mb="6">{t('configDetails.connection.subtitle')}</Text>

        {/* System selector */}
        <Box mb="5">
          <Text fontSize="sm" fontWeight="600" mb="2" color="primary.500">{t('configDetails.connection.hrSystem')}</Text>
          <Box display="grid" gridTemplateColumns="repeat(auto-fill, minmax(140px, 1fr))" gap="3" border={errors.system ? '2px solid' : 'none'} borderColor="red.300" borderRadius="lg" p={errors.system ? '2' : '0'}>
            {HR_SYSTEMS.map((sys) => {
              const SysIcon = sys.icon;
              return (
                <StyledButton
                  key={sys.id}
                  type="button"
                  display="flex"
                  flexDirection="column"
                  alignItems="center"
                  gap="2"
                  p="4"
                  borderRadius="lg"
                  border="2px solid"
                  borderColor={form.system === sys.id ? 'secondary.500' : 'tertiary.200'}
                  bg={form.system === sys.id ? 'secondary.50' : 'white'}
                  cursor="pointer"
                  transition="all 0.15s"
                  _hover={{ borderColor: 'secondary.300' }}
                  onClick={() => handleSystemSelect(sys.id)}
                >
                  <Box color={form.system === sys.id ? 'secondary.500' : 'tertiary.500'}><SysIcon size="1.75em" /></Box>
                  <Text fontSize="sm" fontWeight="500" color="primary.500">{t(sys.nameKey)}</Text>
                </StyledButton>
              );
            })}
          </Box>
          <FieldError id="system-error" error={errors.system} />
        </Box>

        {/* Display name */}
        {form.system && (
          <Box mb="5">
            <FormField
              id="conn-display-name"
              label={t('configDetails.connection.displayName')}
              placeholder={t('configDetails.connection.displayNamePlaceholder')}
              helperText={t('configDetails.connection.displayNameHint')}
              value={form.displayName}
              onChange={handleChange('displayName')}
            />
          </Box>
        )}

        {/* Connector-specific fields — rendered by the registered form component */}
        {ConnectorForm && (
          <ConnectorForm
            form={form}
            errors={errors}
            config={config}
            onChange={handleConnectorChange}
            validateField={validateField}
          />
        )}
      </Box>

      {/* Actions */}
      <Flex justify="space-between" mt="6">
        <Button variant="outline" size="md" borderColor="tertiary.300" color="tertiary.600" onClick={handleBack}>
          {t('configDetails.connection.backButton')}
        </Button>
        <Button bg="primary.500" color="white" _hover={{ bg: 'primary.600' }} size="md" disabled={!isValid} onClick={handleNext}>
          {t('configDetails.connection.nextButton')}
        </Button>
      </Flex>
    </Box>
  );
}

/** @deprecated Use `ConnectionDetailsScreen` instead. Kept for backward compatibility during migration. */
export const ConnectionDetailsPage = ConnectionDetailsScreen;
