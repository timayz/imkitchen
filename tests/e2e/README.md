# End-to-end tests

Playwright drives a real browser against a real server: every critical flow
(sign-up, login, password reset, recipes, the kitchen list, cooking,
groceries, dishes, settings, admin) has a happy path and its main error
paths covered here.

## Running locally

```sh
make up            # Traefik (https://imkitchen.localhost) + MailDev
npm test           # every project; `npm run test:e2e:desktop` for chromium only
npm run test:ui    # the Playwright UI
npm run test:report
```

Playwright itself comes from the nix dev shell (it is not in `package.json`;
see the "Playwright nix runner" note if `npm test` cannot find it). If no
server is listening on port 3000, Playwright migrates and starts one on its
own SQLite file (`e2e.db`, see `playwright.config.ts`); a running dev server
is reused as is.

Point the tests somewhere else with `PLAYWRIGHT_BASE_URL`, e.g. at a server
started by hand:

```sh
IMKITCHEN__DATABASE__URL="sqlite:/tmp/e2e.db?mode=rwc" IMKITCHEN__SERVER__URL=http://localhost:3000 \
  cargo run -q -- migrate && cargo run -q -- serve
PLAYWRIGHT_BASE_URL=http://localhost:3000 npm test
```

The password-reset tests read the mail from MailDev's REST API
(`MAILDEV_URL`, default `http://localhost:1080`) and skip when it is not
reachable. The premium-upgrade test skips when `[premium]` is not configured
(every account is then premium).

## Layout

```
tests/e2e/
├── fixtures.ts        test.extend: `account` (fresh signed-in user), `api`, `admin`
├── global-setup.ts    creates the root administrator once per run
├── support/api.ts     JSON API client used to seed accounts, recipes, lists
├── support/mail.ts    MailDev helper (wait for a mail, extract the reset link)
├── pages/*.ts         page objects: locators and actions per screen
└── *.spec.ts          one file per feature area
```

Conventions that keep the suite reliable:

- **Fresh account per test.** The `account` fixture registers a user through
  `/api/v1/auth/register` from the browser context (the session is bound to
  the User-Agent) and installs the token as the `auth_token` cookie. Tests
  never share state, so they run fully in parallel.
- **Seed through the API, assert through the browser.** `api.createRecipe()`
  returns only once every read model lists the recipe (the detail, the
  `/r/{slug}` page, the dish composer's candidates), because projections are
  asynchronous. Prefer `expect.poll` / auto-retrying assertions over sleeps.
- **Guided tours are skipped** by the fixtures so overlays never intercept
  clicks; `tour.spec.ts` signs in on its own to test them.
- **Served HTML is minified**: default attribute values disappear (`<button
  type="submit">` becomes `<button>`, the "All" chip loses its empty `value`),
  so select by role, label, name or id, never by those attributes.
- **Visually hidden inputs** (`sr-only` radios and checkboxes) are set with
  `check({ force: true })` or by clicking their label.
- A test title containing `@mobile` also runs in the phone project.

## Adding a test

1. Put the screen's locators and actions in `pages/` (or extend one).
2. Seed what the test is not about through `api`.
3. Keep assertions on what the user sees; when a page depends on a
   projection, wait on an auto-retrying assertion or `expect.poll`.

CI runs the suite on every pull request (`.github/workflows/e2e.yaml`) and
uploads the HTML report, plus traces, screenshots and videos on failure.
