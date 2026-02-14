import { useContext, useEffect } from 'react';
import { useSearchParams, useNavigate } from 'react-router-dom';
import { AuthContext } from '@/features/auth/state/AuthContext';
import { Spinner } from '@/shared/ui/Spinner';
import styles from './LoginPage.module.scss';

export function CallbackPage() {
  const authCtx = useContext(AuthContext);
  const [searchParams] = useSearchParams();
  const navigate = useNavigate();

  if (!authCtx) {
    throw new Error('CallbackPage must be used within an AuthProvider');
  }

  const { state, exchangeCode } = authCtx;

  useEffect(() => {
    const code = searchParams.get('code');
    if (code) {
      exchangeCode(code).then(() => {
        navigate('/', { replace: true });
      });
    } else {
      navigate('/login', { replace: true });
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  if (state.error) {
    return (
      <div className={styles['login-page']}>
        <div className={styles['login-card']}>
          <h2 className={styles['login-title']}>Authentication Error</h2>
          <p className={styles['login-subtitle']}>{state.error}</p>
          <a href="/login">Return to login</a>
        </div>
      </div>
    );
  }

  return (
    <div className={styles['login-page']}>
      <div className={styles['login-card']}>
        <Spinner size="lg" />
        <p className={styles['login-subtitle']} style={{ marginTop: '1rem' }}>
          Completing sign in…
        </p>
      </div>
    </div>
  );
}
