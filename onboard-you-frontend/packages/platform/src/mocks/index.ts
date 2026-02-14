export async function startMockServiceWorker() {
  if (import.meta.env.VITE_MOCK_MODE === 'true') {
    const { worker } = await import('./browser');
    await worker.start({
      onUnhandledRequest: 'bypass',
      serviceWorker: {
        url: '/mockServiceWorker.js',
      },
    });
    console.log('[MSW] Mock mode active');
  }
}
