import { useState } from 'react';
import { Navigate } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { useAppDispatch, useAppSelector } from '@/store';
import { performLogin, selectAuth } from '@/features/auth/state/authSlice';
import { DEMO_EMAIL, DEMO_PASSWORD } from '@/features/auth/domain/constants';
import { Button } from '@/shared/ui/Button';
import { APP_NAME } from '@/shared/domain/constants';
import styles from './LoginPage.module.scss';

export function LoginPage() {
  const { t } = useTranslation();
  const dispatch = useAppDispatch();
  const { isAuthenticated, isLoading, error } = useAppSelector(selectAuth);

  const [email, setEmail] = useState(DEMO_EMAIL);
  const [password, setPassword] = useState(DEMO_PASSWORD);

  if (isAuthenticated) {
    return <Navigate to="/" replace />;
  }

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    dispatch(performLogin({ email, password }));
  };

  return (
    <div className={styles['login-page']}>
      <div className={styles['login-card']}>
        <div className={styles['login-brand']}>
          <span className={styles['login-logo']}>📋</span>
          <h1 className={styles['login-app-name']}>{APP_NAME}</h1>
        </div>

        <h2 className={styles['login-title']}>{t('auth.login.title')}</h2>
        <p className={styles['login-subtitle']}>{t('auth.login.subtitle')}</p>

        <form className={styles['login-form']} onSubmit={handleSubmit}>
          <div className={styles['login-field']}>
            <label htmlFor="login-email" className={styles['login-label']}>
              {t('auth.login.emailLabel')}
            </label>
            <input
              id="login-email"
              className={styles['login-input']}
              type="email"
              autoComplete="email"
              required
              placeholder={t('auth.login.emailPlaceholder')}
              value={email}
              onChange={(e) => setEmail(e.target.value)}
            />
          </div>

          <div className={styles['login-field']}>
            <label htmlFor="login-password" className={styles['login-label']}>
              {t('auth.login.passwordLabel')}
            </label>
            <input
              id="login-password"
              className={styles['login-input']}
              type="password"
              autoComplete="current-password"
              required
              placeholder={t('auth.login.passwordPlaceholder')}
              value={password}
              onChange={(e) => setPassword(e.target.value)}
            />
          </div>

          {error && <p className={styles['login-error']}>{error}</p>}

          <div className={styles['login-actions']}>
            <Button
              type="submit"
              variant="primary"
              size="lg"
              disabled={isLoading}
              className={styles['login-btn']}
            >
              {isLoading ? t('auth.login.signingIn') : t('auth.login.submitButton')}
            </Button>
          </div>
        </form>

        <p className={styles['login-footer']}>
          {t('auth.login.footer')}
        </p>
      </div>
    </div>
  );
}
