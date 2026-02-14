// ============================================================================
// OnboardYou — OAuth Callback Page
//
// Handles the Cognito redirect with ?code=…, exchanges the code for tokens,
// then navigates the user to the dashboard.
// ============================================================================

import { useEffect, useRef, useState } from 'react';
import { useNavigate, useSearchParams } from 'react-router-dom';
import { useAuth } from './AuthContext';

export function CallbackPage() {
  const { exchangeCode } = useAuth();
  const navigate = useNavigate();
  const [searchParams] = useSearchParams();
  const [error, setError] = useState<string | null>(null);

  // Prevent double-invocation in StrictMode dev renders.
  const exchangedRef = useRef(false);

  useEffect(() => {
    if (exchangedRef.current) return;

    const code = searchParams.get('code');
    if (!code) {
      setError('No authorization code found in the callback URL.');
      return;
    }

    exchangedRef.current = true;

    exchangeCode(code)
      .then(() => navigate('/', { replace: true }))
      .catch((err: unknown) => {
        const message =
          err instanceof Error ? err.message : 'Token exchange failed.';
        setError(message);
      });
  }, [searchParams, exchangeCode, navigate]);

  if (error) {
    return (
      <div
        style={{
          display: 'flex',
          flexDirection: 'column',
          alignItems: 'center',
          justifyContent: 'center',
          minHeight: '100vh',
          gap: '1rem',
          fontFamily: 'Inter, system-ui, sans-serif',
          color: '#0F172A',
          padding: '2rem',
          textAlign: 'center',
        }}
      >
        <p style={{ color: '#EF4444', fontWeight: 600, fontSize: '1.125rem' }}>
          Authentication Error
        </p>
        <p style={{ color: '#475569', fontSize: '0.875rem', maxWidth: 400 }}>
          {error}
        </p>
        <a
          href="/login"
          style={{
            marginTop: '0.5rem',
            color: '#2563EB',
            fontWeight: 500,
            textDecoration: 'none',
          }}
        >
          ← Back to login
        </a>
      </div>
    );
  }

  return (
    <div
      style={{
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
        justifyContent: 'center',
        minHeight: '100vh',
        gap: '1rem',
        fontFamily: 'Inter, system-ui, sans-serif',
        color: '#475569',
      }}
    >
      {/* Spinner */}
      <div
        style={{
          width: 40,
          height: 40,
          border: '3px solid #E2E8F0',
          borderTopColor: '#2563EB',
          borderRadius: '50%',
          animation: 'spin 0.8s linear infinite',
        }}
      />
      <p>Completing sign-in…</p>

      {/* Inline keyframe for the spinner */}
      <style>{`@keyframes spin { to { transform: rotate(360deg); } }`}</style>
    </div>
  );
}
