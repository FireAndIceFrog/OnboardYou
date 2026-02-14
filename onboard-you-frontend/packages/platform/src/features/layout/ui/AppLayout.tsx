import { Outlet } from 'react-router-dom';
import { useContext } from 'react';
import { LayoutProvider } from '@/features/layout/state/LayoutProvider';
import { LayoutContext } from '@/features/layout/state/LayoutContext';
import { Header } from './Header';
import { Sidebar } from './Sidebar';
import styles from './AppLayout.module.scss';

function LayoutShell() {
  const layoutCtx = useContext(LayoutContext);

  if (!layoutCtx) {
    throw new Error('LayoutShell must be used within a LayoutProvider');
  }

  const { state } = layoutCtx;

  return (
    <div className={styles.layout}>
      <Header />
      <Sidebar />
      <main
        className={[
          styles.main,
          state.sidebarCollapsed ? styles['main--collapsed'] : '',
        ]
          .filter(Boolean)
          .join(' ')}
      >
        <Outlet />
      </main>
    </div>
  );
}

export function AppLayout() {
  return (
    <LayoutProvider>
      <LayoutShell />
    </LayoutProvider>
  );
}
