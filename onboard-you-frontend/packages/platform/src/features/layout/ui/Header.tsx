import { useState, useRef, useEffect } from 'react';
import { useTranslation } from 'react-i18next';
import { useAppDispatch } from '@/store';
import { toggleSidebar } from '@/features/layout/state/layoutSlice';
import { useGlobal } from '@/shared/hooks/useGlobal';
import { APP_NAME } from '@/shared/domain/constants';
import styles from './Header.module.scss';

export function Header() {
  const { t } = useTranslation();
  const dispatch = useAppDispatch();
  const { user, theme, toggleTheme: onToggleTheme, logout } = useGlobal();
  const [dropdownOpen, setDropdownOpen] = useState(false);
  const dropdownRef = useRef<HTMLDivElement>(null);

  const initials = user?.name
    ? user.name
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

  // Focus first menu item when dropdown opens
  useEffect(() => {
    if (dropdownOpen) {
      const firstItem = dropdownRef.current?.querySelector<HTMLElement>('[role="menuitem"]');
      firstItem?.focus();
    }
  }, [dropdownOpen]);

  return (
    <header className={styles.header}>
      <div className={styles['header-left']}>
        <button
          className={styles['header-hamburger']}
          onClick={() => dispatch(toggleSidebar())}
          aria-label={t('layout.header.toggleNavigation')}
        >
          ☰
        </button>
        <span className={styles['header-brand']}>{APP_NAME}</span>
      </div>

      <div className={styles['header-actions']}>
        <button
          className={styles['theme-toggle']}
          onClick={onToggleTheme}
          aria-label={t('layout.header.toggleTheme')}
          title={t('layout.header.switchToMode', { mode: theme === 'light' ? t('common.dark') : t('common.light') })}
        >
          {theme === 'light' ? '🌙' : '☀️'}
        </button>

        <div className={styles['avatar-wrapper']} ref={dropdownRef}>
          <button
            className={styles.avatar}
            onClick={() => setDropdownOpen((prev) => !prev)}
            aria-label={t('layout.header.userMenu')}
            aria-expanded={dropdownOpen}
            aria-haspopup="true"
          >
            {initials}
          </button>

          {dropdownOpen && (
            <div
              className={styles.dropdown}
              role="menu"
              aria-label={t('layout.header.userMenu')}
              onKeyDown={(e) => {
                if (e.key === 'Escape') {
                  setDropdownOpen(false);
                }
              }}
            >
              <div className={styles['dropdown-header']}>
                <span className={styles['dropdown-name']}>{user?.name}</span>
                <span className={styles['dropdown-email']}>{user?.email}</span>
              </div>
              <hr className={styles['dropdown-divider']} />
              <button className={styles['dropdown-item']} onClick={logout} role="menuitem">
                {t('layout.header.signOut')}
              </button>
            </div>
          )}
        </div>
      </div>
    </header>
  );
}
