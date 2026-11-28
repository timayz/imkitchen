import { afterEach, describe, expect, it, vi } from 'vitest';

const { open, close } = vi.hoisted(() => ({ open: vi.fn(), close: vi.fn() }));
vi.mock('sparkling-navigation', () => ({ open, close }));

import { back, pageParams, push, replace, scheme } from './nav.js';

afterEach(() => {
  vi.unstubAllGlobals();
  open.mockReset();
  close.mockReset();
});

describe('scheme', () => {
  it('names the bundle and carries the page parameters', () => {
    const url = new URL(scheme('main', { tab: 'list' }));
    expect(url.protocol).toBe('hybrid:');
    expect(url.searchParams.get('bundle')).toBe('main.lynx.bundle');
    expect(url.searchParams.get('hide_nav_bar')).toBe('1');
    expect(url.searchParams.get('tab')).toBe('list');
  });

  it('opens the container in the colours of the theme', () => {
    vi.stubGlobal('lynx', { __globalProps: { theme: 'dark' } });
    const url = new URL(scheme('about'));
    expect(url.searchParams.get('container_bg_color')).toBe('#1b140c');
    expect(url.searchParams.get('status_font_mode')).toBe('light');
  });
});

describe('pageParams', () => {
  it('reads what iOS delivers', () => {
    vi.stubGlobal('lynx', { __globalProps: { queryItems: { tab: 'list' } } });
    expect(pageParams()).toEqual({ tab: 'list' });
  });

  it('prefers what the Android host router injects', () => {
    vi.stubGlobal('lynx', {
      __globalProps: { queryItems: { containerInitTime: '1' }, pageQuery: { tab: 'list' } },
    });
    expect(pageParams()).toEqual({ containerInitTime: '1', tab: 'list' });
  });

  it('is empty when the host hands nothing over', () => {
    vi.stubGlobal('lynx', { __globalProps: null });
    expect(pageParams()).toEqual({});
  });
});

describe('navigation', () => {
  it('pushes a container and resolves once it is open', async () => {
    open.mockImplementation((_params, done) => done({ code: 1, msg: 'ok' }));
    await push('about');
    const [params] = open.mock.calls[0]!;
    expect(params.scheme).toContain('bundle=about.lynx.bundle');
    expect(params.options).toBeUndefined();
  });

  it('replaces the current container', async () => {
    open.mockImplementation((_params, done) => done({ code: 1, msg: 'ok' }));
    await replace('main');
    expect(open.mock.calls[0]![0].options).toEqual({
      replace: true,
      replaceType: 'onlyCloseAfterOpenSucceed',
    });
  });

  it('rejects when the host refuses', async () => {
    open.mockImplementation((_params, done) => done({ code: 0, msg: 'no such bundle' }));
    await expect(push('about')).rejects.toThrow('no such bundle');
  });

  it('closes the container to go back', () => {
    back();
    expect(close).toHaveBeenCalledOnce();
  });
});
