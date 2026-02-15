import type { NavItem } from './types';

export const NAVIGATION_ITEMS: NavItem[] = [
  { id: 'home', label: 'layout.navigation.clientPortfolio', path: '/', icon: '🏢' },
  { id: 'configs', label: 'layout.navigation.connectedSystems', path: '/config', icon: '🔗' },
  { id: 'settings', label: 'layout.navigation.mySettings', path: '/settings', icon: '⚙️' },
];
