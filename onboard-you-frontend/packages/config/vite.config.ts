import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import { federation } from '@module-federation/vite';
import cssInjectedByJsPlugin from 'vite-plugin-css-injected-by-js';
import { resolve } from 'path';

export default defineConfig({
  plugins: [
    react(),
    cssInjectedByJsPlugin({ relativeCSSInjection: true }),
    federation({
      name: 'configApp',
      filename: 'remoteEntry.js',
      exposes: {
        './App': './src/app/App.tsx',
        './ConfigListScreen': './src/features/config-list/ui/ConfigListScreen.tsx',
        './ConfigDetailsPage': './src/features/config-details/ui/ConfigDetailsPage.tsx',
      },
      shared: {
        react: { singleton: true },
        'react-dom': { singleton: true },
        'react-router-dom': { singleton: true },
      },
      bundleAllCSS: true,
    }),
  ],
  resolve: {
    alias: {
      '@': resolve(__dirname, 'src'),
    },
  },
  css: {
    preprocessorOptions: {
      scss: {
        api: 'modern-compiler',
      },
    },
  },
  server: {
    port: 5174,
    strictPort: true,
    origin: 'http://localhost:5174',
  },
  build: {
    modulePreload: false,
    target: 'chrome89',
    minify: false,
    cssCodeSplit: true,
    outDir: 'dist',
    sourcemap: true,
  },
});
