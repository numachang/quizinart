import { test, expect } from "./fixtures";
import { registerUser, createQuiz } from "./helpers";

/** Answer the current question by selecting the first option and submitting */
async function answerCurrentQuestion(
  page: import("@playwright/test").Page
): Promise<void> {
  await expect(page.locator("#question-form")).toBeVisible();
  await expect(
    page.locator(
      'input[type="radio"][name="option"], input[type="checkbox"][name="options"]'
    ).first()
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

  await expect(
    page.locator(".option-correct, .option-incorrect").first()
  ).toBeVisible();
}

/** Answer all questions in a session */
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
      await expect(page.locator("#question-form")).toBeVisible();
    }
  }
}

test.describe("authorization - cross-user access denied", () => {
  let quizPublicId: string;
  let sessionId: string;

  // beforeEach does a lot of work (register, create quiz, complete session, logout, register)
  test.setTimeout(60_000);

  test.beforeEach(async ({ page }) => {
    // --- User A: create quiz and complete a session ---
    await registerUser(page);
    const quizName = `AuthzQuiz_${Date.now()}`;
    await createQuiz(page, quizName);
    await page.waitForLoadState("networkidle");

    // Extract quiz public_id from URL (dashboard URL = /quiz/{public_id}/dashboard)
    const dashboardUrl = page.url();
    const quizMatch = dashboardUrl.match(/\/quiz\/([^/]+)\/dashboard/);
    expect(quizMatch).not.toBeNull();
    quizPublicId = quizMatch![1];

    // Start and complete a session
    await page.click("text=Start New Session");
    await expect(page.locator('input[name="name"]')).toBeVisible({ timeout: 15_000 });
    await page.fill('input[name="question_count"]', "5");
    await Promise.all([
      page.waitForResponse((resp) => resp.url().includes("/start-session")),
      page.click('input[type="submit"]'),
    ]);
    await answerAllQuestions(page, 5);

    // Go to results and extract session_id from URL
    await Promise.all([
      page.waitForResponse((resp) => resp.url().includes("/results")),
      page.click(".nav-btn-next"),
    ]);
    await expect(page.locator("h1 mark")).toBeVisible();
    const resultsUrl = page.url();
    const sessionMatch = resultsUrl.match(/\/results\/(\d+)/);
    expect(sessionMatch).not.toBeNull();
    sessionId = sessionMatch![1];

    // --- Logout and register User B ---
    await Promise.all([
      page.waitForResponse((resp) => resp.url().includes("/logout")),
      page.click("text=Log Out"),
    ]);
    await page.waitForURL(/\/(login)?$/);
    await registerUser(page);
  });

  test("cannot access another user's quiz dashboard", async ({
    page,
    jsErrors,
  }) => {
    const resp = await page.goto(`/quiz/${quizPublicId}/dashboard`);
    expect(resp?.status()).toBe(403);
  });

  test("cannot access another user's quiz page", async ({
    page,
    jsErrors,
  }) => {
    const resp = await page.goto(`/quiz/${quizPublicId}`);
    expect(resp?.status()).toBe(403);
  });

  test("cannot access another user's session history", async ({
    page,
    jsErrors,
  }) => {
    const resp = await page.goto(`/quiz/${quizPublicId}/sessions`);
    expect(resp?.status()).toBe(403);
  });

  test("cannot access another user's session results", async ({
    page,
    jsErrors,
  }) => {
    const resp = await page.goto(`/results/${sessionId}`);
    expect(resp?.status()).toBe(403);
  });

  test("cannot delete another user's session", async ({ page, jsErrors }) => {
    const resp = await page.request.delete(`/session/${sessionId}/delete`);
    expect(resp.status()).toBe(403);
  });

  test("cannot rename another user's session", async ({ page, jsErrors }) => {
    const resp = await page.request.patch(`/session/${sessionId}/rename`, {
      form: { name: "hacked" },
    });
    expect(resp.status()).toBe(403);
  });

  test("cannot start session on another user's quiz", async ({
    page,
    jsErrors,
  }) => {
    const resp = await page.request.post(`/start-session/${quizPublicId}`, {
      data: {
        name: "hacked-session",
        question_count: 5,
        selection_mode: "random",
      },
      headers: { "Content-Type": "application/json" },
    });
    expect(resp.status()).toBe(403);
  });
});
