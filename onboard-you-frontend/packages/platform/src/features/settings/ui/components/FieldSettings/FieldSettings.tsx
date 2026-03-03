import React, { useState, useEffect, useCallback } from 'react';
import { useTranslation } from 'react-i18next';
import { Box, Field, Heading, Input, Text, Textarea } from '@chakra-ui/react';
import { FieldError } from '../FieldError/FieldError';
import { useSettingsState } from '../../../state/useSettingsState';

export function FieldSettings() {
  const { t } = useTranslation();
  const {
    showAdvanced,
    settings,
    updateBearerSchema,
    updateOAuth2Schema,
    updateBearerBodyPath,
    updateOAuth2BodyPath,
  } = useSettingsState();

  const [schemaText, setSchemaText] = useState('');
  const [schemaError, setSchemaError] = useState('');

  useEffect(() => {
    const obj =
      settings.authType === 'bearer'
        ? settings.bearer.schema
        : settings.oauth2.schema;
    setSchemaText(JSON.stringify(obj || {}, null, 2));
    setSchemaError('');
  }, [settings.authType, settings.bearer.schema, settings.oauth2.schema]);

  const handleSchemaChange = useCallback(
    (e: React.ChangeEvent<HTMLTextAreaElement>) => {
      const txt = e.target.value;
      setSchemaText(txt);
      try {
        const parsed = JSON.parse(txt);
        if (parsed && typeof parsed === 'object' && !Array.isArray(parsed)) {
          if (settings.authType === 'bearer') {
            updateBearerSchema(parsed as Record<string, string>);
          } else {
            updateOAuth2Schema(parsed as Record<string, string>);
          }
          setSchemaError('');
        } else {
          throw new Error('schema must be an object');
        }
      } catch (err) {
        setSchemaError(
          t('validation.invalidJson', { defaultValue: 'Invalid JSON' }),
        );
      }
    },
    [settings.authType, updateBearerSchema, updateOAuth2Schema, t],
  );

  return (
    <Box mb={5} as="fieldset">
      <Heading as="legend" size="lg" fontWeight="semibold" mb={1}>
        {t('settings.dynamic.title')}
      </Heading>
      <Text fontSize="sm" color="fg.muted" mb={5}>
        {t('settings.dynamic.description')}
      </Text>

      <Field.Root invalid={!!schemaError}>
        <Field.Label>{t('settings.dynamic.schema')}</Field.Label>
        <Textarea minH="120px" value={schemaText} onChange={handleSchemaChange} />
        {schemaError && <FieldError id="schema-error" error={schemaError} />}
        <Field.HelperText>
          {t('settings.dynamic.schemaHint')}
        </Field.HelperText>
      </Field.Root>

      {showAdvanced && (
        <Box mt={4}>
          <Field.Root>
            <Field.Label>{t('settings.dynamic.bodyPath')}</Field.Label>
            <Input
              type="text"
              placeholder={t('settings.dynamic.bodyPathPlaceholder', {
                defaultValue: 'e.g. data.items',
              })}
              value={
                settings.authType === 'bearer'
                  ? settings.bearer.bodyPath
                  : settings.oauth2.bodyPath
              }
              onChange={
                settings.authType === 'bearer'
                  ? updateBearerBodyPath
                  : updateOAuth2BodyPath
              }
            />
          </Field.Root>
        </Box>)}
    </Box>
  );
}
