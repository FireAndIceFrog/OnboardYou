import { useGlobal, ErrorBoundary, Spinner } from "@/shared";
import { Suspense } from "react";
import { RemoteHandle } from "../domain/types";
import { RemoteLoadFallback } from "./RemoteFallback";

/**
 * Lives INSIDE the Suspense boundary so it re-renders once the
 * lazy module resolves. Injects platform globals synchronously
 * during render, before the remote component mounts.
 */
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
    <ErrorBoundary fallback={(_, reset) => <RemoteLoadFallback reset={reset} />}>
      <Suspense
        fallback={
          <div style={{ display: 'flex', justifyContent: 'center', padding: '4rem' }}>
            <Spinner size="lg" />
          </div>
        }
      >
        <RemoteInjector handle={handle} />
      </Suspense>
    </ErrorBoundary>
  );
}