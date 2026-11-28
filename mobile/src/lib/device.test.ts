import { afterEach, describe, expect, it, vi } from 'vitest';

import { hostProps, insets, os } from './device.js';

function host(props: Record<string, unknown> | null) {
  vi.stubGlobal('lynx', { __globalProps: props });
}

afterEach(() => vi.unstubAllGlobals());

describe('hostProps', () => {
  it('is empty when the host hands nothing over', () => {
    host(null);
    expect(hostProps()).toEqual({});
    expect(os()).toBe('android');
    expect(insets()).toEqual({ top: 0, bottom: 0 });
  });
});

describe('insets', () => {
  it('takes the safe area the iOS host reports', () => {
    host({ os: 'ios', topHeight: 59, bottomHeight: 34 });
    expect(os()).toBe('ios');
    expect(insets()).toEqual({ top: 59, bottom: 34 });
  });
});
