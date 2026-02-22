import { defineConfig } from "@playwright/test";
import { execSync } from "child_process";
import { existsSync, readFileSync, writeFileSync } from "fs";
import { join } from "path";

const PORT_FILE = join(__dirname, ".playwright-port");

function resolvePort(): number {
  // Reuse port written by the main process so workers stay in sync
  if (existsSync(PORT_FILE)) {
    const p = parseInt(readFileSync(PORT_FILE, "utf-8").trim(), 10);
    if (p > 0) return p;
  }
  // Find a free port via the OS
  const p = parseInt(
    execSync(
      `node -e "require('net').createServer().listen(0, function() { process.stdout.write('' + this.address().port); this.close() })"`,
      { encoding: "utf-8" },
    ).trim(),
    10,
  );
  writeFileSync(PORT_FILE, String(p));
  return p;
}

const port = resolvePort();
const baseURL = `http://127.0.0.1:${port}`;

export default defineConfig({
  testDir: "./e2e",
  timeout: 30_000,
  workers: 3,
  retries: 1,
  globalTeardown: join(__dirname, "playwright-global-teardown.ts"),
  use: {
    baseURL,
  },
  webServer: {
    command: "cargo run",
    url: baseURL,
    reuseExistingServer: !process.env.CI,
    timeout: 120_000,
    env: {
      DATABASE_URL:
        process.env.E2E_DATABASE_URL ||
        "postgresql://quizinart:password@localhost:5432/quizinart_e2e",
      ADDRESS: `127.0.0.1:${port}`,
      SECURE_COOKIES: "false",
      RESEND_API_KEY: "",
      BASE_URL: baseURL,
      RUST_LOG: "warn",
    },
  },
  projects: [{ name: "chromium", use: { browserName: "chromium" } }],
});
