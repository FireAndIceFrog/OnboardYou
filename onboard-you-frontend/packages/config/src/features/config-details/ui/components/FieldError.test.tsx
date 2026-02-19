import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { ChakraProvider } from '@chakra-ui/react';
import { system } from '@/theme';
import { FieldError } from './FieldError';

function renderFieldError(id: string, error?: string) {
  return render(
    <ChakraProvider value={system}>
      <FieldError id={id} error={error} />
    </ChakraProvider>,
  );
}

describe('FieldError', () => {
  it('renders nothing when there is no error', () => {
    const { container } = renderFieldError('test-error');
    expect(container.firstChild).toBeNull();
  });

  it('renders nothing when error is undefined', () => {
    const { container } = renderFieldError('test-error', undefined);
    expect(container.firstChild).toBeNull();
  });

  it('renders the error message with alert role', () => {
    renderFieldError('test-error', 'This field is required');
    const alert = screen.getByRole('alert');
    expect(alert).toHaveTextContent('This field is required');
    expect(alert).toHaveAttribute('id', 'test-error');
  });
});
