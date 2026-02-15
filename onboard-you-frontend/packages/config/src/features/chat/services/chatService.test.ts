import { describe, it, expect } from 'vitest';
import { deriveFlowAction } from './chatService';

describe('deriveFlowAction', () => {
  it('returns a cellphone_sanitizer action for phone keyword', () => {
    const result = deriveFlowAction('clean up phone numbers');
    expect(result).not.toBeNull();
    expect(result!.actionType).toBe('cellphone_sanitizer');
  });

  it('returns a regex_replace action for address keyword', () => {
    const result = deriveFlowAction('clean up the address data');
    expect(result).not.toBeNull();
    expect(result!.actionType).toBe('regex_replace');
  });

  it('returns an identity_deduplicator action for duplicate keyword', () => {
    const result = deriveFlowAction('remove duplicate records');
    expect(result).not.toBeNull();
    expect(result!.actionType).toBe('identity_deduplicator');
  });

  it('returns a pii_masking action for sensitive/mask keyword', () => {
    const result = deriveFlowAction('mask sensitive data');
    expect(result).not.toBeNull();
    expect(result!.actionType).toBe('pii_masking');
  });

  it('returns an iso_country_sanitizer action for country keyword', () => {
    const result = deriveFlowAction('standardise country codes');
    expect(result).not.toBeNull();
    expect(result!.actionType).toBe('iso_country_sanitizer');
  });

  it('returns a handle_diacritics action for accent/diacritics keyword', () => {
    const result = deriveFlowAction('fix diacritics in names');
    expect(result).not.toBeNull();
    expect(result!.actionType).toBe('handle_diacritics');
  });

  it('returns null for unrecognised text', () => {
    const result = deriveFlowAction('hello, how are you?');
    expect(result).toBeNull();
  });

  it('returns null for empty string', () => {
    const result = deriveFlowAction('');
    expect(result).toBeNull();
  });
});
