/*
 * Guided spotlight tours: which tour opens on which tab, where each step
 * points (element ids; Lynx UI methods only take id selectors), and the
 * measuring helpers the overlay uses. The server owns the catalog of tour
 * ids and step keys (`GET /api/v1/tours`); this file owns the anchors and
 * copy for them. A step whose key the server does not list, or whose anchor
 * is not on screen, is skipped.
 */
import type { TourDto } from './api/tours.js';
import { type MessageKey, t } from './i18n/index.js';

export interface StepDef {
  key: string;
  /** Candidate element ids, first one found wins. */
  targetIds: string[];
  titleKey: MessageKey;
  bodyKey: MessageKey;
}

export interface TourDef {
  /** The tab the tour runs on. */
  page: string;
  /** Element ids that must be on screen for the tour to open at all. */
  requires: string[];
  steps: StepDef[];
}

export const TOURS: Record<string, TourDef> = {
  kitchen: {
    page: 'kitchen',
    requires: [],
    steps: [
      {
        key: 'welcome',
        targetIds: ['kt-header'],
        titleKey: 'tour.kitchen.welcome.title',
        bodyKey: 'tour.kitchen.welcome.body',
      },
      {
        key: 'nav',
        targetIds: ['tab-recipes'],
        titleKey: 'tour.kitchen.nav.title',
        bodyKey: 'tour.kitchen.nav.body',
      },
      {
        key: 'list',
        targetIds: ['kt-hero', 'kt-empty'],
        titleKey: 'tour.kitchen.list.title',
        bodyKey: 'tour.kitchen.list.body',
      },
      {
        key: 'cta',
        targetIds: ['kt-cta', 'kt-regen'],
        titleKey: 'tour.kitchen.cta.title',
        bodyKey: 'tour.kitchen.cta.body',
      },
    ],
  },
  recipes: {
    page: 'recipes',
    requires: [],
    steps: [
      {
        key: 'search',
        targetIds: ['rec-search'],
        titleKey: 'tour.recipes.search.title',
        bodyKey: 'tour.recipes.search.body',
      },
      {
        key: 'new',
        targetIds: ['rec-add'],
        titleKey: 'tour.recipes.new.title',
        bodyKey: 'tour.recipes.new.body',
      },
      {
        key: 'library',
        targetIds: ['rec-list'],
        titleKey: 'tour.recipes.library.title',
        bodyKey: 'tour.recipes.library.body',
      },
    ],
  },
  cooking: {
    page: 'kitchen',
    requires: ['kt-hero'],
    steps: [
      {
        key: 'up_next',
        targetIds: ['kt-hero'],
        titleKey: 'tour.cooking.up_next.title',
        bodyKey: 'tour.cooking.up_next.body',
      },
      {
        key: 'start',
        targetIds: ['kt-start'],
        titleKey: 'tour.cooking.start.title',
        bodyKey: 'tour.cooking.start.body',
      },
      {
        key: 'recipe',
        targetIds: ['kt-see'],
        titleKey: 'tour.cooking.recipe.title',
        bodyKey: 'tour.cooking.recipe.body',
      },
      {
        key: 'regenerate',
        targetIds: ['kt-regen'],
        titleKey: 'tour.cooking.regenerate.title',
        bodyKey: 'tour.cooking.regenerate.body',
      },
    ],
  },
  groceries: {
    page: 'groceries',
    requires: ['groc-aisle-0'],
    steps: [
      {
        key: 'route',
        targetIds: ['groc-jumps'],
        titleKey: 'tour.groceries.route.title',
        bodyKey: 'tour.groceries.route.body',
      },
      {
        key: 'aisle',
        targetIds: ['groc-aisle-0'],
        titleKey: 'tour.groceries.aisle.title',
        bodyKey: 'tour.groceries.aisle.body',
      },
      {
        key: 'check',
        targetIds: ['groc-item-0'],
        titleKey: 'tour.groceries.check.title',
        bodyKey: 'tour.groceries.check.body',
      },
    ],
  },
  settings: {
    page: 'settings',
    requires: [],
    steps: [
      {
        key: 'household',
        targetIds: ['set-household'],
        titleKey: 'tour.settings.household.title',
        bodyKey: 'tour.settings.household.body',
      },
      {
        key: 'courses',
        targetIds: ['set-courses'],
        titleKey: 'tour.settings.courses.title',
        bodyKey: 'tour.settings.courses.body',
      },
      {
        key: 'aisles',
        targetIds: ['set-aisles'],
        titleKey: 'tour.settings.aisles.title',
        bodyKey: 'tour.settings.aisles.body',
      },
    ],
  },
};

export interface Pick {
  id: string;
  def: TourDef;
  dto: TourDto;
  /** Catalog index to resume at. */
  start: number;
}

/**
 * The tour to open on `tab`: the first one, in server order, that belongs to
 * the tab and is pending or in progress. Requirements are checked later,
 * once the tab's content is on screen.
 */
export function pickTour(tours: TourDto[], tab: string): Pick | null {
  for (const dto of tours) {
    const def = TOURS[dto.id];
    if (!def || def.page !== tab) continue;
    if (dto.status !== 'pending' && dto.status !== 'in_progress') continue;
    return { id: dto.id, def, dto, start: dto.status === 'in_progress' ? dto.step : 0 };
  }
  return null;
}

export interface TourStep {
  key: string;
  /** Index in the server's step list (what progress is recorded against). */
  index: number;
  targetIds: string[];
  title: string;
  body: string;
}

export interface ActiveTour {
  id: string;
  steps: TourStep[];
  pos: number;
}

export interface Rect {
  left: number;
  top: number;
  right: number;
  bottom: number;
  width: number;
  height: number;
}

/** The viewport-relative box of the element with `id`, or null when absent. */
export function measure(id: string): Promise<Rect | null> {
  return new Promise((resolve) => {
    lynx
      .createSelectorQuery()
      .select(`#${id}`)
      .invoke({
        method: 'boundingClientRect',
        params: { androidEnableTransformProps: true },
        success: (r: Rect) => resolve(r.width > 0 && r.height > 0 ? r : null),
        fail: () => resolve(null),
      })
      .exec();
  });
}

export function scrollIntoView(id: string): void {
  lynx
    .createSelectorQuery()
    .select(`#${id}`)
    .invoke({
      method: 'scrollIntoView',
      params: { scrollIntoViewOptions: { block: 'center', behavior: 'smooth' } },
    })
    .exec();
}

const wait = (ms: number) => new Promise<void>((resolve) => setTimeout(resolve, ms));

/**
 * The first candidate id that measures, retrying while the tab's content is
 * still mounting. Returns the id, not the box: the overlay measures again
 * right before drawing.
 */
export async function locate(ids: string[], tries = 5): Promise<string | null> {
  for (let attempt = 0; attempt < tries; attempt++) {
    for (const id of ids) if (await measure(id)) return id;
    await wait(250);
  }
  return null;
}

/**
 * Resolves a picked tour against the screen: requirements must be present
 * and only steps with an anchor are kept, so "Step n of m" counts what the
 * user will actually see. Returns null when nothing can be shown.
 */
export async function openTour(pick: Pick): Promise<ActiveTour | null> {
  for (const id of pick.def.requires) if (!(await locate([id]))) return null;

  const steps: TourStep[] = [];
  for (const step of pick.def.steps) {
    const index = pick.dto.steps.indexOf(step.key);
    if (index < 0) continue;
    const found = await locate(step.targetIds, steps.length === 0 ? 5 : 1);
    if (!found) continue;
    steps.push({
      key: step.key,
      index,
      targetIds: step.targetIds,
      title: t(step.titleKey),
      body: t(step.bodyKey),
    });
  }
  if (steps.length === 0) return null;

  let pos = steps.findIndex((s) => s.index >= pick.start);
  if (pos < 0) pos = 0;
  return { id: pick.id, steps, pos };
}
