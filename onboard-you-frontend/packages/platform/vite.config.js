import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import federation from '@originjs/vite-plugin-federation';
import { resolve } from 'path';
export default defineConfig({
    plugins: [
        react(),
        federation({
            name: 'platform',
            remotes: {
                configApp: 'http://localhost:5174/assets/remoteEntry.js',
            },
            shared: {
                react: { singleton: true, requiredVersion: false },
                'react-dom': { singleton: true, requiredVersion: false },
                'react-router-dom': { singleton: true, requiredVersion: false },
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
        target: 'esnext',
        minify: false,
        cssCodeSplit: false,
        outDir: 'dist',
        sourcemap: true,
    },
});
