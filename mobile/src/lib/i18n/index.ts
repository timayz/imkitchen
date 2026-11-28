import { hostProps } from '../device.js';
import { en, type MessageKey } from './en.js';
import { fr } from './fr.js';

export type { MessageKey };

export type Locale = 'en' | 'fr';

/** The app's language for the OS one: French for any `fr-*`, else English. */
export function locale(): Locale {
  const lang = (hostProps().language || 'en').toLowerCase();
  return lang.startsWith('fr') ? 'fr' : 'en';
}

const tables: Record<Locale, Record<MessageKey, string>> = { en, fr };

/** The message in the user's language, `%{name}` replaced from `vars`. */
export function t(key: MessageKey, vars: Record<string, string | number> = {}): string {
  let text: string = tables[locale()][key] ?? en[key] ?? key;
  for (const [name, value] of Object.entries(vars)) {
    text = text.split(`%{${name}}`).join(String(value));
  }
  return text;
}
