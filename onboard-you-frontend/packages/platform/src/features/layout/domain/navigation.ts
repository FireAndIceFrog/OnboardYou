import { BuildingIcon, LinkIcon, GearIcon } from '@/shared/ui';
import type { NavItem } from './types';

export const NAVIGATION_ITEMS: NavItem[] = [
  { id: 'home', label: 'layout.navigation.clientPortfolio', path: '/', icon: BuildingIcon },
  { id: 'configs', label: 'layout.navigation.connectedSystems', path: '/config', icon: LinkIcon },
  { id: 'settings', label: 'layout.navigation.mySettings', path: '/settings', icon: GearIcon },
];
