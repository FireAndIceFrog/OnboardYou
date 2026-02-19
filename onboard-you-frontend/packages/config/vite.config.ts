import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import { federation } from '@module-federation/vite';
import { resolve } from 'path';

export default defineConfig({
  plugins: [
    react(),
    federation({
      name: 'configApp',
      filename: 'remoteEntry.js',
      exposes: {
        './App': './src/app/App.tsx',
        './ConfigListScreen': './src/features/config-list/ui/ConfigListScreen.tsx',
        './ConfigDetailsPage': './src/features/config-details/ui/screens/ConfigDetailsScreen.tsx',
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
