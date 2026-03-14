/**
 * Shared Chakra style-prop constants used across config-details components.
 * Centralised here to avoid duplication and keep visual consistency.
 */
export const inputStyles = {
  fontSize: 'sm',
  borderColor: 'tertiary.200',
  bg: 'white',
  _focus: { borderColor: 'secondary.500', boxShadow: '0 0 0 1px var(--chakra-colors-secondary-500)' },
} as const;

export const selectStyles = {
  ...inputStyles,
  cursor: 'pointer',
} as const;
