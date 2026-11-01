import { test, expect } from "./fixtures";
import { AdminUsersPage } from "./pages/admin";
import { CookPage, FollowingPage } from "./pages/cooks";
import { RecipeDetailPage } from "./pages/recipes";
import { PASSWORD, installSession, register, uniqueEmail, uniqueName, Api } from "./support/api";

test.describe("following a chef", () => {
  test("a cook follows a chef and finds them under Following", async ({
    page,
    browser,
    baseURL,
    account,
    api,
    admin,
  }) => {
    // A chef: the role (granted by an admin), a username, and a shared recipe.
    const users = new AdminUsersPage(admin.page);
    await users.goto();
    await users.search.fill(account.email);
    await expect(users.row(account.id)).toBeVisible();
    await users.setRole(account.id, "Chef");
    await expect(users.row(account.id)).toContainText("Chef");

    const username = `chef_${account.id.slice(-6).toLowerCase()}`;
    await api.setUsername(username);
    const recipe = await api.createRecipe({ name: uniqueName("Followed tarte") });
    const detail = new RecipeDetailPage(page);
    await detail.goto(recipe.slug);
    await detail.makePublic();
    await expect(page.locator("#share-button")).toContainText("Make Private");

    // The chef's own page has no Follow button.
    const ownPage = new CookPage(page);
    await expect
      .poll(
        async () => {
          await ownPage.goto(username);
          return ownPage.header.isVisible();
        },
        { timeout: 20_000 },
      )
      .toBe(true);
    await expect(ownPage.header).toContainText(`@${username}`);
    await expect(ownPage.follow).toHaveCount(0);
    await expect(ownPage.followerCount).toHaveText("0");

    // Another cook follows the chef.
    const other = await browser.newContext();
    try {
      const session = await register(other.request, uniqueEmail("follower"), PASSWORD);
      await installSession(other, baseURL!, session.token);
      await new Api(other.request, session.token).skipTours();
      const otherPage = await other.newPage();

      // Nothing followed yet.
      const following = new FollowingPage(otherPage);
      await following.goto();
      await expect(following.list).toContainText("You are not following any chef yet");

      const cook = new CookPage(otherPage);
      await cook.goto(username);
      await expect(cook.follow).toContainText("Follow");
      await cook.follow.click();
      await expect(cook.follow).toContainText("Following");
      await otherPage.reload();
      await expect(cook.follow).toContainText("Following");

      // The list and the count come from a projection: poll.
      await expect
        .poll(
          async () => {
            await following.goto();
            return following.card(username).isVisible();
          },
          { timeout: 20_000 },
        )
        .toBe(true);
      await expect(following.cards()).toHaveCount(1);
      await expect(following.card(username)).toContainText(`@${username}`);
      await following.card(username).click();
      await otherPage.waitForURL(`**/cooks/${username}`);
      await expect(cook.followerCount).toHaveText("1");

      // Unfollow: the list empties again.
      await cook.follow.click();
      await expect(cook.follow).toContainText("Follow");
      await expect(cook.follow).not.toContainText("Following");
      await expect
        .poll(
          async () => {
            await following.goto();
            return following.cards().count();
          },
          { timeout: 20_000 },
        )
        .toBe(0);
    } finally {
      await other.close();
    }
  });
});
