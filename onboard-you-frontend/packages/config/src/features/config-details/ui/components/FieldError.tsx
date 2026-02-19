import { Text } from '@chakra-ui/react';

interface FieldErrorProps {
  id: string;
  error?: string;
}

export function FieldError({ id, error }: FieldErrorProps) {
  if (!error) return null;

  return (
    <Text id={id} fontSize="xs" color="red.500" mt="1" role="alert">
      {error}
    </Text>
  );
}
