import { test, expect } from "./fixtures";
import { registerUser, loginUser } from "./helpers";

test.describe("authentication", () => {
  test("register and login roundtrip", async ({ page, jsErrors }) => {
    const email = await registerUser(page);

    // Logout
    await page.click("text=Log Out");
    await expect(page.locator("h1")).toContainText("Welcome back");

    // Login with the same credentials
    await loginUser(page, email);
    await expect(page.locator("h1")).toContainText("Dashboard");
  });

  test("login with wrong password shows error", async ({
    page,
    jsErrors,
  }) => {
    // Register first
    const email = await registerUser(page);

    // Logout
    await page.click("text=Log Out");
    await expect(page.locator("h1")).toContainText("Welcome back");

    // Try to login with wrong password
    await page.fill('input[name="email"]', email);
    await page.fill('input[name="password"]', "wrongpassword");
    await page.click('button[type="submit"]');

    // Should show error
    await expect(page.locator("small")).toBeVisible();
    await expect(page.locator('input[aria-invalid="true"]')).toBeVisible();
  });

  test("register with duplicate email shows error", async ({
    page,
    jsErrors,
  }) => {
    const email = await registerUser(page);

    // Logout
    await page.click("text=Log Out");
    await expect(page.locator("h1")).toContainText("Welcome back");

    // Try to register again with the same email
    await page.goto("/register");
    await page.fill('input[name="email"]', email);
    await page.fill('input[name="display_name"]', "Another User");
    await page.fill('input[name="password"]', "anotherpass");
    await page.click('button[type="submit"]');

    // Should show error
    await expect(page.locator("small")).toBeVisible();
    await expect(page.locator('input[aria-invalid="true"]')).toBeVisible();
  });

  test("register fields are required (HTML5 validation)", async ({
    page,
    jsErrors,
  }) => {
    await page.goto("/register");

    // All fields have required attribute
    await expect(
      page.locator('input[name="email"][required]')
    ).toBeVisible();
    await expect(
      page.locator('input[name="display_name"][required]')
    ).toBeVisible();
    await expect(
      page.locator('input[name="password"][required]')
    ).toBeVisible();
  });
});
