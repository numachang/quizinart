import { test, expect } from "./fixtures";
import { registerUser, createQuiz } from "./helpers";

/** Answer the current question by selecting the first option and submitting */
async function answerCurrentQuestion(
  page: import("@playwright/test").Page
): Promise<void> {
  // Wait for question form and answer options to be ready
  await expect(page.locator("#question-form")).toBeVisible();
  await expect(
    page.locator('input[type="radio"][name="option"], input[type="checkbox"][name="options"]').first()
  ).toBeAttached();

  const radioCount = await page
    .locator('input[type="radio"][name="option"]')
    .count();

  if (radioCount > 0) {
    await page.locator('input[type="radio"][name="option"]').first().click();
  } else {
    await page
      .locator('input[type="checkbox"][name="options"]')
      .first()
      .click();
  }

  await Promise.all([
    page.waitForResponse((resp) => resp.url().includes("/submit-answer")),
    page.click("#submit-btn"),
  ]);

  // Wait for answer feedback
  await expect(
    page.locator(".option-correct, .option-incorrect").first()
  ).toBeVisible();
}

/** Answer all questions in a session, ending on the last answer page */
async function answerAllQuestions(
  page: import("@playwright/test").Page,
  count: number
): Promise<void> {
  for (let i = 0; i < count; i++) {
    await answerCurrentQuestion(page);
    if (i < count - 1) {
      await Promise.all([
        page.waitForResponse((resp) => resp.request().method() === "GET"),
        page.click(".nav-btn-next"),
      ]);
      // Wait for next question to load (htmx swap)
      await expect(page.locator("#question-form")).toBeVisible();
    }
  }
}

test.describe("quiz session", () => {
  let quizName: string;

  test.beforeEach(async ({ page }) => {
    await registerUser(page);
    quizName = `SessionQuiz_${Date.now()}`;
    await createQuiz(page, quizName);
    // Ensure HTMX scripts are fully loaded after page navigation
    await page.waitForLoadState("networkidle");
  });

  test("start session and answer first question", async ({
    page,
    jsErrors,
  }) => {
    // Go to start session page
    await page.click("text=Start New Session");
    await page.fill('input[name="question_count"]', "5");
    await Promise.all([
      page.waitForResponse((resp) => resp.url().includes("/start-session")),
      page.click('input[type="submit"]'),
    ]);

    // First question should be displayed
    await expect(page.locator("h3")).toBeVisible();
    await expect(page.locator("#question-form")).toBeVisible();

    // Submit button should be disabled initially
    await expect(page.locator("#submit-btn")).toBeDisabled();

    // Click the first radio option
    await page.locator('input[type="radio"][name="option"]').first().click();

    // Submit button should now be enabled
    await expect(page.locator("#submit-btn")).not.toBeDisabled();

    // Submit the answer
    await Promise.all([
      page.waitForResponse((resp) => resp.url().includes("/submit-answer")),
      page.click("#submit-btn"),
    ]);

    // Answer feedback should be displayed
    await expect(
      page.locator(".option-correct, .option-incorrect").first()
    ).toBeVisible();
    await expect(page.locator(".explanation").first()).toBeVisible();
  });

  test("complete full session and view results", async ({
    page,
    jsErrors,
  }) => {
    // Start session
    await page.click("text=Start New Session");
    await page.fill('input[name="question_count"]', "5");
    await Promise.all([
      page.waitForResponse((resp) => resp.url().includes("/start-session")),
      page.click('input[type="submit"]'),
    ]);

    // Answer all 5 questions
    await answerAllQuestions(page, 5);

    // After last question, click "See Results"
    await Promise.all([
      page.waitForResponse((resp) => resp.url().includes("/results")),
      page.click(".nav-btn-next"),
    ]);

    // Results page should show score
    await expect(page.locator("h1 mark")).toBeVisible();
    // Should show the question list table
    await expect(page.locator("table").first()).toBeVisible();
  });

  test("bookmark toggle works", async ({ page, jsErrors }) => {
    // Start session
    await page.click("text=Start New Session");
    await page.fill('input[name="question_count"]', "5");
    await Promise.all([
      page.waitForResponse((resp) => resp.url().includes("/start-session")),
      page.click('input[type="submit"]'),
    ]);

    // Bookmark button should exist and not be active
    const bookmarkBtn = page.locator(".bookmark-btn");
    await expect(bookmarkBtn).toBeVisible();
    await expect(bookmarkBtn).not.toHaveClass(/active/);

    // Click bookmark
    await bookmarkBtn.click();

    // Should become active
    await expect(page.locator(".bookmark-btn.active")).toBeVisible();

    // Click again to unbookmark
    await page.locator(".bookmark-btn.active").click();

    // Should no longer be active
    await expect(page.locator(".bookmark-btn")).not.toHaveClass(/active/);
  });

  test("session appears in session history", async ({ page, jsErrors }) => {
    // Start session
    await page.click("text=Start New Session");
    await expect(page.locator('input[name="name"]')).toBeVisible({ timeout: 30_000 });
    const sessionName = await page
      .locator('input[name="name"]')
      .inputValue();
    await page.fill('input[name="question_count"]', "5");
    await Promise.all([
      page.waitForResponse((resp) => resp.url().includes("/start-session")),
      page.click('input[type="submit"]'),
    ]);

    // Answer all 5 questions
    await answerAllQuestions(page, 5);

    // Go to results
    await Promise.all([
      page.waitForResponse((resp) => resp.url().includes("/results")),
      page.click(".nav-btn-next"),
    ]);
    await expect(page.locator("h1 mark")).toBeVisible();

    // Navigate to dashboard
    await page.click("text=Back to Dashboard");
    await expect(page.locator("h1")).toContainText(quizName);

    // Open session history
    await page.click("text=Open Session History");

    // Session should appear in the table
    await expect(
      page.locator("td", { hasText: sessionName })
    ).toBeVisible();
  });
});
