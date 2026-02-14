import { authHandlers } from './auth';
import { configHandlers } from './config';
export const handlers = [...authHandlers, ...configHandlers];
