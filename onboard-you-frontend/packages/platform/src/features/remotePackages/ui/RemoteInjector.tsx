import { Suspense } from 'react';
import { Center, Spinner } from '@chakra-ui/react';
import { useGlobal } from '@/shared/hooks/useGlobal';
import { ErrorBoundary } from '@/shared/ui/ErrorBoundary';
import type { RemoteHandle } from '../domain/types';
import { RemoteLoadFallback } from './RemoteFallback';

function RemoteInjector({ handle }: { handle: RemoteHandle }) {
  const globals = useGlobal();
  const setGlobal = handle.getSetGlobalValue();

  if (handle.injectGlobals && setGlobal) {
    setGlobal(globals);
  }

  return <handle.Component />;
}

export function RemoteShell({ handle }: { handle: RemoteHandle }) {
  return (
    <ErrorBoundary
      fallback={(_, reset) => <RemoteLoadFallback reset={reset} />}
    >
      <Suspense
        fallback={
          <Center p={16}>
            <Spinner size="lg" />
          </Center>
        }
      >
        <RemoteInjector handle={handle} />
      </Suspense>
    </ErrorBoundary>
  );
}
