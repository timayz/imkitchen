export type RecipeType =
  | 'Appetizer'
  | 'MainCourse'
  | 'Dessert'
  | 'Accompaniment'
  | 'Beverage'
  | 'Condiment'

export type DietaryRestriction = 'Vegetarian' | 'Vegan' | 'GlutenFree' | 'DairyFree' | 'NutFree'

export interface Status {
  status: 'idle' | 'cooking' | 'completed'
  /** Zero-based instruction index while cooking. */
  step: number | null
}

export interface Ingredient {
  name: string
  quantity: number
  unit: 'G' | 'ML' | null
  category: string | null
  /** Already formatted: `250 g`, `1.5 L`, `4`. */
  quantity_label: string
  /** What `POST /groceries/toggle` expects. */
  key: string
}

export interface Instruction {
  description: string
  /** Minutes to wait before the next step, 0 when none. */
  time_next: number
}

export interface Recipe {
  id: string
  owner_id: string
  owner_name: string | null
  recipe_type: RecipeType
  name: string
  slug: string
  origin: string | null
  description: string
  household_size: number
  prep_time: number
  cook_time: number
  total_time: number
  ingredients: Ingredient[]
  instructions: Instruction[]
  dietary_restrictions: DietaryRestriction[]
  accepts_accompaniment: boolean
  advance_prep: string
  is_shared: boolean
  difficulty_score: number
  created_at: number
  /** Relative to the API base URL. */
  thumbnail_url: string | null
  blur_placeholder: string | null
}
