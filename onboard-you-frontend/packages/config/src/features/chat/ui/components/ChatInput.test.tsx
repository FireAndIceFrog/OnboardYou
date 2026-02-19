import { describe, it, expect, vi } from 'vitest';
import { screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { renderWithProviders } from '@/shared/test/testWrapper';
import { ChatInput } from './ChatInput';

describe('ChatInput', () => {
  it('renders a textarea and send button', () => {
    renderWithProviders(<ChatInput onSend={() => {}} />);
    expect(screen.getByRole('textbox')).toBeInTheDocument();
    expect(screen.getByRole('button')).toBeInTheDocument();
  });

  it('calls onSend when the user types and clicks send', async () => {
    const onSend = vi.fn();
    const user = userEvent.setup();
    renderWithProviders(<ChatInput onSend={onSend} />);

    const textarea = screen.getByRole('textbox');
    await user.type(textarea, 'Hello there');
    await user.click(screen.getByRole('button'));

    expect(onSend).toHaveBeenCalledWith('Hello there');
  });

  it('does not call onSend when disabled', async () => {
    const onSend = vi.fn();
    const user = userEvent.setup();
    renderWithProviders(<ChatInput onSend={onSend} disabled />);

    const textarea = screen.getByRole('textbox');
    await user.type(textarea, 'Test{Enter}');

    expect(onSend).not.toHaveBeenCalled();
  });
});
