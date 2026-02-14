// ============================================================================
// OnboardYou — Configs Placeholder
//
// Lazy-loaded placeholder for the Configuration microfrontend.
// This will be replaced when the configs package is integrated.
// ============================================================================

export default function ConfigsPlaceholder() {
  return (
    <div style={{ padding: '2rem', maxWidth: 800 }}>
      <h1
        style={{
          fontSize: '1.5rem',
          fontWeight: 700,
          color: '#0F172A',
          marginBottom: '0.5rem',
        }}
      >
        Configurations
      </h1>
      <p style={{ color: '#475569', fontSize: '0.875rem', marginBottom: '1.5rem' }}>
        Manage your onboarding configurations, field mappings, and workflow templates.
      </p>
      <div
        style={{
          padding: '2.5rem',
          borderRadius: '0.75rem',
          border: '2px dashed #CBD5E1',
          textAlign: 'center',
          color: '#94A3B8',
          fontSize: '0.875rem',
        }}
      >
        The Configuration microfrontend will mount here.
      </div>
    </div>
  );
}
