import { defineConfig } from "@playwright/test";

export default defineConfig({
  testDir: "./e2e",
  timeout: 30_000,
  workers: 3,
  retries: 1,
  use: {
    baseURL: "http://127.0.0.1:1414",
  },
  webServer: {
    command: "cargo run",
    url: "http://127.0.0.1:1414",
    reuseExistingServer: !process.env.CI,
    timeout: 120_000,
    env: {
      DATABASE_URL:
        "postgresql://quizinart:password@localhost:5432/quizinart_e2e",
      ADDRESS: "127.0.0.1:1414",
      SECURE_COOKIES: "false",
      RESEND_API_KEY: "",
      BASE_URL: "http://127.0.0.1:1414",
      RUST_LOG: "warn",
    },
  },
  projects: [{ name: "chromium", use: { browserName: "chromium" } }],
});
