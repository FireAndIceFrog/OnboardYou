import { createSystem, defaultConfig, defineConfig } from '@chakra-ui/react';

const config = defineConfig({
  globalCss: {
    'html': {
      height: '100%',
      scrollBehavior: 'smooth',
    },
    'body': {
      height: '100%',
      textRendering: 'optimizeLegibility',
      display: 'flex',
      flexDirection: 'column',
      margin: 0,
    },
    '#root': {
      height: '100%',
      display: 'flex',
      flexDirection: 'column',
      minHeight: '0',
      flex: '1 0 auto',
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
