import { test, expect } from "./fixtures";
import { registerUser, createQuiz } from "./helpers";

test.describe("quiz lifecycle", () => {
  test.beforeEach(async ({ page }) => {
    await registerUser(page);
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

    // Accept the confirm dialog before clicking delete
    page.on("dialog", (dialog) => dialog.accept());

    // Click delete icon on the quiz card
    const quizCard = page.locator("article", { has: page.locator("h3", { hasText: quizName }) });
    await quizCard.getByTitle("Delete").click();

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
