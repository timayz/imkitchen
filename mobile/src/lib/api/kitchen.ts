import { request } from './client.js';
import type { Ingredient, Recipe, RecipeType, Status } from './recipe.js';

export interface Entry {
  id: string;
  name: string;
  slug: string;
  recipe_type: RecipeType;
  status: Status;
  advance_prep: string;
  prep_time: number;
  cook_time: number;
  total_time: number;
  thumbnail_url: string | null;
  blur_placeholder: string | null;
  /**
   * The composed dish this recipe is part of in the list: entries sharing a
   * `dish_id` form one row. Absent in documents cached before dishes were
   * grouped.
   */
  dish_id?: string | null;
}

/** A composed dish in the list: the name of the row its entries form. */
export interface DishGroup {
  id: string;
  name: string;
}

export interface StepText {
  index: number;
  text: string;
}

export interface CurrentStep {
  index: number;
  description: string;
  time_next: number;
}

export interface Steps {
  completed: StepText[];
  coming: StepText[];
  current: CurrentStep | null;
}

export interface KitchenList {
  kind: 'list';
  entries: Entry[];
  /** Names of the composed dishes in the list; absent in older cached documents. */
  dishes?: DishGroup[];
  /** Always set by the server; `null` only in a list rewritten offline. */
  focused: Recipe | null;
  focused_status: Status;
  completed_count: number;
  total_count: number;
  prep_ahead: Entry[];
  steps: Steps;
  cook_external: boolean;
  external_url: string | null;
}

export type Overview =
  /** Nothing to cook; `has_recipes` says whether generating is possible. */
  { kind: 'empty'; has_recipes: boolean } | KitchenList;

export interface Dish {
  entries: Entry[];
  recipe: Recipe;
  status: Status;
  steps: Steps;
  cook_external: boolean;
  external_url: string | null;
}

export interface Aisle {
  /** `shopping_<Category>` */
  key: string;
  items: Ingredient[];
}

export interface CookingScreen {
  recipe: Recipe;
  status: Status;
  steps: Steps;
  origin_embeddable: boolean;
  show_ingredients: boolean;
  ingredient_aisles: Aisle[];
  external_url: string | null;
}

export function getKitchen(): Promise<Overview> {
  return request<Overview>('/api/v1/kitchen');
}

export function generate(count: number, household_size: number): Promise<Overview> {
  return request<Overview>('/api/v1/kitchen/generate', {
    method: 'POST',
    body: { count, household_size },
  });
}

export function removeFromList(id: string): Promise<void> {
  return request<void>(`/api/v1/kitchen/recipes/${encodeURIComponent(id)}`, { method: 'DELETE' });
}

/** Takes every recipe of a composed dish off the list. */
export function removeDishFromList(id: string): Promise<void> {
  return request<void>(`/api/v1/kitchen/dishes/${encodeURIComponent(id)}`, { method: 'DELETE' });
}

export function getDish(id: string): Promise<Dish> {
  return request<Dish>(`/api/v1/kitchen/recipes/${encodeURIComponent(id)}`);
}

export function getCooking(id: string): Promise<CookingScreen> {
  return request<CookingScreen>(`/api/v1/kitchen/recipes/${encodeURIComponent(id)}/cook`);
}

/** Absolute cooking cursor (idempotent): what the offline queue replays. */
export function setStatus(id: string, status: Status): Promise<CookingScreen> {
  return request<CookingScreen>(`/api/v1/kitchen/recipes/${encodeURIComponent(id)}/status`, {
    method: 'PUT',
    body: status,
  });
}
