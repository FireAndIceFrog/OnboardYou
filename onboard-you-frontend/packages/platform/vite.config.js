import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import { federation } from '@module-federation/vite';
import { resolve } from 'path';
export default defineConfig({
    plugins: [
        react(),
        federation({
            name: 'platform',
            remotes: {
                configApp: {
                    type: 'module',
                    name: 'configApp',
                    entry: 'http://localhost:5174/remoteEntry.js',
                },
            },
            shared: {
                react: { singleton: true },
                'react-dom': { singleton: true },
                'react-router-dom': { singleton: true },
            },
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
});
