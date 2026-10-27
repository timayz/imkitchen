import { expect, type APIRequestContext, type BrowserContext } from "@playwright/test";

/**
 * Thin client over the JSON API (`/api/v1`), used to seed state that a test
 * is not about (accounts, recipes, lists) so the browser part stays short.
 *
 * Every call goes through the browser context's request object: the session
 * is bound to the User-Agent that created it, and the token is then installed
 * as the web session cookie (`auth_token`), which the web and the API share.
 */

/** Passwords are capped at 20 characters by the identity module. */
export const PASSWORD = "correct-horse-42";

/** The configured root address becomes admin on sign-up (`[root]` config). */
export const ROOT_EMAIL = process.env.E2E_ROOT_EMAIL ?? "root@imkitchen.localhost";
export const ROOT_PASSWORD = process.env.E2E_ROOT_PASSWORD ?? "imkitchen";

/** The admin session (global-setup) is bound to this User-Agent. */
export const ADMIN_USER_AGENT = "Mozilla/5.0 (X11; Linux x86_64) imkitchen-e2e-admin/1.0";

export function uniqueEmail(prefix = "e2e"): string {
  const nonce = Math.random().toString(36).slice(2, 8);
  return `${prefix}-${Date.now()}-${nonce}@example.test`;
}

export function uniqueName(prefix: string): string {
  return `${prefix} ${Math.random().toString(36).slice(2, 7)}`;
}

export type Session = {
  id: string;
  email: string;
  password: string;
  token: string;
};

export type Me = {
  id: string;
  email: string;
  username: string | null;
  role: string;
  is_chef: boolean;
  is_admin: boolean;
  is_premium: boolean;
  premium_enabled: boolean;
};

export type Ingredient = {
  name: string;
  quantity: number;
  unit: string | null;
  category: string | null;
};

export type RecipeInput = {
  recipe_type: "Appetizer" | "MainCourse" | "Accompaniment" | "Dessert" | "Beverage" | "Condiment";
  name: string;
  description: string;
  household_size: number;
  prep_time: number;
  cook_time: number;
  ingredients: Ingredient[];
  instructions: { description: string; time_next: number }[];
  dietary_restrictions: string[];
  accepts_accompaniment: boolean;
  advance_prep: string;
};

/** A complete, valid recipe with sensible defaults; override what matters. */
export function recipeInput(overrides: Partial<RecipeInput> = {}): RecipeInput {
  return {
    recipe_type: "MainCourse",
    name: uniqueName("Egg fried rice"),
    description: "A proper description of the dish.",
    household_size: 2,
    prep_time: 10,
    cook_time: 20,
    ingredients: [
      { name: "Rice", quantity: 200, unit: "G", category: "Grocery" },
      { name: "Egg", quantity: 2, unit: null, category: "DairyAndEggs" },
    ],
    instructions: [
      { description: "Cook the rice.", time_next: 12 },
      { description: "Fry the eggs and fold in the rice.", time_next: 0 },
    ],
    dietary_restrictions: [],
    accepts_accompaniment: true,
    advance_prep: "",
    ...overrides,
  };
}

export async function register(
  request: APIRequestContext,
  email: string,
  password: string,
): Promise<Session> {
  const response = await request.post("/api/v1/auth/register", {
    data: { email, password },
  });
  expect(response.status(), await response.text()).toBe(201);
  const body = await response.json();
  return { id: body.user.id, email, password, token: body.token };
}

export async function login(
  request: APIRequestContext,
  email: string,
  password: string,
): Promise<Session> {
  const response = await request.post("/api/v1/auth/login", {
    data: { email, password },
  });
  expect(response.status(), await response.text()).toBe(200);
  const body = await response.json();
  return { id: body.user.id, email, password, token: body.token };
}

/**
 * Signs the browser in by storing the API token as the web session cookie,
 * then waits until a protected page accepts it: the device record the token
 * names is read from a projection that can lag the login by a moment.
 */
export async function installSession(
  context: BrowserContext,
  baseURL: string,
  token: string,
): Promise<void> {
  await context.addCookies([{ name: "auth_token", value: token, url: baseURL }]);
  await waitFor(
    async () => {
      const response = await context.request.get("/settings/general", { maxRedirects: 0 });
      return response.status() === 200 ? true : undefined;
    },
    { label: "the web session to be accepted" },
  );
}

/** Polls `probe` until it returns a value (not undefined) or the timeout passes. */
export async function waitFor<T>(
  probe: () => Promise<T | undefined>,
  { timeout = 15_000, interval = 250, label = "condition" } = {},
): Promise<T> {
  const deadline = Date.now() + timeout;
  for (;;) {
    const value = await probe();
    if (value !== undefined) return value;
    if (Date.now() > deadline) throw new Error(`Timed out waiting for ${label}`);
    await new Promise((resolve) => setTimeout(resolve, interval));
  }
}

export class Api {
  constructor(
    private readonly request: APIRequestContext,
    private readonly token: string,
  ) {}

  private async call(method: string, path: string, data?: unknown) {
    const response = await this.request.fetch(`/api/v1${path}`, {
      method,
      headers: { authorization: `Bearer ${this.token}` },
      data,
    });
    return response;
  }

  async me(): Promise<Me> {
    const response = await this.call("GET", "/me");
    expect(response.ok(), await response.text()).toBe(true);
    return response.json();
  }

  /** Marks every pending guided tour as skipped so no overlay intercepts clicks. */
  async skipTours(): Promise<void> {
    const response = await this.call("GET", "/tours");
    expect(response.ok()).toBe(true);
    const { tours } = (await response.json()) as { tours: { id: string; status: string }[] };
    for (const tour of tours) {
      if (tour.status !== "pending") continue;
      const skipped = await this.call("POST", `/tours/${tour.id}/skip`, { step: 0 });
      expect(skipped.status(), await skipped.text()).toBe(204);
    }
  }

  async setUsername(username: string): Promise<void> {
    const response = await this.call("POST", "/settings/username", { username });
    expect(response.ok(), await response.text()).toBe(true);
  }

  /**
   * Creates a recipe and waits until every read model a page may use lists
   * it: the detail (slug), the web detail page, and the dish composer's
   * candidates (the same projection the kitchen generates from).
   */
  async createRecipe(overrides: Partial<RecipeInput> = {}): Promise<{ id: string; slug: string; name: string }> {
    const input = recipeInput(overrides);
    const created = await this.call("POST", "/recipes");
    expect(created.status(), await created.text()).toBe(201);
    const { id } = await created.json();
    const updated = await this.call("PUT", `/recipes/${id}`, input);
    expect(updated.status(), await updated.text()).toBe(204);
    const slug: string = await waitFor(
      async () => {
        const response = await this.call("GET", `/recipes/${id}`);
        if (!response.ok()) return undefined;
        const detail = await response.json();
        return detail.slug ? detail.slug : undefined;
      },
      { label: `recipe ${id} to get its slug` },
    );
    await waitFor(
      async () => {
        const page = await this.request.get(`/r/${slug}`);
        return page.ok() ? true : undefined;
      },
      { label: `recipe page /r/${slug}` },
    );
    await waitFor(
      async () => {
        const response = await this.call(
          "GET",
          `/dishes/candidates?recipe_type=${input.recipe_type}&search=${encodeURIComponent(input.name)}&limit=100`,
        );
        if (!response.ok()) return undefined;
        const { candidates } = (await response.json()) as { candidates: { id: string }[] };
        return candidates.some((candidate) => candidate.id === id) ? true : undefined;
      },
      { label: `recipe ${id} among the candidates` },
    );
    return { id, slug, name: input.name };
  }

  /** Puts a recipe on the user's list (same as "Add to my list" on the detail page). */
  async addToList(recipeId: string): Promise<void> {
    // The shopping command reads the recipe from the kitchen's own projection,
    // which can lag the detail one: a 404 here only means "not yet".
    await waitFor(
      async () => {
        const response = await this.call("POST", `/recipes/${recipeId}/shopping`);
        if (response.status() === 404) return undefined;
        expect(response.ok(), await response.text()).toBe(true);
        return true;
      },
      { label: `recipe ${recipeId} to be addable to the list` },
    );
    await this.waitForKitchen((overview) => overview.kind === "list");
  }

  async generate(count: number): Promise<void> {
    const response = await this.call("POST", "/kitchen/generate", { count });
    expect(response.ok(), await response.text()).toBe(true);
    await this.waitForKitchen((overview) => overview.kind === "list");
  }

  async kitchen(): Promise<Record<string, unknown>> {
    const response = await this.call("GET", "/kitchen");
    expect(response.ok(), await response.text()).toBe(true);
    return response.json();
  }

  /** Polls the kitchen overview (fed by async projections) until `predicate` holds. */
  async waitForKitchen(
    predicate: (overview: Record<string, unknown>) => boolean,
  ): Promise<Record<string, unknown>> {
    return waitFor(
      async () => {
        const overview = await this.kitchen();
        return predicate(overview) ? overview : undefined;
      },
      { label: "kitchen overview" },
    );
  }

  async groceries(): Promise<Record<string, unknown>> {
    const response = await this.call("GET", "/groceries");
    expect(response.ok(), await response.text()).toBe(true);
    return response.json();
  }

  async waitForGroceries(
    predicate: (list: Record<string, unknown>) => boolean,
  ): Promise<Record<string, unknown>> {
    return waitFor(
      async () => {
        const list = await this.groceries();
        return predicate(list) ? list : undefined;
      },
      { label: "grocery list" },
    );
  }
}
