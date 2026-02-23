# Developer Guide

[日本語](developer-guide_ja.md)

> **Prerequisite:** Complete [Development Environment Setup](setup.md) first.

---

## Table of Contents

1. [Daily Development Cycle](#1-daily-development-cycle)
2. [sqlx Compile-Time Verification](#2-sqlx-compile-time-verification)
3. [Adding DB Migrations](#3-adding-db-migrations)
4. [Writing E2E Tests](#4-writing-e2e-tests)
5. [CI Pipeline and Failure Patterns](#5-ci-pipeline-and-failure-patterns)
6. [Architecture Overview](#6-architecture-overview)
7. [Troubleshooting](#7-troubleshooting)

---

## 1. Daily Development Cycle

### Starting a session

```bash
cd quizinart/
docker compose up -d   # Start local PostgreSQL
cargo run              # Migrations run automatically on startup
# Open http://127.0.0.1:1414
```

### Typical workflow

```
1. Write/change code
2. cargo fmt --all                           # Fix formatting
3. cargo clippy --all-targets --all-features # Fix warnings
4. cargo sqlx prepare                        # If SQL queries changed
5. npx playwright test                       # Run E2E tests
6. git push                                  # Pre-push hook validates everything
```

### Pre-push hook

Installed automatically by `npm install` (the `prepare` script sets `core.hooksPath`). The hook (`.githooks/pre-push`) runs in order:

1. `cargo fmt --all -- --check`
2. `cargo clippy --all-targets --all-features -- -D warnings`
3. `cargo sqlx prepare --check`
4. `npx biome ci static/`
5. `npx playwright test`

**PostgreSQL must be running** for step 3 and 5 to pass.

### When PostgreSQL is NOT needed: `SQLX_OFFLINE=true`

sqlx query macros (`query!`, `query_as!`, `query_scalar!`) connect to PostgreSQL at compile time to verify queries. The `.sqlx/` directory caches these results, enabling compilation without a database:

```bash
SQLX_OFFLINE=true cargo build
SQLX_OFFLINE=true cargo clippy --all-targets --all-features
```

This is used in the Dockerfile and is useful on machines without Docker. It is **not** used during normal development because `cargo sqlx prepare --check` in the pre-push hook needs an up-to-date cache.

---

## 2. sqlx Compile-Time Verification

### How `.sqlx/` works

sqlx checks SQL query syntax and types at compile time by running queries against a real PostgreSQL database. The results are cached as JSON files in `.sqlx/`, keyed by the SHA-256 hash of each query.

When `SQLX_OFFLINE=true`, the compiler reads these cache files instead of connecting to the database. **The `.sqlx/` directory must be committed to Git.**

### Workflow for query changes

1. Ensure Docker PostgreSQL is running and migrations are applied (run `cargo run` once)
2. Write the new/changed `query!` / `query_as!` / `query_scalar!` macro call
3. Regenerate the cache:
   ```bash
   cargo sqlx prepare
   ```
4. Verify the cache:
   ```bash
   cargo sqlx prepare --check
   ```
5. **Commit the updated `.sqlx/` files alongside your Rust code changes**

### The three query macros

| Macro | Returns | Typical use |
|-------|---------|-------------|
| `query!` | `QueryResult` (rows affected) | INSERT / UPDATE / DELETE |
| `query_scalar!` | Single typed value | COUNT, EXISTS, single-column SELECT |
| `query_as!` | Struct mapped from columns | Multi-column SELECT |

### Nullable column handling: the `!` suffix

PostgreSQL columns that are technically nullable (e.g. from JOINs, aggregates) produce `Option<T>` by default, even when logic guarantees a value. Use the `"column!"` alias annotation to force `T`:

```rust
// COUNT always returns a value — force non-optional with "!"
let count: i32 = sqlx::query_scalar!(
    r#"SELECT COUNT(*)::INT AS "count!" FROM quiz_sessions WHERE quiz_id = $1"#,
    quiz_id
)
.fetch_one(&self.pool)
.await?;
```

```rust
// Column filtered by WHERE ... IS NOT NULL — safe to force
let categories: Vec<String> = sqlx::query_scalar!(
    r#"SELECT DISTINCT category AS "category!"
       FROM questions
       WHERE quiz_id = $1 AND category IS NOT NULL
       ORDER BY category"#,
    quiz_id
)
.fetch_all(&self.pool)
.await?;
```

A column that is genuinely optional should have **no** annotation, resulting in `Option<T>` in the model struct.

When using `query_as!`, the model struct field types must match what sqlx infers. If sqlx infers `Option<T>` but the struct has `T`, compilation fails — use the `!` alias in the SQL to fix it.

---

## 3. Adding DB Migrations

This project uses [sqlx's built-in migration system](https://docs.rs/sqlx/latest/sqlx/macro.migrate.html). Migrations are embedded at compile time via the `sqlx::migrate!()` macro and tracked in the `_sqlx_migrations` table.

### Step-by-step

**1. Generate a new migration file**

```bash
sqlx migrate add <description>
```

This creates a timestamped file like `migrations/20260223120000_add_feature_flags.sql`.

Always use **additive** migrations (`ADD COLUMN`, `CREATE TABLE IF NOT EXISTS`, `CREATE INDEX IF NOT EXISTS`). For destructive changes, write a data migration first.

**2. Write the SQL**

```sql
-- migrations/20260223120000_add_feature_flags.sql
CREATE TABLE IF NOT EXISTS feature_flags (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    enabled BOOLEAN NOT NULL DEFAULT FALSE
);
```

**3. Apply and update the sqlx cache**

```bash
# Apply the migration (auto-runs on startup)
cargo run

# Regenerate the sqlx offline cache
cargo sqlx prepare

# Commit everything together
git add migrations/ .sqlx/
```

### How CI applies migrations

CI uses `sqlx migrate run` (via sqlx-cli) before checking the offline cache. Both CI and the Rust app use the same sqlx migration system.

---

## 4. Writing E2E Tests

### File structure

```
e2e/
├── fixtures.ts          — Custom test base with JS error detection
├── helpers.ts           — Shared flows: registerUser, loginUser, createQuiz
├── auth.spec.ts         — Registration, login, validation
├── account.spec.ts      — Account settings, password change
├── navigation.spec.ts   — HTMX navigation, URL updates
├── quiz-lifecycle.spec.ts — Quiz creation, rename, delete
├── quiz-session.spec.ts — Session start, answer, bookmark, results
└── test-data/
    └── test-quiz.json   — Standard test quiz uploaded by createQuiz()
```

### Import from fixtures, not @playwright/test

```typescript
// CORRECT — includes jsErrors fixture
import { test, expect } from "./fixtures";

// WRONG — skips JS error monitoring
import { test, expect } from "@playwright/test";
```

### Always declare `jsErrors` in the test signature

Include `jsErrors` in the destructured parameter to activate the fixture, even if you don't use it directly:

```typescript
test("my test", async ({ page, jsErrors }) => {
    // jsErrors fixture is active — any JS error fails the test
});
```

### Helper functions

```typescript
// Register a new unique user. Returns the generated email.
const email = await registerUser(page);

// Log in with existing credentials (default password: "testpass123")
await loginUser(page, email);

// Create a quiz by uploading test-data/test-quiz.json
await createQuiz(page, "My Quiz Name");
```

### Selector rules

Prefer robust, semantic selectors:

| Preferred | Avoid |
|-----------|-------|
| `input[name="email"]` | `.form > div:nth-child(2) > input` |
| `page.locator("article h3", { hasText: name })` | `article.quiz-card h3` |
| `page.getByTitle("Rename")` | `button.rename-btn` |
| `page.locator("#create-card")` | `page.locator(".card:last-child")` |

Scope interactions to a specific card:

```typescript
const card = page.locator("article", {
  has: page.locator("h3", { hasText: quizName }),
});
await card.getByTitle("Delete").click();
```

### Waiting for HTMX responses

HTMX swaps happen after a network response. Always wait before asserting:

```typescript
await Promise.all([
  page.waitForResponse((resp) => resp.url().includes("/start-session")),
  page.click('input[type="submit"]'),
]);
```

### Running tests

```bash
cd quizinart/

# Run all tests
npx playwright test

# Run a specific file
npx playwright test e2e/auth.spec.ts

# Interactive UI mode
npx playwright test --ui

# Show report after run
npx playwright show-report
```

The E2E database (`quizinart_e2e`) is separate from the development database (`quizinart`).

### TDD flow

1. **Red:** Write the E2E test first — it should fail
2. **Green:** Implement the minimum code to pass
3. **Refactor:** Clean up while keeping tests green

When a test fails, check the implementation first before modifying the test. Test modifications are valid only for specification changes.

---

## 5. CI Pipeline and Failure Patterns

### Pipeline overview (`.github/workflows/ci.yml`)

Triggered on all pull requests and pushes to `main`. Runs on `ubuntu-latest` with PostgreSQL 16.

| Step | Command |
|------|---------|
| Biome lint | `npx biome ci static/` |
| Format check | `cargo fmt --all -- --check` |
| DB migrations | `sqlx migrate run` (via sqlx-cli) |
| sqlx cache check | `cargo sqlx prepare --check` |
| Clippy | `cargo clippy --all-targets --all-features` |
| Unit tests | `cargo test --all-targets --all-features` |
| E2E tests | `npx playwright test` |

### Common failure patterns

| Failure | Fix |
|---------|-----|
| `query file .sqlx/... is not up to date` | Run `cargo sqlx prepare` locally, commit `.sqlx/` |
| `Diff in src/...` (fmt check) | Run `cargo fmt --all`, commit |
| Clippy warning (e.g. `unused variable`) | Fix the warning. CI uses `-D warnings` |
| Biome lint errors | Run `npx biome check --write static/`, commit |
| E2E timeout on HTMX interaction | Add `await page.waitForResponse(...)` around the click |
| E2E `net::ERR_CONNECTION_REFUSED` | The server failed to start — run `cargo run` locally to debug |

### Other workflows

- **`.github/workflows/docker.yml`** — Builds Docker image (`SQLX_OFFLINE=true`), pushes to GHCR, triggers Render.com deploy
- **`.github/workflows/release.yml`** — cargo-dist binary releases for macOS, Linux, Windows

---

## 6. Architecture Overview

### Layer structure

```
src/
├── handlers/        — HTTP layer: route handlers
│   ├── homepage.rs  — Auth routes + quiz CRUD
│   ├── account.rs   — Account management
│   └── quiz/        — Quiz session, questions, dashboard
├── db/              — Data layer: all SQL queries
│   ├── migrations.rs — sqlx standard migration runner (sqlx::migrate!)
│   ├── models.rs     — Return types (plain structs)
│   ├── quiz.rs       — Quiz CRUD
│   ├── session.rs    — Session lifecycle
│   ├── question.rs   — Question fetching, stats
│   ├── user.rs       — Auth, sessions, email verification
│   ├── answer.rs     — Answer submission + scoring
│   └── report.rs     — Statistics queries
├── views/           — HTML rendering (Maud)
│   ├── layout.rs    — Document structure, nav, CSS/JS
│   ├── components.rs — Reusable HTMX nav_link
│   ├── homepage.rs  — Register/login/quiz-list
│   ├── account.rs   — Account settings
│   └── quiz/        — Dashboard, question, session views
├── extractors.rs    — Custom Axum extractors (AuthGuard, IsHtmx, Locale)
├── rejections.rs    — AppError + ResultExt
├── names.rs         — URL & cookie name constants
├── email.rs         — Resend API (email verification, password reset)
└── statics.rs       — Embedded static file serving (include_dir!)
```

### AppState

All database access goes through `db::Db` (wraps `sqlx::PgPool`), stored in `AppState`:

```rust
#[derive(Clone)]
pub struct AppState {
    pub db: db::Db,
    pub secure_cookies: bool,
    pub resend_api_key: String,
    pub base_url: String,
}
```

Handlers call `state.db.*` methods directly (no service layer yet).

### Auth system

**`AuthGuard`** — Extracts the authenticated user or returns 401. Falls back to a legacy `admin_session` cookie for backward compatibility.

```rust
async fn create_quiz(
    AuthGuard(user): AuthGuard,  // user: AuthUser { id, email, display_name }
    State(state): State<AppState>,
    ...
)
```

**Session mechanism:** ULID tokens stored in the `user_sessions` table. Session ID is in an `HttpOnly; SameSite=Lax; Max-Age=86400` cookie (`user_session`). A middleware refreshes the Max-Age on every response (sliding expiration).

**CSRF protection:** A middleware rejects state-changing requests (`POST/PUT/PATCH/DELETE`) that lack either `HX-Request: true` or a matching `Origin`/`Host` pair.

**Password hashing:** argon2 via the `argon2` crate. Runs in a dedicated 4 MB stack thread to avoid stack overflow in debug builds.

### HTMX navigation pattern

All navigation links use the same pattern:

```rust
// views/components.rs
pub fn nav_link(href: &str, body: Markup) -> Markup {
    html! {
        a href=(href)
          hx-get=(href)
          hx-target="main"
          hx-push-url="true"
          hx-swap="innerHTML" { (body) }
    }
}
```

Handlers check `IsHtmx` to decide the response:

- **HTMX request** → Returns `<title>` + inner HTML fragment only (`views::titled()`)
- **Direct browser navigation** → Returns full HTML document (`views::page_with_user()`)

The `views::render()` function handles this automatically:

```rust
pub fn render(is_htmx: bool, title: &str, body: Markup, locale: &str, user_name: Option<&str>) -> Markup {
    if is_htmx { titled(title, body) }
    else { page_with_user(title, body, locale, user_name) }
}
```

### `hx-ext="json-enc"`

Some forms use the `json-enc` extension to send `application/json` instead of form-encoded data. Handlers for these use `Json<T>` extractor. A custom `deserialize_string_or_i32` serde deserializer handles cases where `json-enc` sends numbers as strings.

### i18n (rust-i18n)

Locale files: `locales/en.yml`, `locales/ja.yml`, `locales/zh-CN.yml`, `locales/zh-TW.yml`. The `Locale` extractor reads the `lang` cookie (defaults to `"en"`) and passes it through to view functions:

```rust
html! {
    h1 { (t!("homepage.my_quizzes", locale = locale)) }
}
```

### Error handling

`AppError` in `rejections.rs` has three variants:

| Variant | Status | Usage |
|---------|--------|-------|
| `Internal(&'static str)` | 500 | Server errors (logs the actual error) |
| `Unauthorized` | 401 | Redirects to login |
| `Input(&'static str)` | 400 | Bad user input |

The `ResultExt` trait provides ergonomic conversion:

```rust
let quizzes = state.db.quizzes(user.id)
    .await
    .reject("could not get quizzes")?;       // → AppError::Internal

let questions = parse_json(input)
    .reject_input("failed to decode quiz")?;  // → AppError::Input
```

`reject()` logs the underlying error at `tracing::error!` level; only a generic message reaches the client.

### Static files

Embedded in the binary at compile time via `include_dir!("static")`. Served at `/static/{path}` with `Cache-Control: max-age=3600, must-revalidate`.

---

## 7. Troubleshooting

| Symptom | Cause | Fix |
|---------|-------|-----|
| `connection refused` on startup | PostgreSQL not running | `docker compose up -d` |
| Port 5432 already in use | Another PostgreSQL process | `sudo service postgresql stop`, then `docker compose up -d` |
| `STATUS_STACK_OVERFLOW` (Windows) | argon2 stack usage in debug | `cargo clean && cargo build` |
| Build not reflecting changes (Windows) | exe locked | `taskkill /IM quizinart.exe /F` → `cargo clean && cargo build` |
| `docker: permission denied` (Linux) | User not in docker group | `sudo usermod -aG docker $USER`, then re-login |
| `sqlx prepare --check` fails: "not up to date" | `.sqlx/` stale after SQL change | `cargo sqlx prepare` → commit `.sqlx/` |
| `sqlx prepare --check` fails: "connecting to database" | Missing `.env` or DB is down | `cp .env.example .env` → `docker compose up -d` |
| E2E `net::ERR_CONNECTION_REFUSED` | Server didn't start | Run `cargo run` manually first to verify |
| E2E timeout on HTMX interactions | Missing wait for response | Wrap clicks with `Promise.all([page.waitForResponse(...), page.click(...)])` |
| Migration error: "column already exists" | Non-idempotent migration SQL | Use `ADD COLUMN IF NOT EXISTS` / `CREATE TABLE IF NOT EXISTS` |
| `git push` rejected by pre-push hook | One of the 5 checks failed | Run each check individually (see [Pre-push hook](#pre-push-hook)) |
