import { Outlet } from 'react-router-dom';
import { useAppSelector } from '@/store';
import { selectSidebarCollapsed } from '@/features/layout/state/layoutSlice';
import { Header } from './Header';
import { Sidebar } from './Sidebar';
import styles from './AppLayout.module.scss';

export function AppLayout() {
  const sidebarCollapsed = useAppSelector(selectSidebarCollapsed);

  return (
    <div className={styles.layout}>
      <Header />
      <Sidebar />
      <main
        className={[
          styles.main,
          sidebarCollapsed ? styles['main--collapsed'] : '',
        ]
          .filter(Boolean)
          .join(' ')}
      >
        <Outlet />
      </main>
    </div>
  );
}
