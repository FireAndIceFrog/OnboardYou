export const API_BASE_URL = import.meta.env.VITE_API_BASE_URL ?? 'http://localhost:3000';
export const MOCK_MODE = import.meta.env.VITE_MOCK_MODE === 'true';
export const APP_NAME = 'OnboardYou';
export const TOKEN_REFRESH_INTERVAL = 5 * 60 * 1000; // 5 minutes
