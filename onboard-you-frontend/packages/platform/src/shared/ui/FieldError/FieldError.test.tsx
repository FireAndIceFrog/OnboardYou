import { describe, it, expect } from 'vitest';
import { screen } from '@testing-library/react';
import { renderWithProviders } from '@/shared/test/testWrapper';
import { FieldError } from './FieldError';

describe('FieldError', () => {
  it('renders nothing when no error is provided', () => {
    const { container } = renderWithProviders(
      <FieldError id="test-error" />,
    );
    expect(container.firstChild).toBeNull();
  });

  it('renders error message when error is provided', () => {
    renderWithProviders(
      <FieldError id="test-error" error="Something went wrong" />,
    );
    expect(screen.getByRole('alert')).toHaveTextContent('Something went wrong');
  });

  it('has the correct id attribute', () => {
    renderWithProviders(
      <FieldError id="my-field-error" error="Bad input" />,
    );
    expect(screen.getByRole('alert')).toHaveAttribute('id', 'my-field-error');
  });
});
