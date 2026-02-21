import { test, expect } from "./fixtures";
import { registerUser, loginUser } from "./helpers";

test.describe("account management", () => {
  test.beforeEach(async ({ page }) => {
    await registerUser(page);
  });

  test("account page shows user info", async ({ page, jsErrors }) => {
    await page.click('a[href="/account"]');
    await expect(page.locator("h1")).toContainText("Account");

    // Email and display name inputs should be visible and disabled
    const emailInput = page.locator('input[type="email"][disabled]');
    await expect(emailInput).toBeVisible();
    await expect(emailInput).toHaveValue(/test_.*@example\.com/);

    const nameInput = page.locator('input[type="text"][disabled]');
    await expect(nameInput).toBeVisible();
    await expect(nameInput).toHaveValue("Test User");
  });

  test("change password with wrong current password shows error", async ({
    page,
    jsErrors,
  }) => {
    await page.click('a[href="/account"]');
    await expect(page.locator("h1")).toContainText("Account");

    // Fill wrong current password
    await page.fill('input[name="current_password"]', "wrongpassword");
    await page.fill('input[name="new_password"]', "newpass123");
    await Promise.all([
      page.waitForResponse((resp) => resp.url().includes("/change-password")),
      page.click('button[type="submit"]'),
    ]);

    // Should show error
    await expect(page.locator('input[aria-invalid="true"]')).toBeVisible();
    await expect(page.locator("small")).toBeVisible();
  });

  test("change password successfully", async ({ page, jsErrors }) => {
    const email = await page
      .locator('a[href="/account"]')
      .evaluate(() => ""); // We need the email from registration
    // Re-get email from the account page
    await page.click('a[href="/account"]');
    await expect(page.locator("h1")).toContainText("Account");

    const userEmail = await page
      .locator('input[type="email"][disabled]')
      .inputValue();

    // Change password
    await page.fill('input[name="current_password"]', "testpass123");
    await page.fill('input[name="new_password"]', "newpass456");
    await Promise.all([
      page.waitForResponse((resp) => resp.url().includes("/change-password")),
      page.click('button[type="submit"]'),
    ]);

    // Should show success message
    await expect(
      page.locator("text=password has been changed successfully")
    ).toBeVisible();

    // Logout and login with new password
    await page.goto("/");
    await Promise.all([
      page.waitForResponse((resp) => resp.url().includes("/logout")),
      page.click("text=Log Out"),
    ]);
    await expect(page.locator("h1")).toContainText("Welcome back");

    await loginUser(page, userEmail, "newpass456");
    await expect(page.locator("h1")).toContainText("Dashboard");
  });
});
