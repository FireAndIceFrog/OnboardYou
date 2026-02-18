import { createSystem, defaultConfig, defineConfig } from '@chakra-ui/react';

const config = defineConfig({
  globalCss: {
    'html': {
      scrollBehavior: 'smooth',
    },
    'body': {
      textRendering: 'optimizeLegibility',
    },
    '#root': {
      minHeight: '100vh',
      display: 'flex',
      flexDirection: 'column',
    },
    /* React Flow SVG fix — prevent global resets from collapsing flow edges */
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
