import { NavLink, useLocation } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { Box, Flex, IconButton, Text } from '@chakra-ui/react';
import { useAppSelector, useAppDispatch } from '@/store';
import {
  selectLayout,
  setSidebarOpen,
  toggleSidebarCollapsed,
} from '@/features/layout/state/layoutSlice';
import { NAVIGATION_ITEMS } from '@/features/layout/domain/navigation';
import { ChevronLeftIcon, ChevronRightIcon } from '@/shared/ui';

const HEADER_HEIGHT = '64px';
const SIDEBAR_WIDTH = '260px';
const SIDEBAR_COLLAPSED_WIDTH = '64px';

export function Sidebar() {
  const { t } = useTranslation();
  const dispatch = useAppDispatch();
  const { sidebarOpen, sidebarCollapsed } = useAppSelector(selectLayout);
  const location = useLocation();

  const isActive = (path: string) => {
    if (path === '/') return location.pathname === '/';
    return location.pathname.startsWith(path);
  };

  return (
    <>
      {/* Mobile overlay */}
      {sidebarOpen && (
        <Box
          display={{ base: 'block', lg: 'none' }}
          position="fixed"
          inset="0"
          bg="blackAlpha.400"
          zIndex="overlay"
          backdropFilter="blur(2px)"
          onClick={() => dispatch(setSidebarOpen(false))}
          aria-hidden="true"
        />
      )}

      <Flex
        as="aside"
        direction="column"
        position="fixed"
        top={HEADER_HEIGHT}
        left="0"
        height={`calc(100% - ${HEADER_HEIGHT})`}
        width={sidebarCollapsed ? SIDEBAR_COLLAPSED_WIDTH : SIDEBAR_WIDTH}
        bg="primary.500"
        borderRightWidth="1px"
        borderColor="primary.600"
        zIndex="modal"
        overflowY="auto"
        transition="width 0.25s ease, transform 0.25s ease"
        transform={{
          base: sidebarOpen ? 'translateX(0)' : 'translateX(-100%)',
          lg: 'translateX(0)',
        }}
        shadow={{ base: sidebarOpen ? 'xl' : 'none', lg: 'none' }}
      >
        <Flex as="nav" direction="column" flex="1" gap={1} aria-label="Main navigation">
          {NAVIGATION_ITEMS.map((item) => {
            const active = isActive(item.path);
            const NavIcon = item.icon;
            return (
              <Flex
                key={item.id}
                asChild
              >
                <NavLink
                  to={item.path}
                  onClick={() => {
                    if (sidebarOpen) dispatch(setSidebarOpen(false));
                  }}
                  style={{
                    display: 'flex',
                    alignItems: 'center',
                    padding: '0.75rem',
                    borderRadius: '0.5rem',
                    textDecoration: 'none',
                    fontSize: '0.875rem',
                    fontWeight: active ? 600 : 500,
                    color: active ? '#ffffff' : 'rgba(255,255,255,0.7)',
                    backgroundColor: active ? 'rgba(37,99,235,0.3)' : 'transparent',
                    borderLeft: `3px solid ${active ? '#2563eb' : 'transparent'}`,
                    transition: 'all 0.15s ease',
                  }}
                >
                  <Box w="24px" display="flex" justifyContent="center" flexShrink={0} mr={sidebarCollapsed ? 0 : 3}>
                    <NavIcon size="1.125em" />
                  </Box>
                  {!sidebarCollapsed && (
                    <Text overflow="hidden" textOverflow="ellipsis" whiteSpace="nowrap">
                      {t(item.label)}
                    </Text>
                  )}
                </NavLink>
              </Flex>
            );
          })}
        </Flex>

        <Box display={{ base: 'none', lg: 'block' }} p={3}>
          <IconButton
            variant="outline"
            size="sm"
            w="full"
            onClick={() => dispatch(toggleSidebarCollapsed())}
            color="whiteAlpha.900"
            borderColor="whiteAlpha.800"
            borderWidth="1px"
            _hover={{ bg: 'whiteAlpha.100' }}
            aria-label={
              sidebarCollapsed
                ? t('layout.sidebar.expandSidebar')
                : t('layout.sidebar.collapseSidebar')
            }
          >
            {sidebarCollapsed ? <ChevronRightIcon size="1em" strokeWidth={3} /> : <ChevronLeftIcon size="1em" strokeWidth={3} />}
          </IconButton>
        </Box>
      </Flex>
    </>
  );
}
