import { fireEvent, render } from '@lynx-js/react/testing-library';
import { beforeEach, describe, expect, it, vi } from 'vitest';

const { open } = vi.hoisted(() => ({ open: vi.fn() }));
vi.mock('sparkling-navigation', () => ({ open, close: vi.fn() }));

import { App } from './App.js';

function tap(id: string) {
  fireEvent.tap(elementTree.root!.querySelector(`#${id}`)!);
}

beforeEach(() => {
  open.mockReset();
  lynx.__globalProps = { language: 'en-US', theme: 'light' };
});

describe('main', () => {
  it('opens on the recipes', () => {
    const { container } = render(<App />);
    expect(container.querySelector('.h1')!.textContent).toBe('Recipes');
    expect(container.querySelector('.tab__label--active')!.textContent).toBe('Recipes');
  });

  it('opens on the tab the container was asked for', () => {
    lynx.__globalProps = { language: 'en-US', queryItems: { tab: 'groceries' } };
    const { container } = render(<App />);
    expect(container.querySelector('.h1')!.textContent).toBe('Groceries');
  });

  it('moves from one tab to another', () => {
    const { container } = render(<App />);
    tap('tab-list');
    expect(container.querySelector('.h1')!.textContent).toBe('List');
    expect(container.querySelector('.tab__label--active')!.textContent).toBe('List');
  });

  it('speaks French on a French device', () => {
    lynx.__globalProps = { language: 'fr-FR' };
    const { container } = render(<App />);
    expect(container.querySelector('.h1')!.textContent).toBe('Recettes');
  });

  it('is dark when the system is', () => {
    lynx.__globalProps = { language: 'en-US', theme: 'dark' };
    const { container } = render(<App />);
    expect(container.querySelector('.theme--dark')).not.toBeNull();
  });

  it('pushes the about screen from the settings', () => {
    render(<App />);
    tap('tab-settings');
    tap('settings-about');
    expect(open.mock.calls[0]![0].scheme).toContain('bundle=about.lynx.bundle');
  });
});
