import { createSystem, defaultConfig, defineConfig } from '@chakra-ui/react';

/**
 * Config package theme — mirrors the platform theme.
 * In production the config remote renders inside the platform's ChakraProvider,
 * so this only matters for standalone dev mode (`pnpm dev:config`).
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
