import { lazy} from 'react';
import type { RouteObject } from 'react-router-dom';
import remotePackages from '../remotePackages';
import { RemoteModule, RemotePackageConfig, RemoteHandle, SetGlobalValueFn } from '../domain/types';
import { RemoteShell } from '../ui/components/RemoteInjector';


/* ── Static loader registry ────────────────────────────────── *
 * One line per known MF remote. The JSON config activates them;
 * this map provides the actual import statements so the bundler
 * and Module Federation plugin can resolve them statically.
 *
 * To add a new remote:
 *   1. Add a JSON entry to remotePackages.json
 *   3. Declare the MF remote in vite.config.ts
 *   4. Add a type declaration in src/types/remote.d.ts
 */

/* ── Handle factory (called once per remote at module load) ── */

function createRemoteHandle(config: RemotePackageConfig): RemoteHandle {
  let _setGlobalValue: SetGlobalValueFn | null = null;

  const Component = lazy(async () => {
    const loader = config.entry();
    if (!loader) {
      throw new Error(
        `No loader registered for remote package "${config.package}". ` +
        'Add one to LOADERS in remoteRouteBuilder.tsx.',
      );
    }

    const m = await loader;
    await config?.translations?.(); // trigger loading of translations if provided

    if (config.useGlobal && typeof m.setGlobalValue === 'function') {
      _setGlobalValue = m.setGlobalValue;
    }

    return { default: m.Routes };
  });

  return {
    Component,
    getSetGlobalValue: () => _setGlobalValue,
    injectGlobals: config.useGlobal,
  };
}

/* ── Public API ────────────────────────────────────────────── */

export function buildRemoteRoutes(): RouteObject[] {
  return (remotePackages as RemotePackageConfig[]).map((config) => {
    const handle = createRemoteHandle(config);
    return {
      path: config.path,
      element: <RemoteShell handle={handle} />,
    };
  });
}
