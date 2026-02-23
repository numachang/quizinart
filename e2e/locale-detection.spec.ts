import { test, expect } from "./fixtures";

test.describe("Accept-Language auto-detection", () => {
  test("defaults to Japanese when Accept-Language prefers ja", async ({
    browser,
  }) => {
    const context = await browser.newContext({ locale: "ja" });
    const page = await context.newPage();
    const errors: string[] = [];
    page.on("pageerror", (err) => errors.push(err.message));

    await page.goto("/");
    await expect(page.locator("h1")).toContainText(
      "一問一答で、あらゆる科目をマスター",
    );

    await context.close();
    expect(errors, "JavaScript errors detected").toHaveLength(0);
  });

  test("defaults to zh-CN when Accept-Language prefers zh-CN", async ({
    browser,
  }) => {
    const context = await browser.newContext({ locale: "zh-CN" });
    const page = await context.newPage();
    const errors: string[] = [];
    page.on("pageerror", (err) => errors.push(err.message));

    await page.goto("/");
    await expect(page.locator("h1")).toContainText("一题一练，掌握任何科目");

    await context.close();
    expect(errors, "JavaScript errors detected").toHaveLength(0);
  });

  test("defaults to zh-TW when Accept-Language prefers zh-TW", async ({
    browser,
  }) => {
    const context = await browser.newContext({ locale: "zh-TW" });
    const page = await context.newPage();
    const errors: string[] = [];
    page.on("pageerror", (err) => errors.push(err.message));

    await page.goto("/");
    await expect(page.locator("h1")).toContainText("一題一練，掌握任何科目");

    await context.close();
    expect(errors, "JavaScript errors detected").toHaveLength(0);
  });

  test("defaults to English for unsupported Accept-Language", async ({
    browser,
  }) => {
    const context = await browser.newContext({ locale: "fr" });
    const page = await context.newPage();
    const errors: string[] = [];
    page.on("pageerror", (err) => errors.push(err.message));

    await page.goto("/");
    await expect(page.locator("h1")).toContainText(
      "Master Any Subject, One Quiz at a Time",
    );

    await context.close();
    expect(errors, "JavaScript errors detected").toHaveLength(0);
  });

  test("lang cookie takes priority over Accept-Language", async ({
    browser,
    baseURL,
  }) => {
    const context = await browser.newContext({ locale: "ja" });
    const page = await context.newPage();
    const errors: string[] = [];
    page.on("pageerror", (err) => errors.push(err.message));

    // Set the lang cookie to English explicitly
    await context.addCookies([
      { name: "lang", value: "en", url: baseURL! },
    ]);

    await page.goto("/");
    // Should show English despite Accept-Language preferring Japanese
    await expect(page.locator("h1")).toContainText(
      "Master Any Subject, One Quiz at a Time",
    );

    await context.close();
    expect(errors, "JavaScript errors detected").toHaveLength(0);
  });

  test("selects highest-priority supported language from Accept-Language", async ({
    browser,
  }) => {
    // Setting locale to "ja-JP" makes the browser send "ja-JP,ja;q=0.9"
    // which should match "ja" as the best supported locale
    const context = await browser.newContext({ locale: "ja-JP" });
    const page = await context.newPage();
    const errors: string[] = [];
    page.on("pageerror", (err) => errors.push(err.message));

    await page.goto("/");
    await expect(page.locator("h1")).toContainText(
      "一問一答で、あらゆる科目をマスター",
    );

    await context.close();
    expect(errors, "JavaScript errors detected").toHaveLength(0);
  });
});
