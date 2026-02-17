import { defineConfig } from 'vitest/config';

export default defineConfig({
  test: {
    include: ['src/services/**/*.smoke.ts'],
    testTimeout: 30_000,
    setupFiles: ['src/setup.ts'],
  },
});
