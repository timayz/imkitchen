import { en, type MessageKey } from './en.js'
import { fr } from './fr.js'

export type { MessageKey }

/** Two-letter language from the OS; the server receives the full tag. */
export function locale(): 'en' | 'fr' {
  const lang = (lynx.__globalProps.language || 'en').toLowerCase()
  return lang.startsWith('fr') ? 'fr' : 'en'
}

const tables = { en, fr }

export function t(key: MessageKey, vars: Record<string, string | number> = {}): string {
  let text: string = tables[locale()][key] ?? en[key] ?? key
  for (const [name, value] of Object.entries(vars)) {
    text = text.split(`%{${name}}`).join(String(value))
  }
  return text
}
