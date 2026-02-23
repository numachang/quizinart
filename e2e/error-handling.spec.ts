import { test, expect } from "./fixtures";
import { registerUser } from "./helpers";

test.describe("HTMX error handling", () => {
  test.beforeEach(async ({ page }) => {
    await registerUser(page);
  });

  test("shows toast on server error", async ({ page, jsErrors }) => {
    // Intercept the marketplace HTMX request and return 500
    await page.route("**/marketplace", (route) => {
      if (route.request().headers()["hx-request"]) {
        return route.fulfill({ status: 500, body: "Internal Server Error" });
      }
      return route.continue();
    });

    // Click marketplace link (HTMX navigation)
    await page.click('a[href="/marketplace"]');

    // Verify error toast appears
    const toast = page.locator(".toast-error");
    await expect(toast).toBeVisible({ timeout: 3000 });
    await expect(toast).toContainText("Something went wrong");
  });

  test("shows toast on network error", async ({ page, jsErrors }) => {
    // Abort the marketplace HTMX request to simulate network failure
    await page.route("**/marketplace", (route) => {
      if (route.request().headers()["hx-request"]) {
        return route.abort();
      }
      return route.continue();
    });

    // Click marketplace link
    await page.click('a[href="/marketplace"]');

    // Verify network error toast appears
    const toast = page.locator(".toast");
    await expect(toast).toBeVisible({ timeout: 3000 });
    await expect(toast).toContainText("Network error");
  });

  test("toast auto-dismisses", async ({ page, jsErrors }) => {
    // Intercept to trigger an error toast
    await page.route("**/marketplace", (route) => {
      if (route.request().headers()["hx-request"]) {
        return route.fulfill({ status: 500, body: "" });
      }
      return route.continue();
    });

    await page.click('a[href="/marketplace"]');

    const toast = page.locator(".toast-error");
    await expect(toast).toBeVisible({ timeout: 3000 });

    // Toast should disappear after ~5s (give 7s total timeout)
    await expect(toast).not.toBeVisible({ timeout: 7000 });
  });

  test("redirects to login on 401", async ({ page, jsErrors }) => {
    // Intercept the marketplace HTMX request and return 401
    await page.route("**/marketplace", (route) => {
      if (route.request().headers()["hx-request"]) {
        return route.fulfill({ status: 401, body: "Unauthorized" });
      }
      return route.continue();
    });

    // Click marketplace link
    await page.click('a[href="/marketplace"]');

    // Should redirect to login page
    await page.waitForURL("**/login", { timeout: 5000 });
    await expect(page).toHaveURL(/\/login/);
  });
});

test.describe("loading indicator", () => {
  test("progress bar shows during HTMX request", async ({
    page,
    jsErrors,
  }) => {
    await registerUser(page);

    // Delay the marketplace response to observe the progress bar
    await page.route("**/marketplace", async (route) => {
      if (route.request().headers()["hx-request"]) {
        await new Promise((r) => setTimeout(r, 500));
        return route.continue();
      }
      return route.continue();
    });

    // Click marketplace link
    await page.click('a[href="/marketplace"]');

    // Progress bar should be visible during the delayed request
    const bar = page.locator("#htmx-progress.htmx-progress-active");
    await expect(bar).toBeVisible({ timeout: 2000 });

    // Wait for request to complete and page to load
    await expect(page.locator("h1")).toContainText("Marketplace", {
      timeout: 10000,
    });

    // Progress bar should be hidden after completion
    await expect(bar).not.toBeVisible({ timeout: 3000 });
  });
});
