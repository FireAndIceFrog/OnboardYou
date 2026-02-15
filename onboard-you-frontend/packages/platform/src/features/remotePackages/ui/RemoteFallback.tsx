
export function RemoteLoadFallback({ reset }: { reset: () => void }) {
  return (
    <div
      role="alert"
      aria-live="assertive"
      style={{
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
        gap: '1rem',
        padding: '4rem',
        textAlign: 'center',
      }}
    >
      <span style={{ fontSize: '2rem' }} aria-hidden="true">⚠️</span>
      <h2 style={{ margin: 0 }}>Failed to load module</h2>
      <p style={{ margin: 0, color: '#64748B' }}>
        The remote module could not be loaded. Please check your connection and try again.
      </p>
      <button
        onClick={reset}
        type="button"
        style={{
          marginTop: '0.5rem',
          padding: '0.5rem 1.5rem',
          background: '#2563EB',
          color: '#fff',
          border: 'none',
          borderRadius: '0.375rem',
          cursor: 'pointer',
        }}
      >
        Try Again
      </button>
    </div>
  );
}
