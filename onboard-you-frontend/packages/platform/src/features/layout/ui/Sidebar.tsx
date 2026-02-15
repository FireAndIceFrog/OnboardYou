import { NavLink, useLocation } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { useAppSelector, useAppDispatch } from '@/store';
import {
  selectLayout,
  setSidebarOpen,
  toggleSidebarCollapsed,
} from '@/features/layout/state/layoutSlice';
import { NAVIGATION_ITEMS } from '@/features/layout/domain/navigation';
import styles from './Sidebar.module.scss';

export function Sidebar() {
  const { t } = useTranslation();
  const dispatch = useAppDispatch();
  const { sidebarOpen, sidebarCollapsed } = useAppSelector(selectLayout);
  const location = useLocation();

  const isActive = (path: string) => {
    if (path === '/') return location.pathname === '/';
    return location.pathname.startsWith(path);
  };

  return (
    <>
      {/* Mobile overlay backdrop */}
      {sidebarOpen && (
        <div
          className={styles.overlay}
          onClick={() => dispatch(setSidebarOpen(false))}
          aria-hidden="true"
        />
      )}

      <aside
        className={[
          styles.sidebar,
          sidebarCollapsed ? styles['sidebar--collapsed'] : '',
          sidebarOpen ? styles['sidebar--open'] : '',
        ]
          .filter(Boolean)
          .join(' ')}
      >
        <nav className={styles.nav} aria-label="Main navigation">
          {NAVIGATION_ITEMS.map((item) => (
            <NavLink
              key={item.id}
              to={item.path}
              className={[
                styles['nav-item'],
                isActive(item.path) ? styles['nav-item--active'] : '',
              ]
                .filter(Boolean)
                .join(' ')}
              onClick={() => {
                // Close mobile sidebar on navigation
                if (sidebarOpen) {
                  dispatch(setSidebarOpen(false));
                }
              }}
            >
              <span className={styles['nav-icon']}>{item.icon}</span>
              {!sidebarCollapsed && (
                <span className={styles['nav-label']}>{t(item.label)}</span>
              )}
            </NavLink>
          ))}
        </nav>

        <button
          className={styles['collapse-btn']}
          onClick={() => dispatch(toggleSidebarCollapsed())}
          aria-label={sidebarCollapsed ? t('layout.sidebar.expandSidebar') : t('layout.sidebar.collapseSidebar')}
        >
          {sidebarCollapsed ? '→' : '←'}
        </button>
      </aside>
    </>
  );
}
