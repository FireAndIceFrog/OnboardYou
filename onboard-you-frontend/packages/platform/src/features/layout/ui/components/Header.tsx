import { useTranslation } from 'react-i18next';
import { Box, Flex, IconButton, Menu, Portal, Text } from '@chakra-ui/react';
import { useAppDispatch } from '@/store';
import { toggleSidebar } from '@/features/layout/state/layoutSlice';
import { useGlobal } from '@/shared/hooks/useGlobal';
import { APP_NAME } from '@/shared/domain/constants';

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
      bg="bg"
      borderBottomWidth="1px"
      borderColor="border"
      zIndex="modal"
      alignItems="center"
      justifyContent="space-between"
    >
      <Flex alignItems="center" gap={3}>
        <IconButton
          display={{ base: 'flex', lg: 'none' }}
          variant="ghost"
          size="sm"
          onClick={() => dispatch(toggleSidebar())}
          aria-label={t('layout.header.toggleNavigation')}
        >
          ☰
        </IconButton>
        <Text
          fontSize="xl"
          fontWeight="bold"
          letterSpacing="-0.02em"
          bgGradient="to-r"
          gradientFrom="blue.600"
          gradientTo="purple.600"
          bgClip="text"
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
              colorPalette="blue"
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
