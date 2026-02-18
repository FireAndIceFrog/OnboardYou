import { defineConfig, loadEnv } from 'vite';
import react from '@vitejs/plugin-react';
import { federation } from '@module-federation/vite';
import { resolve } from 'path';

export default defineConfig(({ mode }) => {
  const env = loadEnv(mode, process.cwd(), '');
  const configRemoteUrl =
    env.VITE_CONFIG_REMOTE_URL || 'http://localhost:5174';

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
          entry: `${configRemoteUrl}/remoteEntry.js`,
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
