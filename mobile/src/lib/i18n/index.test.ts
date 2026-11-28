import { afterEach, describe, expect, it, vi } from 'vitest';

import { en } from './en.js';
import { fr } from './fr.js';
import { locale, t } from './index.js';

function host(props: Record<string, unknown> | null) {
  vi.stubGlobal('lynx', { __globalProps: props });
}

afterEach(() => vi.unstubAllGlobals());

describe('locale', () => {
  it('is French for any French tag', () => {
    host({ language: 'fr-CA' });
    expect(locale()).toBe('fr');
  });

  it('is English for any other language', () => {
    host({ language: 'de-DE' });
    expect(locale()).toBe('en');
  });

  it('is English when the host says nothing', () => {
    host(null);
    expect(locale()).toBe('en');
  });
});

describe('t', () => {
  it('answers in the language of the device', () => {
    host({ language: 'fr-FR' });
    expect(t('tabs.groceries')).toBe('Courses');
    host({ language: 'en-US' });
    expect(t('tabs.groceries')).toBe('Groceries');
  });

  it('replaces the variables of a message', () => {
    host({ language: 'en' });
    vi.spyOn(en, 'about.version', 'get').mockReturnValue('Version %{n} of %{n}');
    expect(t('about.version', { n: 2 })).toBe('Version 2 of 2');
  });
});

describe('translations', () => {
  it('has every message in French', () => {
    expect(Object.keys(fr).sort()).toEqual(Object.keys(en).sort());
    for (const text of Object.values(fr)) expect(text).not.toBe('');
  });
});
