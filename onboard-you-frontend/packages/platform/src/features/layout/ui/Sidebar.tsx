import { useContext } from 'react';
import { NavLink, useLocation } from 'react-router-dom';
import { LayoutContext } from '@/features/layout/state/LayoutContext';
import { NAVIGATION_ITEMS } from '@/features/layout/domain/navigation';
import styles from './Sidebar.module.scss';

export function Sidebar() {
  const layoutCtx = useContext(LayoutContext);
  const location = useLocation();

  if (!layoutCtx) {
    throw new Error('Sidebar must be used within a LayoutProvider');
  }

  const { state, dispatch } = layoutCtx;

  const isActive = (path: string) => {
    if (path === '/') return location.pathname === '/';
    return location.pathname.startsWith(path);
  };

  return (
    <>
      {/* Mobile overlay backdrop */}
      {state.sidebarOpen && (
        <div
          className={styles.overlay}
          onClick={() => dispatch({ type: 'SET_SIDEBAR_OPEN', payload: false })}
          aria-hidden="true"
        />
      )}

      <aside
        className={[
          styles.sidebar,
          state.sidebarCollapsed ? styles['sidebar--collapsed'] : '',
          state.sidebarOpen ? styles['sidebar--open'] : '',
        ]
          .filter(Boolean)
          .join(' ')}
      >
        <nav className={styles.nav}>
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
                if (state.sidebarOpen) {
                  dispatch({ type: 'SET_SIDEBAR_OPEN', payload: false });
                }
              }}
            >
              <span className={styles['nav-icon']}>{item.icon}</span>
              {!state.sidebarCollapsed && (
                <span className={styles['nav-label']}>{item.label}</span>
              )}
            </NavLink>
          ))}
        </nav>

        <button
          className={styles['collapse-btn']}
          onClick={() => dispatch({ type: 'TOGGLE_SIDEBAR_COLLAPSED' })}
          aria-label={state.sidebarCollapsed ? 'Expand sidebar' : 'Collapse sidebar'}
        >
          {state.sidebarCollapsed ? '→' : '←'}
        </button>
      </aside>
    </>
  );
}
