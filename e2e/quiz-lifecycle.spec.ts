import { test, expect } from "./fixtures";
import { registerUser, createQuiz } from "./helpers";

test.describe("quiz lifecycle", () => {
  test.beforeEach(async ({ page }) => {
    await registerUser(page);
  });

  test("quiz list shows marketplace and upload import cards", async ({
    page,
    jsErrors,
  }) => {
    await page.goto("/");

    // Marketplace card should exist and appear first
    const marketplaceCard = page.locator("#marketplace-card");
    await expect(marketplaceCard).toBeVisible();

    // Upload card should exist
    const uploadCard = page.locator("#upload-card");
    await expect(uploadCard).toBeVisible();

    // Marketplace card should come before upload card
    const cards = page.locator(".quiz-card, #marketplace-card, #upload-card");
    const marketplaceIndex = await cards.evaluateAll((els) =>
      els.findIndex((el) => el.id === "marketplace-card")
    );
    const uploadIndex = await cards.evaluateAll((els) =>
      els.findIndex((el) => el.id === "upload-card")
    );
    expect(marketplaceIndex).toBeLessThan(uploadIndex);
  });

  test("marketplace import card navigates to marketplace", async ({
    page,
    jsErrors,
  }) => {
    await page.goto("/");
    await page.locator("#marketplace-card").click();
    await expect(page.locator("h1")).toContainText("Marketplace");
  });

  test("upload import card opens upload dialog", async ({
    page,
    jsErrors,
  }) => {
    await page.goto("/");
    await page.locator("#upload-card").click();
    await expect(page.locator("#create-dialog")).toBeVisible();
  });

  test("create quiz navigates to quiz dashboard", async ({
    page,
    jsErrors,
  }) => {
    const quizName = `TestQuiz_${Date.now()}`;
    await createQuiz(page, quizName);

    // Quiz dashboard should show the quiz name and stats table
    await expect(page.locator("h1")).toContainText(quizName);
    await expect(page.locator("table").first()).toBeVisible();
  });

  test("quiz appears on quiz list after creation", async ({
    page,
    jsErrors,
  }) => {
    const quizName = `TestQuiz_${Date.now()}`;
    await createQuiz(page, quizName);

    // Navigate back to quiz list
    await page.goto("/");
    await expect(page.locator("h1")).toContainText("My Quizzes");

    // Quiz card should be visible with the quiz name
    await expect(page.locator("article h3", { hasText: quizName })).toBeVisible();
  });

  test("rename quiz updates quiz name", async ({
    page,
    jsErrors,
  }) => {
    const quizName = `TestQuiz_${Date.now()}`;
    await createQuiz(page, quizName);

    // Go back to quiz list
    await page.goto("/");
    await expect(page.locator("article h3", { hasText: quizName })).toBeVisible();

    // Click rename icon on the quiz card
    const quizCard = page.locator("article", { has: page.locator("h3", { hasText: quizName }) });
    await quizCard.getByTitle("Rename").click();

    // Rename dialog should appear
    const dialog = page.locator('#rename-dialog');
    await expect(dialog).toBeVisible();

    // Change the name
    const newName = `Renamed_${Date.now()}`;
    await dialog.locator('#rename-input').fill(newName);
    await dialog.locator('footer button:not(.secondary)').click();

    // Quiz list should update with new name
    await expect(page.locator("article h3", { hasText: newName })).toBeVisible();
    await expect(page.locator("article h3", { hasText: quizName })).not.toBeVisible();
  });

  test("delete quiz removes it from quiz list", async ({
    page,
    jsErrors,
  }) => {
    const quizName = `TestQuiz_${Date.now()}`;
    await createQuiz(page, quizName);

    // Go back to quiz list
    await page.goto("/");
    await expect(page.locator("article h3", { hasText: quizName })).toBeVisible();

    // Click delete icon on the quiz card
    const quizCard = page.locator("article", { has: page.locator("h3", { hasText: quizName }) });
    await quizCard.getByTitle("Delete").click();

    // Confirm in custom dialog
    await expect(page.locator("#confirm-dialog")).toBeVisible();
    await page.locator("#confirm-dialog [data-confirm-ok]").click();

    // Quiz card should be removed
    await expect(page.locator("article h3", { hasText: quizName })).not.toBeVisible();
  });

  test("most recently played quiz appears first", async ({
    page,
    jsErrors,
  }) => {
    // Create two quizzes
    const quiz1 = `Quiz1_${Date.now()}`;
    await createQuiz(page, quiz1);
    await page.goto("/");
    const quiz2 = `Quiz2_${Date.now()}`;
    await createQuiz(page, quiz2);

    // Go back to quiz list — quiz2 was created second but neither has sessions
    await page.goto("/");

    // Play quiz1: click its card to open dashboard
    await page.locator("article h3 a", { hasText: quiz1 }).click();
    await expect(page.locator("h1")).toContainText(quiz1);
    await page.waitForLoadState("networkidle");

    // Start a session and answer one question
    await page.click("text=Start New Session");
    await page.fill('input[name="question_count"]', "5");
    await Promise.all([
      page.waitForResponse((resp) => resp.url().includes("/start-session")),
      page.click('input[type="submit"]'),
    ]);
    await expect(page.locator("#question-form")).toBeVisible();
    await expect(
      page.locator('input[type="radio"][name="option"], input[type="checkbox"][name="options"]').first()
    ).toBeVisible();
    const radioCount = await page.locator('input[type="radio"][name="option"]').count();
    if (radioCount > 0) {
      await page.locator('input[type="radio"][name="option"]').first().click();
    } else {
      await page.locator('input[type="checkbox"][name="options"]').first().click();
    }
    await Promise.all([
      page.waitForResponse((resp) => resp.url().includes("/submit-answer")),
      page.click("#submit-btn"),
    ]);

    // Abandon and go back to quiz list
    await page.click("text=Quit");
    await page.locator("#abandon-dialog button:not(.secondary)").click();
    await expect(page.locator("h1")).toContainText(quiz1);

    await page.goto("/");
    await expect(page.locator("h1")).toContainText("My Quizzes");

    // quiz1 (most recently played) should be the first quiz card
    const firstCard = page.locator(".quiz-card").first();
    await expect(firstCard.locator("h3")).toContainText(quiz1);
  });
});
