import { useTranslation } from 'react-i18next';
import { Box, Flex, IconButton, Menu, Portal, Text } from '@chakra-ui/react';
import { useAppDispatch } from '@/store';
import { toggleSidebar } from '@/features/layout/state/layoutSlice';
import { useGlobal } from '@/shared/hooks/useGlobal';
import { APP_NAME } from '@/shared/domain/constants';
import { MenuIcon } from '@/shared/ui';

const HEADER_HEIGHT = '64px';

export function Header() {
  const { t } = useTranslation();
  const dispatch = useAppDispatch();
  const { user, logout } = useGlobal();

  const initials = user?.name
    ? user.name
        .split(' ')
        .map((n) => n[0])
        .join('')
        .toUpperCase()
        .slice(0, 2)
    : '??';

  return (
    <Flex
      as="header"
      position="fixed"
      top="0"
      left="0"
      right="0"
      height={HEADER_HEIGHT}
      px={5}
      bg="primary.500"
      borderBottomWidth="1px"
      borderColor="primary.600"
      zIndex="modal"
      alignItems="center"
      justifyContent="space-between"
    >
      <Flex alignItems="center" gap={3}>
        <IconButton
          display={{ base: 'flex', lg: 'none' }}
          variant="ghost"
          size="sm"
          color="white"
          _hover={{ bg: 'whiteAlpha.200' }}
          onClick={() => dispatch(toggleSidebar())}
          aria-label={t('layout.header.toggleNavigation')}
        >
          <MenuIcon size="1.25em" />
        </IconButton>
        <Text
          fontSize="xl"
          fontWeight="bold"
          letterSpacing="-0.02em"
          color="white"
        >
          {APP_NAME}
        </Text>
      </Flex>

      <Flex alignItems="center" gap={3}>
        <Menu.Root positioning={{ placement: 'bottom-end' }}>
          <Menu.Trigger asChild>
            <IconButton
              rounded="full"
              size="sm"
              bg="secondary.500"
              color="white"
              _hover={{ bg: 'secondary.600' }}
              aria-label={t('layout.header.userMenu')}
            >
              {initials}
            </IconButton>
          </Menu.Trigger>
          <Portal>
            <Menu.Positioner>
              <Menu.Content minW="220px">
                <Box px={4} py={3}>
                  <Text fontWeight="semibold" fontSize="sm">
                    {user?.name}
                  </Text>
                  <Text fontSize="xs" color="fg.muted" mt={1}>
                    {user?.email}
                  </Text>
                </Box>
                <Menu.Separator />
                <Menu.Item value="logout" onClick={logout}>
                  {t('layout.header.signOut')}
                </Menu.Item>
              </Menu.Content>
            </Menu.Positioner>
          </Portal>
        </Menu.Root>
      </Flex>
    </Flex>
  );
}
