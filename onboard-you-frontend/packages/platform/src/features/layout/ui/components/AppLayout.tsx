import { Outlet } from 'react-router-dom';
import { Box } from '@chakra-ui/react';
import { useAppSelector } from '@/store';
import { selectSidebarCollapsed } from '@/features/layout/state/layoutSlice';
import { Header } from './Header';
import { Sidebar } from './Sidebar';

const HEADER_HEIGHT = '64px';
const SIDEBAR_WIDTH = '260px';
const SIDEBAR_COLLAPSED_WIDTH = '64px';

export function AppLayout() {
  const sidebarCollapsed = useAppSelector(selectSidebarCollapsed);

  return (
    <Box minH="100%" display="flex" flexDirection="column" height="100%">
      <Header />
      <Sidebar />
      <Box
        as="main"
        mt={HEADER_HEIGHT}
        ml={{ base: '0', lg: sidebarCollapsed ? SIDEBAR_COLLAPSED_WIDTH : SIDEBAR_WIDTH }}
        p={6}
        flex="1"
        minH="0"
        transition="margin-left 0.25s ease"
      >
        <Outlet />
      </Box>
    </Box>
  );
}
