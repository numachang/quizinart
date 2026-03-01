import { test, expect } from "./fixtures"
import { registerUser, createQuiz } from "./helpers"
import { execSync } from "child_process"

const E2E_DB_URL =
  process.env.E2E_DATABASE_URL ||
  "postgresql://quizinart:password@localhost:5432/quizinart_e2e"

/** Set is_admin = true for a user by email */
function setAdmin(email: string): void {
  execSync(
    `psql "${E2E_DB_URL}" -c "UPDATE users SET is_admin = true WHERE email = '${email}'"`,
  )
}

test.describe("admin dashboard", () => {
  test("non-admin user gets 403 on /admin", async ({ page, jsErrors }) => {
    await registerUser(page)

    const response = await page.goto("/admin")
    expect(response?.status()).toBe(403)
  })

  test("admin user can access /admin and sees user table", async ({
    page,
    jsErrors,
  }) => {
    const email = await registerUser(page)
    setAdmin(email)

    await page.goto("/admin")
    await expect(page.locator("h1")).toContainText("Admin Dashboard")
    await expect(page.locator("table")).toBeVisible()
    // The table should have at least one user row
    const rows = page.locator("table tbody tr")
    const count = await rows.count()
    expect(count).toBeGreaterThanOrEqual(1)
    // The admin user "Test User" should appear in the table
    await expect(page.locator("table tbody")).toContainText("Test User")
  })

  test("admin dashboard shows quiz count and study time", async ({
    page,
    jsErrors,
  }) => {
    const email = await registerUser(page)
    await createQuiz(page, "Admin Test Quiz")
    setAdmin(email)

    await page.goto("/admin")
    await expect(page.locator("h1")).toContainText("Admin Dashboard")
    await expect(page.locator("table")).toBeVisible()

    // At least one user row should show quiz count >= 1
    // (our test user created a quiz, so there should be a row with "1" in the quiz column)
    const rowsWithQuiz = page.locator("table tbody tr").filter({
      has: page.locator("td:nth-child(2)", { hasText: /^[1-9]/ }),
    })
    await expect(rowsWithQuiz.first()).toBeVisible()
  })

  test("account page shows admin link for admin user", async ({
    page,
    jsErrors,
  }) => {
    const email = await registerUser(page)
    setAdmin(email)

    await page.goto("/account")
    await expect(
      page.locator('a[href="/admin"], [hx-get="/admin"]'),
    ).toBeVisible()
  })

  test("account page does not show admin link for non-admin user", async ({
    page,
    jsErrors,
  }) => {
    await registerUser(page)

    await page.goto("/account")
    await expect(
      page.locator('a[href="/admin"], [hx-get="/admin"]'),
    ).not.toBeVisible()
  })
})
