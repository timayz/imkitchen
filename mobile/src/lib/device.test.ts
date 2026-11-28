import { afterEach, describe, expect, it, vi } from 'vitest';

import { bottomInset, hostProps, os } from './device.js';

function host(props: Record<string, unknown> | null) {
  vi.stubGlobal('lynx', { __globalProps: props });
}

afterEach(() => vi.unstubAllGlobals());

describe('hostProps', () => {
  it('is empty when the host hands nothing over', () => {
    host(null);
    expect(hostProps()).toEqual({});
    expect(os()).toBe('android');
    expect(bottomInset()).toBe(0);
  });
});

describe('bottomInset', () => {
  it('takes the safe area the iOS host reports', () => {
    host({ os: 'ios', bottomHeight: 34 });
    expect(os()).toBe('ios');
    expect(bottomInset()).toBe(34);
  });
});
