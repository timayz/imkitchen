import { afterEach, describe, expect, it, vi } from 'vitest';

import { theme } from './theme.js';

function host(props: Record<string, unknown> | null) {
  vi.stubGlobal('lynx', { __globalProps: props });
}

afterEach(() => vi.unstubAllGlobals());

describe('theme', () => {
  it('follows the host', () => {
    host({ theme: 'dark' });
    expect(theme()).toBe('dark');
    host({ theme: 'light' });
    expect(theme()).toBe('light');
  });

  it('is light when the host says nothing', () => {
    host({});
    expect(theme()).toBe('light');
    host(null);
    expect(theme()).toBe('light');
  });
});
