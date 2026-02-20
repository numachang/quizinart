import { test as base, expect } from "@playwright/test";

// All tests automatically monitor for JS errors via pageerror
export const test = base.extend<{ jsErrors: string[] }>({
  jsErrors: async ({ page }, use) => {
    const errors: string[] = [];
    page.on("pageerror", (err) => errors.push(err.message));
    await use(errors);
    expect(errors, "JavaScript errors detected").toHaveLength(0);
  },
});

export { expect };
