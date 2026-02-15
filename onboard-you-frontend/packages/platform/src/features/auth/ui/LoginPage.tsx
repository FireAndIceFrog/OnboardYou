import { useTranslation } from 'react-i18next';
import { useAppDispatch } from '@/store';
import { performLogin } from '@/features/auth/state/authSlice';
import { Button } from '@/shared/ui/Button';
import { MOCK_MODE, APP_NAME } from '@/shared/domain/constants';
import styles from './LoginPage.module.scss';

export function LoginPage() {
  const { t } = useTranslation();
  const dispatch = useAppDispatch();

  const login = () => {
    dispatch(performLogin());
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

        <div className={styles['login-actions']}>
          {MOCK_MODE ? (
            <Button variant="primary" size="lg" onClick={login} className={styles['login-btn']}>
              {t('auth.login.demoButton')}
            </Button>
          ) : (
            <Button variant="primary" size="lg" onClick={login} className={styles['login-btn']}>
              {t('auth.login.ssoButton')}
            </Button>
          )}
        </div>

        <p className={styles['login-footer']}>
          {t('auth.login.footer')}
        </p>
      </div>
    </div>
  );
}
