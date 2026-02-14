import { useContext } from 'react';
import { AuthContext } from '@/features/auth/state/AuthContext';
import { Button } from '@/shared/ui/Button';
import { MOCK_MODE, APP_NAME } from '@/shared/domain/constants';
import styles from './LoginPage.module.scss';

export function LoginPage() {
  const authCtx = useContext(AuthContext);

  if (!authCtx) {
    throw new Error('LoginPage must be used within an AuthProvider');
  }

  const { login } = authCtx;

  return (
    <div className={styles['login-page']}>
      <div className={styles['login-card']}>
        <div className={styles['login-brand']}>
          <span className={styles['login-logo']}>📋</span>
          <h1 className={styles['login-app-name']}>{APP_NAME}</h1>
        </div>

        <h2 className={styles['login-title']}>Welcome back</h2>
        <p className={styles['login-subtitle']}>Sign in to your workspace</p>

        <div className={styles['login-actions']}>
          {MOCK_MODE ? (
            <Button variant="primary" size="lg" onClick={login} className={styles['login-btn']}>
              🧪 Demo Mode — Continue as Demo User
            </Button>
          ) : (
            <Button variant="primary" size="lg" onClick={login} className={styles['login-btn']}>
              Sign in with SSO
            </Button>
          )}
        </div>

        <p className={styles['login-footer']}>
          Secure authentication powered by AWS Cognito
        </p>
      </div>
    </div>
  );
}
