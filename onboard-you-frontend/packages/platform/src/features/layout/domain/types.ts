export interface LayoutState {
  sidebarOpen: boolean;
  sidebarCollapsed: boolean;
}

export interface NavItem {
  id: string;
  label: string;
  path: string;
  icon: string; // emoji for now
}
