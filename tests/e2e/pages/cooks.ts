import { type Locator, type Page } from "@playwright/test";

/** A chef's public page, `/cooks/{username}`. */
export class CookPage {
  readonly header: Locator;
  readonly follow: Locator;
  readonly followerCount: Locator;

  constructor(private readonly page: Page) {
    this.header = page.locator("#cook-header");
    this.follow = page.locator("#follow-btn");
    this.followerCount = page.locator("#follower-count");
  }

  async goto(username: string) {
    await this.page.goto(`/cooks/${username}`);
  }
}

/** The chefs the user follows, `/cooks`. */
export class FollowingPage {
  readonly list: Locator;

  constructor(private readonly page: Page) {
    this.list = page.locator("#cooks-list");
  }

  async goto() {
    await this.page.goto("/cooks");
  }

  card(username: string) {
    return this.list.locator(`#cook-${username}`);
  }

  cards() {
    return this.list.locator("a[id^='cook-']");
  }
}
