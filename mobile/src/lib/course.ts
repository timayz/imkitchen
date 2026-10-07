import type { RecipeType } from './api/recipe.js'
import { t } from './i18n/index.js'

/** Per-course presentation, mirroring the web's meal-type tints. */
export interface Course {
  emoji: string
  /** Accent (text on soft, dots, progress). */
  ink: string
  /** Soft tinted background. */
  soft: string
  /** Strong tint (badges, hero blob). */
  tint: string
  label: string
}

const COURSES: Record<RecipeType, Omit<Course, 'label'>> = {
  Appetizer: { emoji: '🥗', ink: '#1e3a5f', soft: '#dde9f5', tint: '#3b6fb0' },
  MainCourse: { emoji: '🍛', ink: '#7a2d05', soft: '#fde3cf', tint: '#ef6c1e' },
  Accompaniment: { emoji: '🥖', ink: '#1b4d2e', soft: '#d8ebdb', tint: '#2f7d4f' },
  Dessert: { emoji: '🍰', ink: '#6b1f49', soft: '#f6dbeb', tint: '#c4428d' },
  Beverage: { emoji: '🥤', ink: '#155e75', soft: '#d4eef5', tint: '#2596be' },
  Condiment: { emoji: '🥫', ink: '#78350f', soft: '#fbeacb', tint: '#c6951d' },
}

export function course(type: RecipeType): Course {
  const base = COURSES[type] ?? COURSES.MainCourse
  return { ...base, label: t(`course.${type}` as const) }
}

/** Grocery aisle presentation, keyed by the API's `shopping_<Category>`. */
export interface AisleStyle {
  emoji: string
  hex: string
  soft: string
  label: string
}

const AISLES: Record<string, Omit<AisleStyle, 'label'>> = {
  shopping_FruitsAndVegetables: { emoji: '🥬', hex: '#2f7d4f', soft: '#d8ebdb' },
  shopping_Butcher: { emoji: '🥩', hex: '#b33b1f', soft: '#fee2e2' },
  shopping_Seafood: { emoji: '🐟', hex: '#2563eb', soft: '#dbeafe' },
  shopping_DairyAndEggs: { emoji: '🥛', hex: '#d97706', soft: '#fef3c7' },
  shopping_Bakery: { emoji: '🥖', hex: '#ef6c1e', soft: '#fde3cf' },
  shopping_Grocery: { emoji: '🥫', hex: '#78716c', soft: '#f5f5f4' },
  shopping_Frozen: { emoji: '❄️', hex: '#0284c7', soft: '#e0f2fe' },
  shopping_Refrigerated: { emoji: '🧊', hex: '#3b6fb0', soft: '#dde9f5' },
  shopping_SnacksAndConfectionery: { emoji: '🍬', hex: '#c4428d', soft: '#f6dbeb' },
}

export function aisle(key: string): AisleStyle {
  const base = AISLES[key] ?? { emoji: '🛒', hex: '#8a7e70', soft: '#f3ead6' }
  return { ...base, label: t(`aisle.${key}` as never) ?? key }
}

export function minutes(total: number): string {
  if (total < 60) return `${total} min`
  const h = Math.floor(total / 60)
  const m = total % 60
  return m === 0 ? `${h} h` : `${h} h ${m}`
}
