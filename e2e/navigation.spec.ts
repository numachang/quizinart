import { test, expect } from "./fixtures";

test.describe("unauthenticated navigation", () => {
  test("login page loads without JS errors", async ({ page, jsErrors }) => {
    await page.goto("/");
    await expect(page.locator("h1")).toContainText("Welcome back");
  });

  test("forgot password link works without JS errors", async ({
    page,
    jsErrors,
  }) => {
    await page.goto("/");
    await page.click('a[href="/forgot-password"]');
    await expect(page.locator("h1")).toContainText("Forgot");
  });

  test("register link works without JS errors", async ({
    page,
    jsErrors,
  }) => {
    await page.goto("/");
    await page.click('a[href="/register"]');
    await expect(page.locator("h1")).toContainText("Create your account");
  });

  test("register page loads directly", async ({ page, jsErrors }) => {
    await page.goto("/register");
    await expect(page.locator("h1")).toContainText("Create your account");
  });

  test("forgot password page loads directly", async ({ page, jsErrors }) => {
    await page.goto("/forgot-password");
    await expect(page.locator("h1")).toContainText("Forgot");
  });
});

test.describe("authenticated navigation", () => {
  test.beforeEach(async ({ page }) => {
    // Register a new user (RESEND_API_KEY is empty so email verification is skipped)
    await page.goto("/register");
    await page.fill('input[name="email"]', `test_${Date.now()}@example.com`);
    await page.fill('input[name="display_name"]', "Test User");
    await page.fill('input[name="password"]', "testpass123");
    await page.click('button[type="submit"]');
    // Wait for htmx swap to complete
    await expect(page.locator("h1")).toContainText("Dashboard");
    // Full page reload to get the header with account/logout links
    await page.goto("/");
    await expect(page.locator("h1")).toContainText("Dashboard");
  });

  test("account link works without JS errors", async ({
    page,
    jsErrors,
  }) => {
    await page.click('a[href="/account"]');
    await expect(page.locator("h1")).toContainText("Account");
  });

  test("logout works without JS errors", async ({ page, jsErrors }) => {
    // The logout button uses hx-post with HX-Redirect to /
    await page.click("text=Log Out");
    await expect(page.locator("h1")).toContainText("Welcome back");
  });
});
