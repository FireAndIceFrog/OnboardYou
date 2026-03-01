import { defineConfig, loadEnv } from 'vite';
import react from '@vitejs/plugin-react';
import { federation } from '@module-federation/vite';
import { resolve } from 'path';

export default defineConfig(({ mode }) => {
  const env = loadEnv(mode, process.cwd(), '');
  // remote entry for config app.  in prod we sync the bundle to
  // <bucket>/config/, so the URL must include the `/config` prefix.
  // default for local development should match the dev server's base.
  const remoteUrl =
    env.VITE_REMOTE_URL || 'http://localhost:5174';

  return {
  plugins: [
    react(),
    federation({
      name: 'platform',
      filename: 'remoteEntry.js',
      remotes: {
        configApp: {
          type: 'module',
          name: 'configApp',
          entry: `${remoteUrl}/config/remoteEntry.js`,
          entryGlobalName: 'configApp',
          shareScope: 'default',
        },
      },
      shared: {
        react: { singleton: true },
        'react-dom': { singleton: true },
        'react-router-dom': { singleton: true },
        '@reduxjs/toolkit': { singleton: true },
        'react-redux': { singleton: true },
        'react-i18next': { singleton: true },
        i18next: { singleton: true },
        '@chakra-ui/react': { singleton: true },
        '@emotion/react': { singleton: true },
      },
    }),
  ],
  resolve: {
    alias: {
      '@': resolve(__dirname, 'src'),
    },
  },
  server: {
    port: 5173,
    strictPort: false,
  },
  build: {
    modulePreload: false,
    target: 'chrome89',
    minify: false,
    cssCodeSplit: false,
    outDir: 'dist',
    sourcemap: true,
  },
};
});
