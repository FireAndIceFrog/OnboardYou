import './i18n';
import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';

async function bootstrap() {
  if (import.meta.env.VITE_MOCK_MODE === 'true') {
    const { startMockServiceWorker } = await import('@/mocks');
    await startMockServiceWorker();
  }

  const { default: App } = await import('@/app/App');
  const root = document.getElementById('root');
  if (!root) throw new Error('Root element not found');

  createRoot(root).render(
    <StrictMode>
      <App />
    </StrictMode>,
  );
}

bootstrap();
