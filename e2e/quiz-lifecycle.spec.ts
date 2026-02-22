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
});
