import en from './en.json';
import es from './es.json';
import fr from './fr.json';
import de from './de.json';

export const messages = {
  en,
  es,
  fr,
  de,
} as const;

export type Locale = keyof typeof messages;
