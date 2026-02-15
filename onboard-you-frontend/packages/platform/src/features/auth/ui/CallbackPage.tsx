import { useEffect } from 'react';
import { useSearchParams, useNavigate } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { useAppSelector, useAppDispatch } from '@/store';
import { exchangeCode, selectAuth } from '@/features/auth/state/authSlice';
import { Spinner } from '@/shared/ui/Spinner';
import styles from './LoginPage.module.scss';

export function CallbackPage() {
  const { t } = useTranslation();
  const dispatch = useAppDispatch();
  const state = useAppSelector(selectAuth);
  const [searchParams] = useSearchParams();
  const navigate = useNavigate();

  useEffect(() => {
    const code = searchParams.get('code');
    if (code) {
      dispatch(exchangeCode(code)).then(() => {
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
          <h2 className={styles['login-title']}>{t('auth.callback.errorTitle')}</h2>
          <p className={styles['login-subtitle']}>{state.error}</p>
          <a href="/login">{t('auth.callback.returnToLogin')}</a>
        </div>
      </div>
    );
  }

  return (
    <div className={styles['login-page']}>
      <div className={styles['login-card']}>
        <Spinner size="lg" />
        <p className={styles['login-subtitle']} style={{ marginTop: '1rem' }}>
          {t('auth.callback.completingSignIn')}
        </p>
      </div>
    </div>
  );
}
