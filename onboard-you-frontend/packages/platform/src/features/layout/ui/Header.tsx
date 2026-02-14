import { useContext, useState, useRef, useEffect } from 'react';
import { LayoutContext } from '@/features/layout/state/LayoutContext';
import { AuthContext } from '@/features/auth/state/AuthContext';
import { GlobalContext } from '@/shared/state/GlobalContext';
import { APP_NAME } from '@/shared/domain/constants';
import styles from './Header.module.scss';

export function Header() {
  const layoutCtx = useContext(LayoutContext);
  const authCtx = useContext(AuthContext);
  const globalCtx = useContext(GlobalContext);
  const [dropdownOpen, setDropdownOpen] = useState(false);
  const dropdownRef = useRef<HTMLDivElement>(null);

  if (!layoutCtx || !authCtx || !globalCtx) {
    throw new Error('Header must be used within Layout, Auth, and Global providers');
  }

  const { dispatch: layoutDispatch } = layoutCtx;
  const { state: authState, logout } = authCtx;
  const { state: globalState, dispatch: globalDispatch } = globalCtx;

  const initials = authState.user?.name
    ? authState.user.name
        .split(' ')
        .map((n) => n[0])
        .join('')
        .toUpperCase()
        .slice(0, 2)
    : '??';

  // Close dropdown when clicking outside
  useEffect(() => {
    function handleClickOutside(e: MouseEvent) {
      if (dropdownRef.current && !dropdownRef.current.contains(e.target as Node)) {
        setDropdownOpen(false);
      }
    }
    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, []);

  return (
    <header className={styles.header}>
      <div className={styles['header-left']}>
        <button
          className={styles['header-hamburger']}
          onClick={() => layoutDispatch({ type: 'TOGGLE_SIDEBAR' })}
          aria-label="Toggle navigation"
        >
          ☰
        </button>
        <span className={styles['header-brand']}>{APP_NAME}</span>
      </div>

      <div className={styles['header-actions']}>
        <button
          className={styles['theme-toggle']}
          onClick={() => globalDispatch({ type: 'TOGGLE_THEME' })}
          aria-label="Toggle theme"
          title={`Switch to ${globalState.theme === 'light' ? 'dark' : 'light'} mode`}
        >
          {globalState.theme === 'light' ? '🌙' : '☀️'}
        </button>

        <div className={styles['avatar-wrapper']} ref={dropdownRef}>
          <button
            className={styles.avatar}
            onClick={() => setDropdownOpen((prev) => !prev)}
            aria-label="User menu"
          >
            {initials}
          </button>

          {dropdownOpen && (
            <div className={styles.dropdown}>
              <div className={styles['dropdown-header']}>
                <span className={styles['dropdown-name']}>{authState.user?.name}</span>
                <span className={styles['dropdown-email']}>{authState.user?.email}</span>
              </div>
              <hr className={styles['dropdown-divider']} />
              <button className={styles['dropdown-item']} onClick={logout}>
                Sign out
              </button>
            </div>
          )}
        </div>
      </div>
    </header>
  );
}
