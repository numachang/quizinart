import { test, expect } from "./fixtures";
import { registerUser } from "./helpers";

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

test.describe("mobile navigation", () => {
  test.use({ viewport: { width: 375, height: 667 } });

  test("hamburger menu toggles on mobile", async ({ page, jsErrors }) => {
    await page.goto("/");

    // Hamburger should be visible on mobile
    const toggle = page.locator("#nav-toggle");
    await expect(toggle).toBeVisible();

    // Nav menu should be hidden initially
    const menu = page.locator("#nav-menu");
    await expect(menu).not.toHaveClass(/open/);

    // Click hamburger to open
    await toggle.click();
    await expect(menu).toHaveClass(/open/);

    // Click hamburger again to close
    await toggle.click();
    await expect(menu).not.toHaveClass(/open/);
  });

  test("hamburger menu closes on Escape", async ({ page, jsErrors }) => {
    await page.goto("/");
    const toggle = page.locator("#nav-toggle");
    const menu = page.locator("#nav-menu");

    await toggle.click();
    await expect(menu).toHaveClass(/open/);

    await page.keyboard.press("Escape");
    await expect(menu).not.toHaveClass(/open/);
  });

  test("hamburger menu closes on outside click", async ({
    page,
    jsErrors,
  }) => {
    await page.goto("/");
    const toggle = page.locator("#nav-toggle");
    const menu = page.locator("#nav-menu");

    await toggle.click();
    await expect(menu).toHaveClass(/open/);

    // Click on the main content area (outside menu)
    await page.locator("main").click();
    await expect(menu).not.toHaveClass(/open/);
  });
});

test.describe("authenticated navigation", () => {
  test.beforeEach(async ({ page }) => {
    await registerUser(page);
  });

  test("logo navigates to home", async ({ page, jsErrors }) => {
    // Navigate away first
    await page.click('a[href="/account"]');
    await expect(page.locator("h1")).toContainText("Account");

    // Click logo to go back
    const logo = page.locator('nav a[href="/"]', { hasText: "Quizinart" });
    await expect(logo).toBeVisible();
    await logo.click();
    await expect(page.locator("h1")).toContainText("My Quizzes");
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
    await Promise.all([
      page.waitForResponse((resp) => resp.url().includes("/logout")),
      page.click("text=Log Out"),
    ]);
    await expect(page.locator("h1")).toContainText("Welcome back");
  });
});

test.describe("mobile authenticated navigation", () => {
  test.use({ viewport: { width: 375, height: 667 } });

  test.beforeEach(async ({ page }) => {
    await registerUser(page);
  });

  test("account link works via hamburger menu", async ({
    page,
    jsErrors,
  }) => {
    const toggle = page.locator("#nav-toggle");
    await toggle.click();

    await page.click('a[href="/account"]');
    await expect(page.locator("h1")).toContainText("Account");
  });

  test("logout works via hamburger menu", async ({ page, jsErrors }) => {
    const toggle = page.locator("#nav-toggle");
    await toggle.click();

    await Promise.all([
      page.waitForResponse((resp) => resp.url().includes("/logout")),
      page.click("text=Log Out"),
    ]);
    await expect(page.locator("h1")).toContainText("Welcome back");
  });

  test("marketplace link works via hamburger menu", async ({
    page,
    jsErrors,
  }) => {
    const toggle = page.locator("#nav-toggle");
    await toggle.click();

    const marketplaceLink = page.locator('#nav-menu a[href="/marketplace"]');
    await expect(marketplaceLink).toBeVisible();
    await marketplaceLink.click();
    await expect(page.locator("h1")).toContainText("Marketplace");
  });
});
