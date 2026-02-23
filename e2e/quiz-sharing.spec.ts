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

test.describe("quiz sharing", () => {
  let quizName: string;

  test.beforeEach(async ({ page }) => {
    await registerUser(page);
    quizName = `ShareQuiz_${Date.now()}`;
    await createQuiz(page, quizName);
  });

  /** Extract the public_id from a dashboard URL like /quiz/{public_id}/dashboard */
  function extractPublicId(url: string): string {
    const match = url.match(/\/quiz\/([^/]+)\/dashboard/);
    if (!match) throw new Error(`Could not extract public_id from URL: ${url}`);
    return match[1];
  }

  test("share toggle icon works on quiz list card", async ({
    page,
    jsErrors,
  }) => {
    // Navigate to quiz list
    await page.goto("/");
    await expect(page.locator("h1")).toContainText("My Quizzes");

    // Find the quiz card
    const quizCard = page.locator("article", {
      has: page.locator("h3", { hasText: quizName }),
    });

    // Should show public_off icon (not shared by default)
    const shareToggle = quizCard.locator("[data-share-toggle]");
    await expect(shareToggle).toBeVisible();
    await expect(shareToggle).toContainText("public_off");

    // Toggle ON
    await Promise.all([
      page.waitForResponse((resp) => resp.url().includes("/toggle-share/")),
      shareToggle.click(),
    ]);

    // Should now show public icon (not public_off)
    const updatedToggle = quizCard.locator("[data-share-toggle]");
    await expect(updatedToggle).toContainText("public");
    await expect(updatedToggle).not.toContainText("public_off");

    // Toggle OFF
    await Promise.all([
      page.waitForResponse((resp) => resp.url().includes("/toggle-share/")),
      updatedToggle.click(),
    ]);

    // Should be back to public_off
    const finalToggle = quizCard.locator("[data-share-toggle]");
    await expect(finalToggle).toContainText("public_off");
  });

  test("shared quiz page is accessible by another user", async ({
    page,
    jsErrors,
  }) => {
    const publicId = extractPublicId(page.url());

    // Toggle sharing ON from quiz list
    await toggleShare(page, quizName);

    // User B: new context
    const browser = page.context().browser()!;
    const context2 = await browser.newContext();
    const page2 = await context2.newPage();
    await registerUser(page2);

    // Navigate to the shared quiz page
    await page2.goto(`/shared/${publicId}`);
    await expect(page2.locator("h1")).toContainText(quizName);
    await expect(
      page2.locator("button", { hasText: "Add to My Library" })
    ).toBeVisible();

    await context2.close();
  });

  test("non-shared quiz page shows not available", async ({
    page,
    jsErrors,
  }) => {
    const publicId = extractPublicId(page.url());

    // Sharing is OFF by default — don't toggle

    // User B: new context
    const browser = page.context().browser()!;
    const context2 = await browser.newContext();
    const page2 = await context2.newPage();
    await registerUser(page2);

    // Navigate to the shared quiz page
    await page2.goto(`/shared/${publicId}`);
    await expect(page2.locator("h1")).toContainText("Quiz Not Available");

    await context2.close();
  });

  test("user can add shared quiz to library", async ({ page, jsErrors }) => {
    const publicId = extractPublicId(page.url());

    // Toggle sharing ON from quiz list
    await toggleShare(page, quizName);

    // User B: new context
    const browser = page.context().browser()!;
    const context2 = await browser.newContext();
    const page2 = await context2.newPage();
    await registerUser(page2);

    // Navigate to shared page and add to library
    await page2.goto(`/shared/${publicId}`);
    await expect(page2.locator("h1")).toContainText(quizName);

    await Promise.all([
      page2.waitForResponse((resp) =>
        resp.url().includes("/add-to-library/")
      ),
      page2.locator("button", { hasText: "Add to My Library" }).click(),
    ]);

    // Should be redirected to quiz dashboard
    await expect(page2.locator("h1")).toContainText(quizName);

    // Go to homepage — quiz should appear in "My Quizzes"
    await page2.goto("/");
    await expect(page2.locator("h1")).toContainText("My Quizzes");
    await expect(
      page2.locator("article h3", { hasText: quizName })
    ).toBeVisible();

    await context2.close();
  });

  test("user can start session on imported shared quiz", async ({
    page,
    jsErrors,
  }) => {
    const publicId = extractPublicId(page.url());

    // Toggle sharing ON from quiz list
    await toggleShare(page, quizName);

    // User B: new context
    const browser = page.context().browser()!;
    const context2 = await browser.newContext();
    const page2 = await context2.newPage();
    await registerUser(page2);

    // Add to library
    await page2.goto(`/shared/${publicId}`);
    await Promise.all([
      page2.waitForResponse((resp) =>
        resp.url().includes("/add-to-library/")
      ),
      page2.locator("button", { hasText: "Add to My Library" }).click(),
    ]);

    // Start a session from the dashboard
    await expect(page2.locator("h1")).toContainText(quizName);

    // Navigate to quiz start page directly
    await page2.goto(`/quiz/${publicId}`);
    await expect(page2.locator('input[name="question_count"]')).toBeVisible();
    await page2.fill('input[name="question_count"]', "5");
    await Promise.all([
      page2.waitForResponse((resp) => resp.url().includes("/start-session")),
      page2.click('input[type="submit"]'),
    ]);

    // Question form should be visible
    await expect(page2.locator("#question-form")).toBeVisible();

    await context2.close();
  });

  test("owner cannot delete quiz after another user imported it", async ({
    page,
    jsErrors,
  }) => {
    const publicId = extractPublicId(page.url());

    // Toggle sharing ON from quiz list
    await toggleShare(page, quizName);

    // User B: add to library
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
    await context2.close();

    // Owner (User A): try to delete the quiz
    await page.goto("/");
    await expect(
      page.locator("article h3", { hasText: quizName })
    ).toBeVisible();

    // Accept the confirm dialog
    page.on("dialog", (dialog) => dialog.accept());

    const quizCard = page.locator("article", {
      has: page.locator("h3", { hasText: quizName }),
    });
    await quizCard.getByTitle("Delete").click();

    // Delete should be blocked — error message visible
    await expect(page.locator("article").filter({ hasText: /cannot be deleted|other users/ })).toBeVisible();
    // Quiz should still be in the list
    await expect(
      page.locator("article h3", { hasText: quizName })
    ).toBeVisible();
  });

  test("already imported quiz shows 'Go to Dashboard' on shared page", async ({
    page,
    jsErrors,
  }) => {
    const publicId = extractPublicId(page.url());

    // Toggle sharing ON from quiz list
    await toggleShare(page, quizName);

    // User B: add to library
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

    // Visit shared page again — should show "Go to Dashboard" instead of "Add to Library"
    await page2.goto(`/shared/${publicId}`);
    await expect(
      page2.locator("button", { hasText: "Go to Dashboard" })
    ).toBeVisible();
    await expect(
      page2.locator("button", { hasText: "Add to My Library" })
    ).not.toBeVisible();

    await context2.close();
  });

  test("imported quiz does not show share toggle icon", async ({
    page,
    jsErrors,
  }) => {
    const publicId = extractPublicId(page.url());

    // Toggle sharing ON from quiz list
    await toggleShare(page, quizName);

    // User B: import the quiz
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

    // User B goes to quiz list — imported quiz should NOT have share toggle
    await page2.goto("/");
    const quizCard = page2.locator("article", {
      has: page2.locator("h3", { hasText: quizName }),
    });
    await expect(quizCard).toBeVisible();
    await expect(quizCard.locator("[data-share-toggle]")).toHaveCount(0);

    await context2.close();
  });
});
