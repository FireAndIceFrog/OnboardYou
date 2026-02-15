import { authHandlers } from './auth';
import { configHandlers } from './config';
import { settingsHandlers } from './settings';
export const handlers = [...authHandlers, ...configHandlers, ...settingsHandlers];
