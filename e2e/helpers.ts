import { Page, expect } from "@playwright/test";
import path from "path";

/**
 * Register a new user and navigate to the quiz list page.
 * Returns the generated email address.
 */
export async function registerUser(page: Page): Promise<string> {
  const email = `test_${Date.now()}_${Math.random().toString(36).slice(2, 6)}@example.com`;
  await page.goto("/register");
  await page.fill('input[name="email"]', email);
  await page.fill('input[name="display_name"]', "Test User");
  await page.fill('input[name="password"]', "testpass123");
  await page.click('button[type="submit"]');
  await page.waitForURL((url) => url.pathname === "/", { timeout: 15000 });
  await expect(page.locator("h1")).toContainText("My Quizzes");
  return email;
}

/**
 * Log in with existing credentials.
 */
export async function loginUser(
  page: Page,
  email: string,
  password = "testpass123"
): Promise<void> {
  await page.goto("/login");
  await page.fill('input[name="email"]', email);
  await page.fill('input[name="password"]', password);
  await page.click('button[type="submit"]');
  await page.waitForURL((url) => url.pathname === "/", { timeout: 15000 });
  await expect(page.locator("h1")).toContainText("My Quizzes");
}

/**
 * Create a quiz by uploading the test-quiz.json file.
 * Assumes the quiz list page is already visible.
 */
export async function createQuiz(
  page: Page,
  quizName: string
): Promise<void> {
  await page.fill('input[name="quiz_name"]', quizName);
  const fileInput = page.locator('input[name="quiz_file"]');
  await fileInput.setInputFiles(
    path.join(__dirname, "test-data", "test-quiz.json")
  );
  await Promise.all([
    page.waitForResponse((resp) => resp.url().includes("/create-quiz")),
    page.click('input[type="submit"]'),
  ]);
  // Should navigate to quiz dashboard
  await expect(page.locator("h1")).toContainText(quizName);
}
