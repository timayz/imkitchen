import { fireEvent, render } from '@lynx-js/react/testing-library';
import { beforeEach, describe, expect, it, vi } from 'vitest';

const { close } = vi.hoisted(() => ({ close: vi.fn() }));
vi.mock('sparkling-navigation', () => ({ open: vi.fn(), close }));

import { App } from './App.js';

beforeEach(() => close.mockReset());

describe('about', () => {
  it('says what the app runs on', () => {
    lynx.__globalProps = { os: 'ios', osVersion: '26.1', language: 'fr-FR', theme: 'dark' };
    const { container } = render(<App />);
    const values = [...container.querySelectorAll('.muted')].map((el) => el.textContent);
    expect(values).toEqual(['test', 'iOS 26.1', 'fr-FR', 'Sombre']);
  });

  it('renders when the host hands nothing over', () => {
    // @ts-expect-error Sparkling 2.0.1 on iOS, before the host injects them.
    lynx.__globalProps = null;
    const { container } = render(<App />);
    expect(container.querySelector('.h1')!.textContent).toBe('About');
  });

  it('goes back', () => {
    lynx.__globalProps = {};
    render(<App />);
    fireEvent.tap(elementTree.root!.querySelector('#about-back')!);
    expect(close).toHaveBeenCalledOnce();
  });
});
