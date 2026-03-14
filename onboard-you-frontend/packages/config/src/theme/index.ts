import { createSystem, defaultConfig, defineConfig } from '@chakra-ui/react';

/**
 * Config package theme — mirrors the platform theme.
 * In production the config remote renders inside the platform's ChakraProvider,
 * so this only matters for standalone dev mode (`pnpm dev:config`).
 *
 * 3 colour palette:
 *   Primary   → #1a365d  (deep navy)
 *   Secondary → #2563eb  (blue-600)
 *   Tertiary  → #64748b  (slate-500)
 */
const config = defineConfig({
  globalCss: {
    '.react-flow svg, .react-flow__edges svg': {
      display: 'initial',
      maxWidth: 'initial',
    },
  },
  theme: {
    tokens: {
      colors: {
        primary: {
          50: { value: '#e8edf4' },
          100: { value: '#c5d0e3' },
          200: { value: '#9eb2d0' },
          300: { value: '#7793bd' },
          400: { value: '#597daf' },
          500: { value: '#1a365d' },
          600: { value: '#162d4f' },
          700: { value: '#112340' },
          800: { value: '#0d1a30' },
          900: { value: '#081020' },
        },
        secondary: {
          50: { value: '#eff6ff' },
          100: { value: '#dbeafe' },
          200: { value: '#bfdbfe' },
          300: { value: '#93c5fd' },
          400: { value: '#60a5fa' },
          500: { value: '#2563eb' },
          600: { value: '#1d4ed8' },
          700: { value: '#1e40af' },
          800: { value: '#1e3a8a' },
          900: { value: '#172554' },
        },
        tertiary: {
          50: { value: '#f8fafc' },
          100: { value: '#f1f5f9' },
          200: { value: '#e2e8f0' },
          300: { value: '#cbd5e1' },
          400: { value: '#94a3b8' },
          500: { value: '#64748b' },
          600: { value: '#475569' },
          700: { value: '#334155' },
          800: { value: '#1e293b' },
          900: { value: '#0f172a' },
        },
      },
      fonts: {
        heading: {
          value:
            "'Inter', system-ui, -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif",
        },
        body: {
          value:
            "'Inter', system-ui, -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif",
        },
        mono: {
          value:
            "'JetBrains Mono', ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, 'Liberation Mono', 'Courier New', monospace",
        },
      },
    },
  },
});

export const system = createSystem(defaultConfig, config);
