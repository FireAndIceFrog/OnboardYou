import React, { useCallback, useRef, useState, useEffect } from 'react';
import { useTranslation } from 'react-i18next';
import {
  Box,
  Button,
  Field,
  Flex,
  Heading,
  Input,
  NativeSelect,
  Table,
  Text,
  IconButton,
} from '@chakra-ui/react';
import { useSettingsState } from '../../../state/useSettingsState';
import { FIELD_TYPE_OPTIONS } from '../../../domain/types';
import type { SchemaFieldType } from '../../../domain/types';

/** Stable row identity that doesn't change when the user edits a field name. */
let nextRowId = 0;

type Row = { id: number; name: string; type: string };

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

  const schema =
    settings.authType === 'bearer'
      ? settings.bearer.schema
      : settings.oauth2.schema;

  const updateSchema = settings.authType === 'bearer' ? updateBearerSchema : updateOAuth2Schema;

  const [rows, setRows] = useState<Row[]>(() =>
    Object.entries(schema).map(([name, type]) => ({
      id: nextRowId++,
      name,
      type,
    })),
  );

  // Pending commit: when a handler wants to push to Redux it writes
  // the rows snapshot here and the effect below picks it up.
  const pendingCommit = useRef<Row[] | null>(null);

  // Track the last schema we pushed so we don't echo-back from Redux.
  const lastSyncedSchema = useRef(schema);

  // Mirror rows to a ref so event handlers can read current state
  // without closing over a stale snapshot.
  const rowsRef = useRef(rows);
  rowsRef.current = rows;

  /* ── Sync FROM Redux when it changes externally (e.g. fetch) ── */
  useEffect(() => {
    if (schema !== lastSyncedSchema.current) {
      lastSyncedSchema.current = schema;
      setRows(
        Object.entries(schema).map(([name, type]) => ({
          id: nextRowId++,
          name,
          type,
        })),
      );
    }
  }, [schema]);

  /* ── Flush pending commits to Redux outside render cycle ── */
  useEffect(() => {
    if (pendingCommit.current) {
      const toCommit = pendingCommit.current;
      pendingCommit.current = null;
      const next: Record<string, string> = {};
      for (const r of toCommit) next[r.name] = r.type;
      lastSyncedSchema.current = next;
      updateSchema(next);
    }
  });

  /* ── Row actions ────────────────────────────────────────── */
  const scheduleCommit = useCallback((updated: Row[]) => {
    pendingCommit.current = updated;
  }, []);

  const handleAdd = useCallback(() => {
    setRows((prev) => {
      let name = '';
      const existing = new Set(prev.map((r) => r.name));
      if (existing.has(name)) {
        let i = 1;
        while (existing.has(`field_${i}`)) i++;
        name = `field_${i}`;
      }
      const added = [...prev, { id: nextRowId++, name, type: 'string' }];
      scheduleCommit(added);
      return added;
    });
  }, [scheduleCommit]);

  const handleDelete = useCallback(
    (id: number) => {
      setRows((prev) => {
        const updated = prev.filter((r) => r.id !== id);
        scheduleCommit(updated);
        return updated;
      });
    },
    [scheduleCommit],
  );

  /** Update local state only — commit to Redux on blur. */
  const handleNameInput = useCallback(
    (id: number, newName: string) => {
      setRows((prev) => prev.map((r) => (r.id === id ? { ...r, name: newName } : r)));
    },
    [],
  );

  /** Commit the local names to Redux when a name field loses focus.
   *  This runs in an event handler (not during render) so dispatching
   *  directly is safe — no "setState during render" warning. */
  const handleNameBlur = useCallback(() => {
    const current = rowsRef.current;
    const next: Record<string, string> = {};
    for (const r of current) next[r.name] = r.type;
    lastSyncedSchema.current = next;
    updateSchema(next);
  }, [updateSchema]);

  const handleTypeChange = useCallback(
    (id: number, type: SchemaFieldType) => {
      setRows((prev) => {
        const updated = prev.map((r) => (r.id === id ? { ...r, type } : r));
        scheduleCommit(updated);
        return updated;
      });
    },
    [scheduleCommit],
  );

  return (
    <Box mb={5} as="fieldset">
      <Heading as="legend" size="lg" fontWeight="semibold" mb={1}>
        {t('settings.dynamic.title')}
      </Heading>
      <Text fontSize="sm" color="fg.muted" mb={5}>
        {t('settings.dynamic.description')}
      </Text>

      {/* Add button */}
      <Button
        variant="outline"
        size="sm"
        mb={3}
        onClick={handleAdd}
        aria-label={t('settings.dynamic.addField')}
      >
        {t('settings.dynamic.addField')}
      </Button>

      {/* Schema table */}
      {rows.length === 0 ? (
        <Text fontSize="sm" color="fg.muted" mb={4}>
          {t('settings.dynamic.emptySchema')}
        </Text>
      ) : (
        <Table.Root size="sm" mb={4}>
          <Table.Header>
            <Table.Row>
              <Table.ColumnHeader>{t('settings.dynamic.fieldName')}</Table.ColumnHeader>
              <Table.ColumnHeader>{t('settings.dynamic.fieldType')}</Table.ColumnHeader>
              <Table.ColumnHeader w="1px" />
            </Table.Row>
          </Table.Header>
          <Table.Body>
            {rows.map((row) => (
              <Table.Row key={row.id}>
                <Table.Cell>
                  <Input
                    size="sm"
                    value={row.name}
                    placeholder={t('settings.dynamic.fieldNamePlaceholder')}
                    onChange={(e) => handleNameInput(row.id, e.target.value)}
                    onBlur={handleNameBlur}
                    aria-label={t('settings.dynamic.fieldName')}
                  />
                </Table.Cell>
                <Table.Cell>
                  <NativeSelect.Root size="sm">
                    <NativeSelect.Field
                      value={row.type}
                      onChange={(e) =>
                        handleTypeChange(row.id, e.target.value as SchemaFieldType)
                      }
                      aria-label={t('settings.dynamic.fieldType')}
                    >
                      {FIELD_TYPE_OPTIONS.map((opt) => (
                        <option key={opt.value} value={opt.value}>
                          {opt.label}
                        </option>
                      ))}
                    </NativeSelect.Field>
                    <NativeSelect.Indicator />
                  </NativeSelect.Root>
                </Table.Cell>
                <Table.Cell>
                  <IconButton
                    aria-label={t('settings.dynamic.deleteField')}
                    variant="ghost"
                    colorPalette="red"
                    size="sm"
                    onClick={() => handleDelete(row.id)}
                  >
                    ✕
                  </IconButton>
                </Table.Cell>
              </Table.Row>
            ))}
          </Table.Body>
        </Table.Root>
      )}

      <Text fontSize="xs" color="fg.muted" mb={4}>
        {t('settings.dynamic.schemaHint')}
      </Text>

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
        </Box>
      )}
    </Box>
  );
}
