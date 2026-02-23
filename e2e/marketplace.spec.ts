import { test, expect } from "./fixtures";
import { registerUser, createQuiz, loginUser } from "./helpers";

/**
 * Toggle the share state for a quiz from the quiz list page.
 * Navigates to "/", finds the quiz card, and clicks the share toggle icon.
 */
async function toggleShare(page: import("@playwright/test").Page, quizName: string) {
  await page.goto("/");
  const card = page.locator("article", {
    has: page.locator("h3", { hasText: quizName }),
  });
  await Promise.all([
    page.waitForResponse((resp) => resp.url().includes("/toggle-share/")),
    card.locator("[data-share-toggle]").click(),
  ]);
}

test.describe("marketplace", () => {
  test("marketplace page shows shared quizzes", async ({ page, jsErrors }) => {
    // User A: create and share a quiz
    await registerUser(page);
    const quizName = `MktQuiz_${Date.now()}`;
    await createQuiz(page, quizName);

    // Toggle sharing ON from quiz list
    await toggleShare(page, quizName);

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

    // Toggle sharing ON from quiz list
    await toggleShare(page, quizName);

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

    // Extract public_id from dashboard URL before navigating away
    const publicId = page.url().match(/\/quiz\/([^/]+)\/dashboard/)![1];

    // Toggle sharing ON from quiz list
    await toggleShare(page, quizName);

    // User B: import via shared page, then check marketplace
    const browser = page.context().browser()!;
    const context2 = await browser.newContext();
    const page2 = await context2.newPage();
    await registerUser(page2);

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

test.describe("marketplace search and filter", () => {
  test("search filters quizzes by name", async ({ page, jsErrors }) => {
    // User A: create and share two quizzes
    await registerUser(page);
    const quizName1 = `SearchAlpha_${Date.now()}`;
    const quizName2 = `SearchBeta_${Date.now()}`;

    await createQuiz(page, quizName1);
    await toggleShare(page, quizName1);

    await page.goto("/");
    await createQuiz(page, quizName2);
    await toggleShare(page, quizName2);

    // User B: search on marketplace
    const browser = page.context().browser()!;
    const context2 = await browser.newContext();
    const page2 = await context2.newPage();
    await registerUser(page2);

    await page2.goto("/marketplace");
    await expect(page2.locator("h1")).toContainText("Marketplace");

    // Both quizzes should be visible initially
    await expect(
      page2.locator("article", { hasText: quizName1 })
    ).toBeVisible();
    await expect(
      page2.locator("article", { hasText: quizName2 })
    ).toBeVisible();

    // Search for "Alpha" — only first quiz should remain
    const searchInput = page2.locator('input[name="q"]');
    await searchInput.fill("Alpha");
    // Wait for HTMX to update results
    await page2.waitForResponse((resp) =>
      resp.url().includes("/marketplace/search")
    );
    await expect(
      page2.locator("#quiz-results article", { hasText: quizName1 })
    ).toBeVisible();
    await expect(
      page2.locator("#quiz-results article", { hasText: quizName2 })
    ).toHaveCount(0);

    await context2.close();
  });

  test("category filter works", async ({ page, jsErrors }) => {
    // User A: create and share a quiz (test-quiz.json has Science and History)
    await registerUser(page);
    const quizName = `CatFilter_${Date.now()}`;
    await createQuiz(page, quizName);
    await toggleShare(page, quizName);

    // User B: filter by category on marketplace
    const browser = page.context().browser()!;
    const context2 = await browser.newContext();
    const page2 = await context2.newPage();
    await registerUser(page2);

    await page2.goto("/marketplace");

    // Category dropdown should be available
    const categorySelect = page2.locator('select[name="category"]');
    await expect(categorySelect).toBeVisible();

    // Should have at least "All Categories" and some real categories
    const options = categorySelect.locator("option");
    await expect(options).not.toHaveCount(0);

    await context2.close();
  });

  test("empty search shows no-results message", async ({ page, jsErrors }) => {
    await registerUser(page);
    await page.goto("/marketplace");

    const searchInput = page.locator('input[name="q"]');
    await searchInput.fill("nonexistent_quiz_xyz_999");
    await page.waitForResponse((resp) =>
      resp.url().includes("/marketplace/search")
    );

    await expect(page.locator("#quiz-results")).toContainText(
      /No quizzes found|No shared quizzes/
    );
  });
});
