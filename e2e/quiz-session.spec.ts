import { test, expect } from "./fixtures";
import { registerUser, createQuiz } from "./helpers";

/** Answer the current question by selecting the first option and submitting */
async function answerCurrentQuestion(
  page: import("@playwright/test").Page
): Promise<void> {
  // Wait for question form and answer options to be fully visible
  await expect(page.locator("#question-form")).toBeVisible();
  await expect(
    page.locator('input[type="radio"][name="option"], input[type="checkbox"][name="options"]').first()
  ).toBeVisible();

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
    await expect(page.locator("#question-form")).toBeVisible();
    await expect(page.locator("h3")).toBeVisible();

    // Wait for answer options to be fully rendered (same pattern as answerCurrentQuestion)
    await expect(
      page.locator('input[type="radio"][name="option"], input[type="checkbox"][name="options"]').first()
    ).toBeVisible();

    // Submit button should be disabled initially
    await expect(page.locator("#submit-btn")).toBeDisabled();

    // Click the first option (questions are shuffled, so first question may be
    // single-choice radio or multiple-choice checkbox)
    const radioCount = await page.locator('input[type="radio"][name="option"]').count();
    if (radioCount > 0) {
      await page.locator('input[type="radio"][name="option"]').first().click();
    } else {
      await page.locator('input[type="checkbox"][name="options"]').first().click();
    }

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
      page.locator("td", { hasText: sessionName }).first()
    ).toBeVisible();
  });

  test("abandon quiz mid-session returns to dashboard", async ({
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
    await expect(page.locator("#question-form")).toBeVisible();

    // Click the "Quit" link to open abandon dialog
    await page.click("text=Quit");
    const dialog = page.locator("#abandon-dialog");
    await expect(dialog).toBeVisible();

    // Click Cancel — dialog should close
    await dialog.locator("button.secondary").click();
    await expect(dialog).not.toBeVisible();

    // Open again and confirm abandon
    await page.click("text=Quit");
    await expect(dialog).toBeVisible();
    await dialog.locator("button:not(.secondary)").click();

    // Should return to dashboard
    await expect(page.locator("h1")).toContainText(quizName);
  });

  test("previous question navigation works", async ({ page, jsErrors }) => {
    // Start session
    await page.click("text=Start New Session");
    await page.fill('input[name="question_count"]', "5");
    await Promise.all([
      page.waitForResponse((resp) => resp.url().includes("/start-session")),
      page.click('input[type="submit"]'),
    ]);

    // Answer first question
    await answerCurrentQuestion(page);

    // Go to second question
    await Promise.all([
      page.waitForResponse((resp) => resp.request().method() === "GET"),
      page.click(".nav-btn-next"),
    ]);
    await expect(page.locator("#question-form")).toBeVisible();

    // "Previous" button should be visible on Q2
    const prevBtn = page.locator(".nav-btn-back");
    await expect(prevBtn).toBeVisible();

    // Click Previous to go back to Q1
    await prevBtn.click();

    // Should see the answered Q1 (answer feedback visible)
    await expect(
      page.locator(".option-correct, .option-incorrect").first()
    ).toBeVisible();
  });

  test("next button navigates sequentially after going back", async ({
    page,
    jsErrors,
  }) => {
    // Start session with 5 questions
    await page.click("text=Start New Session");
    await page.fill('input[name="question_count"]', "5");
    await Promise.all([
      page.waitForResponse((resp) => resp.url().includes("/start-session")),
      page.click('input[type="submit"]'),
    ]);

    // Answer Q1
    await answerCurrentQuestion(page);
    // Verify we're on Q1 answer page
    await expect(page.locator("main strong")).toContainText("1");

    // Go to Q2
    await Promise.all([
      page.waitForResponse((resp) => resp.request().method() === "GET"),
      page.click(".nav-btn-next"),
    ]);
    await expect(page.locator("#question-form")).toBeVisible();

    // Answer Q2
    await answerCurrentQuestion(page);
    // Verify we're on Q2 answer page
    await expect(page.locator("main strong")).toContainText("2");

    // Now go back to Q1 via Previous twice (Q2 answer → Q1 answer)
    await page.click(".nav-btn-back");
    await expect(page.locator("main strong")).toContainText("1");

    // Click Next — should go to Q2 (not Q3)
    await page.click(".nav-btn-next");
    await expect(page.locator("main strong")).toContainText("2");

    // Click Next again — should go to Q3 (the next unanswered)
    await page.click(".nav-btn-next");
    await expect(page.locator("main strong")).toContainText("3");
    await expect(page.locator("#question-form")).toBeVisible();
  });

  test("next button navigates sequentially after resuming session", async ({
    page,
    jsErrors,
  }) => {
    // Start session and note the session name
    await page.click("text=Start New Session");
    await expect(page.locator('input[name="name"]')).toBeVisible({
      timeout: 30_000,
    });
    const sessionName = await page
      .locator('input[name="name"]')
      .inputValue();
    await page.fill('input[name="question_count"]', "5");
    await Promise.all([
      page.waitForResponse((resp) => resp.url().includes("/start-session")),
      page.click('input[type="submit"]'),
    ]);

    // Answer Q1 and Q2
    await answerCurrentQuestion(page);
    await Promise.all([
      page.waitForResponse((resp) => resp.request().method() === "GET"),
      page.click(".nav-btn-next"),
    ]);
    await answerCurrentQuestion(page);

    // Abandon session
    await page.click("text=Quit");
    await page.locator("#abandon-dialog button:not(.secondary)").click();
    await expect(page.locator("h1")).toContainText(quizName);

    // Resume via session history
    await page.click("text=Open Session History");
    await expect(
      page.locator("td", { hasText: sessionName })
    ).toBeVisible();
    const sessionRow = page.locator("tr", { hasText: sessionName });
    await sessionRow.locator("a", { hasText: /2\/5/ }).click();

    // Should resume at Q3 (first unanswered)
    await expect(page.locator("#question-form")).toBeVisible();
    await expect(page.locator("main strong")).toContainText("3");

    // Go back to Q1
    await page.click(".nav-btn-back");
    await expect(page.locator("main strong")).toContainText("2");
    await page.click(".nav-btn-back");
    await expect(page.locator("main strong")).toContainText("1");

    // Click Next — should go to Q2 (not Q3)
    await page.click(".nav-btn-next");
    await expect(page.locator("main strong")).toContainText("2");

    // Click Next — should go to Q3
    await page.click(".nav-btn-next");
    await expect(page.locator("main strong")).toContainText("3");
    await expect(page.locator("#question-form")).toBeVisible();
  });

  test("resume incomplete session from session history", async ({
    page,
    jsErrors,
  }) => {
    // Start session and note the session name
    await page.click("text=Start New Session");
    await expect(page.locator('input[name="name"]')).toBeVisible({ timeout: 30_000 });
    const sessionName = await page.locator('input[name="name"]').inputValue();
    await page.fill('input[name="question_count"]', "5");
    await Promise.all([
      page.waitForResponse((resp) => resp.url().includes("/start-session")),
      page.click('input[type="submit"]'),
    ]);

    // Answer first question only
    await answerCurrentQuestion(page);

    // Abandon the session
    await page.click("text=Quit");
    await page.locator("#abandon-dialog button:not(.secondary)").click();
    await expect(page.locator("h1")).toContainText(quizName);

    // Open session history
    await page.click("text=Open Session History");
    await expect(page.locator("td", { hasText: sessionName })).toBeVisible();

    // Click the progress link to resume (shows "1/5")
    const sessionRow = page.locator("tr", { hasText: sessionName });
    await sessionRow.locator("a", { hasText: /1\/5/ }).click();

    // Should show the question form with resuming indicator
    await expect(page.locator("#question-form")).toBeVisible();
  });

  test("session rename from session history", async ({ page, jsErrors }) => {
    // Complete a session
    await page.click("text=Start New Session");
    await expect(page.locator('input[name="name"]')).toBeVisible({ timeout: 30_000 });
    const sessionName = await page.locator('input[name="name"]').inputValue();
    await page.fill('input[name="question_count"]', "5");
    await Promise.all([
      page.waitForResponse((resp) => resp.url().includes("/start-session")),
      page.click('input[type="submit"]'),
    ]);
    await answerAllQuestions(page, 5);
    await Promise.all([
      page.waitForResponse((resp) => resp.url().includes("/results")),
      page.click(".nav-btn-next"),
    ]);
    await expect(page.locator("h1 mark")).toBeVisible();

    // Navigate to session history
    await page.click("text=Back to Dashboard");
    await page.click("text=Open Session History");
    await expect(page.locator("td", { hasText: sessionName })).toBeVisible();

    // Click rename icon
    const sessionRow = page.locator("tr", { hasText: sessionName });
    await sessionRow.getByTitle("Rename").click();

    // Rename dialog should appear
    const dialog = page.locator("#rename-dialog");
    await expect(dialog).toBeVisible();

    // Change the name
    const newName = `Renamed_${Date.now()}`;
    await dialog.locator("#rename-input").fill(newName);
    await dialog.locator("footer button:not(.secondary)").click();

    // Rename handler returns dashboard, so verify we're on the dashboard
    await expect(page.locator("h1")).toContainText(quizName);

    // Navigate to session history to confirm the rename
    await page.click("text=Open Session History");
    await expect(page.locator("td", { hasText: newName })).toBeVisible();
    await expect(page.locator("td", { hasText: sessionName })).not.toBeVisible();
  });

  test("session delete from session history", async ({ page, jsErrors }) => {
    // Complete a session
    await page.click("text=Start New Session");
    await expect(page.locator('input[name="name"]')).toBeVisible({ timeout: 30_000 });
    const sessionName = await page.locator('input[name="name"]').inputValue();
    await page.fill('input[name="question_count"]', "5");
    await Promise.all([
      page.waitForResponse((resp) => resp.url().includes("/start-session")),
      page.click('input[type="submit"]'),
    ]);
    await answerAllQuestions(page, 5);
    await Promise.all([
      page.waitForResponse((resp) => resp.url().includes("/results")),
      page.click(".nav-btn-next"),
    ]);
    await expect(page.locator("h1 mark")).toBeVisible();

    // Navigate to session history
    await page.click("text=Back to Dashboard");
    await page.click("text=Open Session History");
    await expect(page.locator("td", { hasText: sessionName })).toBeVisible();

    // Click delete icon
    const sessionRow = page.locator("tr", { hasText: sessionName });
    await sessionRow.getByTitle("Delete").click();

    // Confirm in custom dialog
    await expect(page.locator("#confirm-dialog")).toBeVisible();
    await page.locator("#confirm-dialog [data-confirm-ok]").click();

    // Session should be removed
    await expect(page.locator("td", { hasText: sessionName })).not.toBeVisible();
  });

  test("retry incorrect questions creates new session", async ({
    page,
    jsErrors,
  }) => {
    // Complete a session (some answers will be incorrect since we always pick first option)
    await page.click("text=Start New Session");
    await page.fill('input[name="question_count"]', "5");
    await Promise.all([
      page.waitForResponse((resp) => resp.url().includes("/start-session")),
      page.click('input[type="submit"]'),
    ]);
    await answerAllQuestions(page, 5);
    await Promise.all([
      page.waitForResponse((resp) => resp.url().includes("/results")),
      page.click(".nav-btn-next"),
    ]);
    await expect(page.locator("h1 mark")).toBeVisible();

    // Check if retry incorrect button exists (only if there are incorrect answers)
    const retryBtn = page.locator("button", { hasText: "Incorrect Questions" });
    const retryVisible = await retryBtn.isVisible();
    if (retryVisible) {
      await retryBtn.click();

      // Should start a new session with the question form
      await expect(page.locator("#question-form")).toBeVisible();
    }
    // If all answers were correct, retry button won't appear — that's fine
  });

  test("retry bookmarked questions creates new session", async ({
    page,
    jsErrors,
  }) => {
    // Start session and bookmark first question
    await page.click("text=Start New Session");
    await page.fill('input[name="question_count"]', "5");
    await Promise.all([
      page.waitForResponse((resp) => resp.url().includes("/start-session")),
      page.click('input[type="submit"]'),
    ]);

    // Bookmark the first question
    await page.locator(".bookmark-btn").click();
    await expect(page.locator(".bookmark-btn.active")).toBeVisible();

    // Answer all questions
    await answerAllQuestions(page, 5);

    // Go to results
    await Promise.all([
      page.waitForResponse((resp) => resp.url().includes("/results")),
      page.click(".nav-btn-next"),
    ]);
    await expect(page.locator("h1 mark")).toBeVisible();

    // Retry bookmarked button should be visible
    const retryBtn = page.locator("button", { hasText: "Bookmarked Questions" });
    await expect(retryBtn).toBeVisible();
    await retryBtn.click();

    // Should start a new session with the question form
    await expect(page.locator("#question-form")).toBeVisible();
  });

  test("review question from results page", async ({ page, jsErrors }) => {
    // Complete a session
    await page.click("text=Start New Session");
    await page.fill('input[name="question_count"]', "5");
    await Promise.all([
      page.waitForResponse((resp) => resp.url().includes("/start-session")),
      page.click('input[type="submit"]'),
    ]);
    await answerAllQuestions(page, 5);
    await Promise.all([
      page.waitForResponse((resp) => resp.url().includes("/results")),
      page.click(".nav-btn-next"),
    ]);
    await expect(page.locator("h1 mark")).toBeVisible();

    // Click a question row in the results table to review it
    const questionRow = page.locator("table").last().locator("tbody tr").first();
    await questionRow.click();

    // Should show the answered question with feedback
    await expect(
      page.locator(".option-correct, .option-incorrect").first()
    ).toBeVisible();

    // "Back to Results" button should be visible (from=report context)
    await expect(
      page.locator("button", { hasText: "Back to Results" })
    ).toBeVisible();

    // Click back to results
    await page.locator("button", { hasText: "Back to Results" }).click();

    // Should return to results page
    await expect(page.locator("h1 mark")).toBeVisible();
  });
});
