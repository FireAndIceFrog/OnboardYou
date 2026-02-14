/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly VITE_COGNITO_DOMAIN: string;
  readonly VITE_COGNITO_CLIENT_ID: string;
  readonly VITE_REDIRECT_URI: string;
  readonly VITE_API_BASE_URL: string;
  readonly VITE_MOCK_MODE: string;
  readonly VITE_CONFIG_REMOTE_URL: string;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}
