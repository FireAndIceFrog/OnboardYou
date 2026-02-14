// ============================================================================
// OnboardYou — AppLayout
//
// Main application shell: fixed header, collapsible sidebar, main content
// area that renders child routes via <Outlet />.
// ============================================================================

import { useCallback, useRef, useEffect } from 'react';
import { Outlet } from 'react-router-dom';
import { useAuth } from '@/auth/AuthContext';
import { useGlobalStore } from '@/hooks/useGlobal';
import { MenuBar } from './MenuBar';
import styles from './AppLayout.module.scss';

export function AppLayout() {
  const { user, logout } = useAuth();
  const { sidebarOpen, toggleSidebar, setSidebarOpen } = useGlobalStore();
  const overlayRef = useRef<HTMLDivElement>(null);

  // Close sidebar on Escape key
  useEffect(() => {
    function handleKey(e: KeyboardEvent) {
      if (e.key === 'Escape' && sidebarOpen) {
        setSidebarOpen(false);
      }
    }
    document.addEventListener('keydown', handleKey);
    return () => document.removeEventListener('keydown', handleKey);
  }, [sidebarOpen, setSidebarOpen]);

  const handleOverlayClick = useCallback(() => {
    setSidebarOpen(false);
  }, [setSidebarOpen]);

  const initials = user?.name
    ? user.name
        .split(' ')
        .map((n) => n[0])
        .join('')
        .toUpperCase()
        .slice(0, 2)
    : '?';

  return (
    <div className={styles.layout}>
      {/* ---- Header ---- */}
      <header className={styles.header}>
        <div className={styles.headerLeft}>
          <button
            type="button"
            className={styles.hamburger}
            onClick={toggleSidebar}
            aria-label={sidebarOpen ? 'Collapse sidebar' : 'Expand sidebar'}
          >
            <svg
              width="20"
              height="20"
              viewBox="0 0 20 20"
              fill="none"
              xmlns="http://www.w3.org/2000/svg"
            >
              <path
                d="M3 5h14M3 10h14M3 15h14"
                stroke="currentColor"
                strokeWidth="1.5"
                strokeLinecap="round"
              />
            </svg>
          </button>

          <span className={styles.brand}>OnboardYou</span>
        </div>

        <div className={styles.headerRight}>
          <div className={styles.userMenu}>
            <button type="button" className={styles.avatar} title={user?.name ?? 'User'}>
              {initials}
            </button>

            <div className={styles.dropdown}>
              <div className={styles.dropdownHeader}>
                <p className={styles.dropdownName}>{user?.name}</p>
                <p className={styles.dropdownEmail}>{user?.email}</p>
              </div>
              <hr className={styles.dropdownDivider} />
              <button
                type="button"
                className={styles.dropdownItem}
                onClick={logout}
              >
                Sign out
              </button>
            </div>
          </div>
        </div>
      </header>

      {/* ---- Sidebar ---- */}
      <aside
        className={`${styles.sidebar} ${sidebarOpen ? '' : styles['sidebar--collapsed']}`}
      >
        <MenuBar collapsed={!sidebarOpen} />
      </aside>

      {/* ---- Mobile overlay ---- */}
      {sidebarOpen && (
        <div
          ref={overlayRef}
          className={styles.overlay}
          onClick={handleOverlayClick}
          aria-hidden="true"
        />
      )}

      {/* ---- Main content ---- */}
      <main className={`${styles.main} ${sidebarOpen ? '' : styles['main--expanded']}`}>
        <Outlet />
      </main>
    </div>
  );
}
