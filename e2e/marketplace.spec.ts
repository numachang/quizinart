import { test, expect } from "./fixtures";
import { registerUser, createQuiz, loginUser } from "./helpers";

test.describe("marketplace", () => {
  test("marketplace page shows shared quizzes", async ({ page, jsErrors }) => {
    // User A: create and share a quiz
    await registerUser(page);
    const quizName = `MktQuiz_${Date.now()}`;
    await createQuiz(page, quizName);

    // Toggle sharing ON
    await Promise.all([
      page.waitForResponse((resp) => resp.url().includes("/toggle-share/")),
      page.locator("button", { hasText: "Share" }).click(),
    ]);

    // User B: visit marketplace
    const browser = page.context().browser()!;
    const context2 = await browser.newContext();
    const page2 = await context2.newPage();
    await registerUser(page2);

    await page2.goto("/marketplace");
    await expect(page2.locator("h1")).toContainText("Marketplace");
    await expect(
      page2.locator("article", { hasText: quizName })
    ).toBeVisible();

    await context2.close();
  });

  test("non-shared quiz does NOT appear on marketplace", async ({
    page,
    jsErrors,
  }) => {
    // User A: create a quiz but do NOT share it
    await registerUser(page);
    const quizName = `Private_${Date.now()}`;
    await createQuiz(page, quizName);

    // User B: visit marketplace
    const browser = page.context().browser()!;
    const context2 = await browser.newContext();
    const page2 = await context2.newPage();
    await registerUser(page2);

    await page2.goto("/marketplace");
    await expect(page2.locator("h1")).toContainText("Marketplace");
    await expect(
      page2.locator("article", { hasText: quizName })
    ).toHaveCount(0);

    await context2.close();
  });

  test("user can import quiz from marketplace", async ({ page, jsErrors }) => {
    // User A: create and share a quiz
    await registerUser(page);
    const quizName = `Import_${Date.now()}`;
    await createQuiz(page, quizName);

    await Promise.all([
      page.waitForResponse((resp) => resp.url().includes("/toggle-share/")),
      page.locator("button", { hasText: "Share" }).click(),
    ]);

    // User B: visit marketplace and import
    const browser = page.context().browser()!;
    const context2 = await browser.newContext();
    const page2 = await context2.newPage();
    await registerUser(page2);

    await page2.goto("/marketplace");
    const quizCard = page2.locator("article", { hasText: quizName });
    await expect(quizCard).toBeVisible();

    await Promise.all([
      page2.waitForResponse((resp) =>
        resp.url().includes("/add-to-library/")
      ),
      quizCard.locator("button", { hasText: "Add to My Library" }).click(),
    ]);

    // Should be redirected to quiz dashboard
    await expect(page2.locator("h1")).toContainText(quizName);

    await context2.close();
  });

  test("already-imported quiz shows 'Go to Dashboard'", async ({
    page,
    jsErrors,
  }) => {
    // User A: create and share a quiz
    await registerUser(page);
    const quizName = `Already_${Date.now()}`;
    await createQuiz(page, quizName);

    await Promise.all([
      page.waitForResponse((resp) => resp.url().includes("/toggle-share/")),
      page.locator("button", { hasText: "Share" }).click(),
    ]);

    // User B: import via shared page, then check marketplace
    const browser = page.context().browser()!;
    const context2 = await browser.newContext();
    const page2 = await context2.newPage();
    await registerUser(page2);

    // Extract public_id from User A's dashboard URL
    const publicId = page.url().match(/\/quiz\/([^/]+)\/dashboard/)![1];
    await page2.goto(`/shared/${publicId}`);
    await Promise.all([
      page2.waitForResponse((resp) =>
        resp.url().includes("/add-to-library/")
      ),
      page2.locator("button", { hasText: "Add to My Library" }).click(),
    ]);

    // Visit marketplace — should show "Go to Dashboard" for this quiz
    await page2.goto("/marketplace");
    const quizCard = page2.locator("article", { hasText: quizName });
    await expect(
      quizCard.locator("a, button", { hasText: "Go to Dashboard" })
    ).toBeVisible();
    await expect(
      quizCard.locator("button", { hasText: "Add to My Library" })
    ).toHaveCount(0);

    await context2.close();
  });

  test("marketplace link is visible in navbar for authenticated users", async ({
    page,
    jsErrors,
  }) => {
    await registerUser(page);
    const marketplaceLink = page.locator('nav a[href="/marketplace"]');
    await expect(marketplaceLink).toBeVisible();
  });
});
