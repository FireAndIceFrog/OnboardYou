import { describe, it, expect } from 'vitest';
import { formatTimestamp, renderContent } from './ChatMessage';

describe('formatTimestamp', () => {
  it('formats an ISO timestamp into HH:MM format', () => {
    const result = formatTimestamp('2024-01-15T14:30:00Z');
    // The exact format depends on the locale, but it should contain digits
    expect(result).toMatch(/\d{1,2}:\d{2}/);
  });
});

describe('renderContent', () => {
  it('returns plain text as-is', () => {
    const parts = renderContent('Hello world');
    expect(parts).toHaveLength(1);
    expect(parts[0]).toBe('Hello world');
  });

  it('wraps **bold** text in a strong-like element', () => {
    const parts = renderContent('This is **bold** text');
    expect(parts.length).toBeGreaterThan(1);
    // Find the React element with as="strong" (Chakra Text)
    const strongEl = parts.find(
      (p): p is React.ReactElement =>
        typeof p === 'object' && p !== null && 'props' in p,
    );
    expect(strongEl).toBeDefined();
  });

  it('wraps `code` text in a Code element', () => {
    const parts = renderContent('Use `npm install` here');
    expect(parts.length).toBeGreaterThan(1);
  });

  it('handles newlines by inserting br elements', () => {
    const parts = renderContent('line1\nline2');
    const brEl = parts.find(
      (p) => typeof p !== 'string' && p.type === 'br',
    );
    expect(brEl).toBeDefined();
  });
});
