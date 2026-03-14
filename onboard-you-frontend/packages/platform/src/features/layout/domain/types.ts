import type { ComponentType, SVGProps } from 'react';

export interface LayoutState {
  sidebarOpen: boolean;
  sidebarCollapsed: boolean;
}

export interface NavItem {
  id: string;
  label: string;
  path: string;
  icon: ComponentType<SVGProps<SVGSVGElement> & { size?: number | string }>;
}
