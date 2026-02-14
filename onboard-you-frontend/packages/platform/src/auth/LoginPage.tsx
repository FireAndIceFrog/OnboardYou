// ============================================================================
// OnboardYou — Login Page
// ============================================================================

import { useAuth } from './AuthContext';
import styles from './LoginPage.module.scss';

export function LoginPage() {
  const { login } = useAuth();

  return (
    <div className={styles.page}>
      <div className={styles.card}>
        {/* Brand */}
        <div className={styles.brand}>
          <div className={styles.logo}>
            <svg
              width="40"
              height="40"
              viewBox="0 0 40 40"
              fill="none"
              xmlns="http://www.w3.org/2000/svg"
              aria-hidden="true"
            >
              <rect width="40" height="40" rx="10" fill="#2563EB" />
              <path
                d="M12 20C12 15.5817 15.5817 12 20 12C24.4183 12 28 15.5817 28 20C28 24.4183 24.4183 28 20 28"
                stroke="white"
                strokeWidth="2.5"
                strokeLinecap="round"
              />
              <path
                d="M20 16V22L24 24"
                stroke="white"
                strokeWidth="2.5"
                strokeLinecap="round"
                strokeLinejoin="round"
              />
            </svg>
          </div>
          <h1 className={styles.title}>OnboardYou</h1>
        </div>

        <p className={styles.subtitle}>Sign in to your account</p>

        {/* SSO Button */}
        <button type="button" className={styles.ssoButton} onClick={login}>
          <svg
            width="20"
            height="20"
            viewBox="0 0 20 20"
            fill="none"
            xmlns="http://www.w3.org/2000/svg"
            aria-hidden="true"
          >
            <path
              d="M10 2a3 3 0 100 6 3 3 0 000-6zM6 9a4 4 0 00-4 4v1a2 2 0 002 2h12a2 2 0 002-2v-1a4 4 0 00-4-4H6z"
              fill="currentColor"
            />
          </svg>
          Sign in with SSO
        </button>

        {/* Footer */}
        <p className={styles.footer}>
          Secure single sign-on powered by your organization's identity provider.
        </p>
      </div>
    </div>
  );
}
