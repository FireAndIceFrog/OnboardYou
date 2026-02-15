import '@/i18n';
import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import { startMockServiceWorker } from '@/mocks';
import App from '@/app/App';

async function bootstrap() {
  await startMockServiceWorker();

  const root = document.getElementById('root');
  if (!root) throw new Error('Root element not found');

  createRoot(root).render(
    <StrictMode>
      <App />
    </StrictMode>,
  );
}

bootstrap();
