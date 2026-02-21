import { Page, expect } from "@playwright/test";
import path from "path";

/**
 * Register a new user and navigate to the dashboard.
 * Returns the generated email address.
 */
export async function registerUser(page: Page): Promise<string> {
  const email = `test_${Date.now()}_${Math.random().toString(36).slice(2, 6)}@example.com`;
  await page.goto("/register");
  // Wait for HTMX to initialize (hx-post attribute gets processed)
  await page.waitForLoadState("networkidle");
  await page.fill('input[name="email"]', email);
  await page.fill('input[name="display_name"]', "Test User");
  await page.fill('input[name="password"]', "testpass123");
  await Promise.all([
    page.waitForResponse(
      (resp) => resp.url().includes("/register") && resp.request().method() === "POST"
    ),
    page.click('button[type="submit"]'),
  ]);
  await expect(page.locator("h1")).toContainText("Dashboard");
  // Full page reload to get header with account/logout links
  await page.goto("/");
  await expect(page.locator("h1")).toContainText("Dashboard");
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
  await page.goto("/");
  await page.waitForLoadState("networkidle");
  await page.fill('input[name="email"]', email);
  await page.fill('input[name="password"]', password);
  await Promise.all([
    page.waitForResponse((resp) => resp.url().includes("/login")),
    page.click('button[type="submit"]'),
  ]);
  await expect(page.locator("h1")).toContainText("Dashboard");
}

/**
 * Create a quiz by uploading the test-quiz.json file.
 * Assumes the dashboard is already visible.
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
