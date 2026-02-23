import { test, expect } from "./fixtures";
import { registerUser, createQuiz, loginUser } from "./helpers";

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

  test("share toggle shows share link and can be toggled off", async ({
    page,
    jsErrors,
  }) => {
    // Should be on quiz dashboard after createQuiz
    await expect(page.locator("h1")).toContainText(quizName);

    // Click "Share" button
    await Promise.all([
      page.waitForResponse((resp) => resp.url().includes("/toggle-share/")),
      page.locator("button", { hasText: "Share" }).click(),
    ]);

    // Share link should be visible
    const shareUrl = page.locator("#share-url");
    await expect(shareUrl).toBeVisible();
    await expect(shareUrl).toHaveAttribute("readonly", "");

    // "Stop Sharing" button should be visible
    await expect(
      page.locator("button", { hasText: "Stop Sharing" })
    ).toBeVisible();

    // Toggle OFF
    await Promise.all([
      page.waitForResponse((resp) => resp.url().includes("/toggle-share/")),
      page.locator("button", { hasText: "Stop Sharing" }).click(),
    ]);

    // Share link should disappear, "Share" button should be back
    await expect(page.locator("#share-url")).not.toBeVisible();
    await expect(
      page.locator("button", { hasText: "Share" })
    ).toBeVisible();
  });

  test("shared quiz page is accessible by another user", async ({
    page,
    jsErrors,
  }) => {
    const publicId = extractPublicId(page.url());

    // Toggle sharing ON
    await Promise.all([
      page.waitForResponse((resp) => resp.url().includes("/toggle-share/")),
      page.locator("button", { hasText: "Share" }).click(),
    ]);
    await expect(page.locator("#share-url")).toBeVisible();

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

    // Toggle sharing ON
    await Promise.all([
      page.waitForResponse((resp) => resp.url().includes("/toggle-share/")),
      page.locator("button", { hasText: "Share" }).click(),
    ]);

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

    // Toggle sharing ON
    await Promise.all([
      page.waitForResponse((resp) => resp.url().includes("/toggle-share/")),
      page.locator("button", { hasText: "Share" }).click(),
    ]);

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

    // Navigate to quiz start page directly (HTMX buttons don't work reliably in separate contexts)
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

    // Toggle sharing ON
    await Promise.all([
      page.waitForResponse((resp) => resp.url().includes("/toggle-share/")),
      page.locator("button", { hasText: "Share" }).click(),
    ]);

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

    // Toggle sharing ON
    await Promise.all([
      page.waitForResponse((resp) => resp.url().includes("/toggle-share/")),
      page.locator("button", { hasText: "Share" }).click(),
    ]);

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
});
