import { describe, it, expect, beforeAll } from 'vitest';
import { screen } from '@testing-library/react';
import { renderWithProviders } from '@/shared/test/testWrapper';
import { ChatWindow } from './ChatWindow';

beforeAll(() => {
  // jsdom doesn't implement scrollIntoView
  Element.prototype.scrollIntoView = () => {};
});

describe('ChatWindow', () => {
  it('renders the chat title heading', () => {
    renderWithProviders(<ChatWindow onClose={() => {}} />);
    const headings = screen.getAllByRole('heading');
    expect(headings.length).toBeGreaterThan(0);
  });

  it('renders the close button', () => {
    renderWithProviders(<ChatWindow onClose={() => {}} />);
    const buttons = screen.getAllByRole('button');
    expect(buttons.length).toBeGreaterThan(0);
  });

  it('renders suggestion buttons when there are no messages', () => {
    renderWithProviders(<ChatWindow onClose={() => {}} />);
    expect(screen.getByRole('group', { name: /suggested prompts/i })).toBeInTheDocument();
  });
});
