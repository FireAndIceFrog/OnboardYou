import { useState } from 'react';
import { Navigate } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import {
  Button,
  Card,
  Center,
  Field,
  Heading,
  Input,
  Text,
  VStack,
} from '@chakra-ui/react';
import { useAppDispatch, useAppSelector } from '@/store';
import { performLogin, selectAuth } from '@/features/auth/state/authSlice';
import { APP_NAME } from '@/shared/domain/constants';

export function LoginPage() {
  const { t } = useTranslation();
  const dispatch = useAppDispatch();
  const { isAuthenticated, isLoading, error } = useAppSelector(selectAuth);

  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');

  if (isAuthenticated) {
    return <Navigate to="/" replace />;
  }

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    dispatch(performLogin({ email, password }));
  };

  return (
    <Center minH="100%" bg="bg.subtle" p={4}>
      <Card.Root maxW="420px" w="full" shadow="lg">
        <Card.Body p={{ base: 8, md: 12 }}>
          <VStack gap={2} mb={7} textAlign="center">
            <Text fontSize="4xl">📋</Text>
            <Heading
              size="xl"
              fontWeight="bold"
              letterSpacing="-0.02em"
              bgGradient="to-r"
              gradientFrom="blue.600"
              gradientTo="purple.600"
              bgClip="text"
            >
              {APP_NAME}
            </Heading>
          </VStack>

          <Heading as="h2" size="lg" fontWeight="semibold" textAlign="center" mb={1}>
            {t('auth.login.title')}
          </Heading>
          <Text fontSize="sm" color="fg.muted" textAlign="center" mb={7}>
            {t('auth.login.subtitle')}
          </Text>

          <form onSubmit={handleSubmit}>
            <VStack gap={4} mb={6}>
              <Field.Root>
                <Field.Label fontWeight="semibold">
                  {t('auth.login.emailLabel')}
                </Field.Label>
                <Input
                  type="email"
                  autoComplete="email"
                  required
                  placeholder={t('auth.login.emailPlaceholder')}
                  value={email}
                  onChange={(e) => setEmail(e.target.value)}
                />
              </Field.Root>

              <Field.Root>
                <Field.Label fontWeight="semibold">
                  {t('auth.login.passwordLabel')}
                </Field.Label>
                <Input
                  type="password"
                  autoComplete="current-password"
                  required
                  placeholder={t('auth.login.passwordPlaceholder')}
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                />
              </Field.Root>

              {error && (
                <Text fontSize="sm" color="fg.error" textAlign="center">
                  {error}
                </Text>
              )}

              <Button
                type="submit"
                colorPalette="blue"
                size="lg"
                w="full"
                loading={isLoading}
                loadingText={t('auth.login.signingIn')}
              >
                {t('auth.login.submitButton')}
              </Button>
            </VStack>
          </form>

          <Text fontSize="xs" color="fg.muted" textAlign="center">
            {t('auth.login.footer')}
          </Text>
        </Card.Body>
      </Card.Root>
    </Center>
  );
}
