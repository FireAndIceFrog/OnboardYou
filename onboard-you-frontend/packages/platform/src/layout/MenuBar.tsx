// ============================================================================
// OnboardYou — MenuBar (sidebar navigation)
//
// Renders navigation items with icons. Highlights the active route.
// In collapsed mode only the icons are visible.
// ============================================================================

import { NavLink, useLocation } from 'react-router-dom';
import styles from './MenuBar.module.scss';

// ---------------------------------------------------------------------------
// Nav item definitions
// ---------------------------------------------------------------------------

interface NavItem {
  label: string;
  path: string;
  /** Inline SVG icon (20×20). */
  icon: React.ReactNode;
}

const NAV_ITEMS: NavItem[] = [
  {
    label: 'Home',
    path: '/',
    icon: (
      <svg width="20" height="20" viewBox="0 0 20 20" fill="none">
        <path
          d="M3 10.5L10 4l7 6.5"
          stroke="currentColor"
          strokeWidth="1.5"
          strokeLinecap="round"
          strokeLinejoin="round"
        />
        <path
          d="M5 9v7a1 1 0 001 1h3v-4h2v4h3a1 1 0 001-1V9"
          stroke="currentColor"
          strokeWidth="1.5"
          strokeLinecap="round"
          strokeLinejoin="round"
        />
      </svg>
    ),
  },
  {
    label: 'Configurations',
    path: '/configs',
    icon: (
      <svg width="20" height="20" viewBox="0 0 20 20" fill="none">
        <path
          d="M8.325 2.317a1.5 1.5 0 013.35 0l.155.77a1.5 1.5 0 002.1.82l.682-.39a1.5 1.5 0 011.675 2.49l-.527.63a1.5 1.5 0 000 1.726l.527.63a1.5 1.5 0 01-1.675 2.49l-.682-.39a1.5 1.5 0 00-2.1.82l-.155.77a1.5 1.5 0 01-3.35 0l-.155-.77a1.5 1.5 0 00-2.1-.82l-.682.39a1.5 1.5 0 01-1.675-2.49l.527-.63a1.5 1.5 0 000-1.726l-.527-.63a1.5 1.5 0 011.675-2.49l.682.39a1.5 1.5 0 002.1-.82l.155-.77z"
          stroke="currentColor"
          strokeWidth="1.5"
        />
        <circle cx="10" cy="10" r="2.5" stroke="currentColor" strokeWidth="1.5" />
      </svg>
    ),
  },
  {
    label: 'Settings',
    path: '/settings',
    icon: (
      <svg width="20" height="20" viewBox="0 0 20 20" fill="none">
        <path
          d="M4 7h12M4 13h8"
          stroke="currentColor"
          strokeWidth="1.5"
          strokeLinecap="round"
        />
        <circle cx="8" cy="7" r="2" stroke="currentColor" strokeWidth="1.5" />
        <circle cx="14" cy="13" r="2" stroke="currentColor" strokeWidth="1.5" />
      </svg>
    ),
  },
];

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

interface MenuBarProps {
  collapsed: boolean;
}

export function MenuBar({ collapsed }: MenuBarProps) {
  const location = useLocation();

  /** Match the root path exactly, and sub-paths by prefix. */
  function isActive(path: string): boolean {
    if (path === '/') return location.pathname === '/';
    return location.pathname.startsWith(path);
  }

  return (
    <nav className={styles.nav} aria-label="Main navigation">
      <ul className={styles.list}>
        {NAV_ITEMS.map((item) => (
          <li key={item.path}>
            <NavLink
              to={item.path}
              end={item.path === '/'}
              className={`${styles.navItem} ${isActive(item.path) ? styles['navItem--active'] : ''} ${collapsed ? styles['navItem--collapsed'] : ''}`}
              title={collapsed ? item.label : undefined}
            >
              <span className={styles.navIcon}>{item.icon}</span>
              {!collapsed && <span className={styles.navLabel}>{item.label}</span>}
            </NavLink>
          </li>
        ))}
      </ul>
    </nav>
  );
}
