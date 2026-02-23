# 開発者ガイド

[English](developer-guide.md)

> **前提条件:** 先に[開発環境構築手順](setup_ja.md)を完了してください。

---

## 目次

1. [日常の開発サイクル](#1-日常の開発サイクル)
2. [sqlx コンパイル時検証](#2-sqlx-コンパイル時検証)
3. [DB マイグレーション追加手順](#3-db-マイグレーション追加手順)
4. [E2E テストの書き方](#4-e2e-テストの書き方)
5. [CI パイプラインと失敗パターン](#5-ci-パイプラインと失敗パターン)
6. [アーキテクチャ概要](#6-アーキテクチャ概要)
7. [トラブルシューティング](#7-トラブルシューティング)

---

## 1. 日常の開発サイクル

### セッションの開始

```bash
cd quizinart/
docker compose up -d   # ローカル PostgreSQL を起動
cargo run              # 起動時にマイグレーションが自動実行される
# http://127.0.0.1:1414 を開く
```

### 典型的なワークフロー

```
1. コードを書く/変更する
2. cargo fmt --all                           # フォーマット修正
3. cargo clippy --all-targets --all-features # 警告修正
4. cargo sqlx prepare                        # SQL クエリを変更した場合
5. npx playwright test                       # E2E テスト実行
6. git push                                  # pre-push hook が全チェックを実行
```

### Pre-push hook

`npm install` で自動インストールされる（`prepare` スクリプトが `core.hooksPath` を設定）。hook（`.githooks/pre-push`）は以下を順に実行：

1. `cargo fmt --all -- --check`
2. `cargo clippy --all-targets --all-features -- -D warnings`
3. `cargo sqlx prepare --check`
4. `npx biome ci static/`
5. `npx playwright test`

ステップ 3 と 5 のために **PostgreSQL が起動している必要がある**。

### PostgreSQL が不要な場合：`SQLX_OFFLINE=true`

sqlx のクエリマクロ（`query!`、`query_as!`、`query_scalar!`）はコンパイル時に PostgreSQL に接続してクエリを検証する。`.sqlx/` ディレクトリにその結果がキャッシュされ、データベースなしでのコンパイルが可能になる：

```bash
SQLX_OFFLINE=true cargo build
SQLX_OFFLINE=true cargo clippy --all-targets --all-features
```

これは Dockerfile や Docker を使えない環境で利用される。通常の開発では pre-push hook の `cargo sqlx prepare --check` に最新のキャッシュが必要なため、**使用しない**。

---

## 2. sqlx コンパイル時検証

### `.sqlx/` の仕組み

sqlx はコンパイル時に SQL クエリの構文と型を実際の PostgreSQL データベースに対して検証する。その結果は `.sqlx/` 内に JSON ファイルとしてキャッシュされ、各クエリの SHA-256 ハッシュがキーになる。

`SQLX_OFFLINE=true` の場合、コンパイラはデータベースの代わりにこれらのキャッシュファイルを読む。**`.sqlx/` ディレクトリは Git にコミットする必要がある。**

### クエリ変更時のワークフロー

1. Docker PostgreSQL が起動し、マイグレーションが適用されていることを確認（`cargo run` を一度実行）
2. `query!` / `query_as!` / `query_scalar!` マクロの呼び出しを新規作成/変更
3. キャッシュを再生成：
   ```bash
   cargo sqlx prepare
   ```
4. キャッシュの整合性を確認：
   ```bash
   cargo sqlx prepare --check
   ```
5. **更新された `.sqlx/` ファイルを Rust コードと一緒にコミット**

### 3つのクエリマクロ

| マクロ | 戻り値 | 主な用途 |
|--------|--------|----------|
| `query!` | `QueryResult`（影響行数） | INSERT / UPDATE / DELETE |
| `query_scalar!` | 単一の型付き値 | COUNT、EXISTS、単一列 SELECT |
| `query_as!` | カラムからマッピングされた構造体 | 複数列の SELECT |

### Nullable カラムの扱い：`!` サフィックス

PostgreSQL で技術的に nullable なカラム（JOIN や集約関数由来）はデフォルトで `Option<T>` になる。ロジック上値が保証されている場合、`"column!"` エイリアスで `T` を強制できる：

```rust
// COUNT は常に値を返す — "!" で非 Optional に強制
let count: i32 = sqlx::query_scalar!(
    r#"SELECT COUNT(*)::INT AS "count!" FROM quiz_sessions WHERE quiz_id = $1"#,
    quiz_id
)
.fetch_one(&self.pool)
.await?;
```

```rust
// WHERE ... IS NOT NULL でフィルタ済み — 安全に強制可能
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

本当にオプショナルなカラムにはアノテーションを付けず、モデル構造体で `Option<T>` とする。

`query_as!` 使用時は、モデル構造体のフィールド型が sqlx の推論と一致する必要がある。sqlx が `Option<T>` を推論するが構造体が `T` の場合、コンパイルエラーになる — SQL 内の `!` エイリアスで修正する。

---

## 3. DB マイグレーション追加手順

このプロジェクトは [sqlx の標準マイグレーションシステム](https://docs.rs/sqlx/latest/sqlx/macro.migrate.html)を使用する。マイグレーションは `sqlx::migrate!()` マクロでコンパイル時に埋め込まれ、`_sqlx_migrations` テーブルで管理される。

### 手順

**1. マイグレーションファイルを生成**

```bash
sqlx migrate add <説明>
```

`migrations/20260223120000_add_feature_flags.sql` のようなタイムスタンプ付きファイルが生成される。

常に**追加的**マイグレーション（`ADD COLUMN`、`CREATE TABLE IF NOT EXISTS`、`CREATE INDEX IF NOT EXISTS`）を使う。破壊的変更が必要な場合は、先にデータ移行を行う。

**2. SQL を記述**

```sql
-- migrations/20260223120000_add_feature_flags.sql
CREATE TABLE IF NOT EXISTS feature_flags (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    enabled BOOLEAN NOT NULL DEFAULT FALSE
);
```

**3. 適用と sqlx キャッシュ更新**

```bash
# マイグレーション適用（起動時に自動実行）
cargo run

# sqlx オフラインキャッシュを再生成
cargo sqlx prepare

# まとめてコミット
git add migrations/ .sqlx/
```

### CI でのマイグレーション

CI はオフラインキャッシュのチェック前に `sqlx migrate run`（sqlx-cli 経由）でマイグレーションを適用する。CI と Rust アプリは同じ sqlx マイグレーションシステムを使用する。

---

## 4. E2E テストの書き方

### ファイル構成

```
e2e/
├── fixtures.ts          — JS エラー検知付きカスタム test ベース
├── helpers.ts           — 共通フロー: registerUser, loginUser, createQuiz
├── auth.spec.ts         — 登録、ログイン、バリデーション
├── account.spec.ts      — アカウント設定、パスワード変更
├── navigation.spec.ts   — HTMX ナビゲーション、URL 更新
├── quiz-lifecycle.spec.ts — クイズ作成、名前変更、削除
├── quiz-session.spec.ts — セッション開始、回答、ブックマーク、結果
└── test-data/
    └── test-quiz.json   — createQuiz() がアップロードするテストクイズ
```

### @playwright/test ではなく fixtures からインポート

```typescript
// 正しい — jsErrors fixture が含まれる
import { test, expect } from "./fixtures";

// 間違い — JS エラー監視がスキップされる
import { test, expect } from "@playwright/test";
```

### テストシグネチャに必ず `jsErrors` を宣言

直接使わなくても、fixture を有効化するためにデストラクチャリング引数に含める：

```typescript
test("my test", async ({ page, jsErrors }) => {
    // jsErrors fixture がアクティブ — ページ上の JS エラーでテストが失敗する
});
```

### ヘルパー関数

```typescript
// 新しいユニークユーザーを登録。生成されたメールアドレスを返す。
const email = await registerUser(page);

// 既存の認証情報でログイン（デフォルトパスワード: "testpass123"）
await loginUser(page, email);

// test-data/test-quiz.json をアップロードしてクイズを作成
await createQuiz(page, "My Quiz Name");
```

### セレクタのルール

堅牢でセマンティックなセレクタを使う：

| 推奨 | 避ける |
|------|--------|
| `input[name="email"]` | `.form > div:nth-child(2) > input` |
| `page.locator("article h3", { hasText: name })` | `article.quiz-card h3` |
| `page.getByTitle("Rename")` | `button.rename-btn` |
| `page.locator("#create-card")` | `page.locator(".card:last-child")` |

特定のカードにスコープする：

```typescript
const card = page.locator("article", {
  has: page.locator("h3", { hasText: quizName }),
});
await card.getByTitle("Delete").click();
```

### HTMX レスポンスの待機

HTMX のスワップはネットワークレスポンス後に発生する。アサーション前に必ず待機する：

```typescript
await Promise.all([
  page.waitForResponse((resp) => resp.url().includes("/start-session")),
  page.click('input[type="submit"]'),
]);
```

### テストの実行

```bash
cd quizinart/

# 全テスト実行
npx playwright test

# 特定ファイルのみ実行
npx playwright test e2e/auth.spec.ts

# インタラクティブ UI モード
npx playwright test --ui

# 実行後レポート表示
npx playwright show-report
```

E2E データベース（`quizinart_e2e`）は開発用データベース（`quizinart`）とは別になっている。

### TDD フロー

1. **Red:** E2E テストを先に書く — 失敗するはず
2. **Green:** テストを通す最小限のコードを実装
3. **Refactor:** テストが通る状態を維持しつつ整理

テスト失敗時は、テストを修正する前にまず実装側に問題がないか確認する。テストの修正が正当なのは仕様変更の場合のみ。

---

## 5. CI パイプラインと失敗パターン

### パイプライン概要（`.github/workflows/ci.yml`）

全プルリクエストと `main` への push で実行。`ubuntu-latest` + PostgreSQL 16 サービスコンテナ。

| ステップ | コマンド |
|----------|---------|
| Biome lint | `npx biome ci static/` |
| フォーマットチェック | `cargo fmt --all -- --check` |
| DB マイグレーション | `sqlx migrate run`（sqlx-cli 経由） |
| sqlx キャッシュチェック | `cargo sqlx prepare --check` |
| Clippy | `cargo clippy --all-targets --all-features` |
| ユニットテスト | `cargo test --all-targets --all-features` |
| E2E テスト | `npx playwright test` |

### よくある失敗パターン

| 失敗 | 修正方法 |
|------|----------|
| `query file .sqlx/... is not up to date` | ローカルで `cargo sqlx prepare` を実行し `.sqlx/` をコミット |
| `Diff in src/...`（fmt チェック） | `cargo fmt --all` を実行しコミット |
| Clippy 警告（例：`unused variable`） | 警告を修正。CI は `-D warnings` を使用 |
| Biome lint エラー | `npx biome check --write static/` を実行しコミット |
| E2E：HTMX 操作でタイムアウト | クリックを `await page.waitForResponse(...)` で囲む |
| E2E：`net::ERR_CONNECTION_REFUSED` | サーバー起動失敗 — ローカルで `cargo run` を実行してデバッグ |

### その他のワークフロー

- **`.github/workflows/docker.yml`** — Docker イメージビルド（`SQLX_OFFLINE=true`）、GHCR へ push、Render.com デプロイ
- **`.github/workflows/release.yml`** — cargo-dist による macOS / Linux / Windows 向けバイナリリリース

---

## 6. アーキテクチャ概要

### レイヤー構造

```
src/
├── services/        — ビジネスロジック層
│   └── auth.rs      — AuthService（ログイン、登録、パスワードリセット等）
├── handlers/        — HTTP 層：ルートハンドラ（薄く、サービスに委譲）
│   ├── homepage.rs  — 認証ルート + クイズ CRUD
│   ├── account.rs   — アカウント管理
│   └── quiz/        — クイズセッション、問題、ダッシュボード
├── db/              — データ層：全 SQL クエリ
│   ├── migrations.rs — sqlx 標準マイグレーションランナー（sqlx::migrate!）
│   ├── models.rs     — 戻り値の型（プレーン構造体）
│   ├── quiz.rs       — クイズ CRUD
│   ├── session.rs    — セッションライフサイクル
│   ├── question.rs   — 問題取得、統計
│   ├── user.rs       — 認証、セッション、メール認証
│   ├── answer.rs     — 回答送信 + 採点
│   └── report.rs     — 統計クエリ
├── views/           — HTML レンダリング（Maud）
│   ├── layout.rs    — ドキュメント構造、ナビ、CSS/JS
│   ├── components.rs — 再利用可能な HTMX nav_link
│   ├── homepage.rs  — 登録/ログイン/クイズ一覧
│   ├── account.rs   — アカウント設定
│   └── quiz/        — ダッシュボード、問題、セッション画面
├── extractors.rs    — カスタム Axum エクストラクタ（AuthGuard, IsHtmx, Locale）
├── rejections.rs    — AppError + ResultExt
├── names.rs         — URL・Cookie 名定数
├── email.rs         — Resend API（メール認証、パスワードリセット）
└── statics.rs       — 組み込み静的ファイル配信（include_dir!）
```

### AppState

全データベースアクセスは `db::Db`（`sqlx::PgPool` のラッパー）を通じて行い、`AppState` に格納。認証ビジネスロジックは `AuthService` が担当：

```rust
#[derive(Clone)]
pub struct AppState {
    pub db: db::Db,
    pub auth: services::auth::AuthService,
    pub secure_cookies: bool,
}
```

認証ハンドラは `state.auth.*` メソッドに委譲する（例：`state.auth.login()`、`state.auth.register()`）。`AuthService` は `AuthRepository` trait 依存、Resend API キー、ベース URL を所有する。クイズハンドラは引き続き `state.db.*` を直接呼び出す。

### 認証システム

**`AuthGuard`** — 認証済みユーザーを取り出す。未認証なら 401 を返す。後方互換性のため旧 `admin_session` Cookie にもフォールバック。

```rust
async fn create_quiz(
    AuthGuard(user): AuthGuard,  // user: AuthUser { id, email, display_name }
    State(state): State<AppState>,
    ...
)
```

**セッション機構：** ULID トークンを `user_sessions` テーブルに保存。セッション ID は `HttpOnly; SameSite=Lax; Max-Age=86400` Cookie（`user_session`）に格納。ミドルウェアが毎レスポンスで Max-Age をリフレッシュ（スライディング有効期限）。

**CSRF 保護：** ミドルウェアが状態変更リクエスト（`POST/PUT/PATCH/DELETE`）で `HX-Request: true` または `Origin`/`Host` ペア一致のどちらかを要求。

**パスワードハッシュ：** `argon2` クレート。デバッグビルドでのスタックオーバーフロー回避のため、専用 4MB スタックスレッドで実行。

### HTMX ナビゲーションパターン

全ナビゲーションリンクは同じパターン：

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

ハンドラは `IsHtmx` でレスポンスを分岐：

- **HTMX リクエスト** → `<title>` + 内部 HTML フラグメントのみ返す（`views::titled()`）
- **ブラウザ直接アクセス** → 完全な HTML ドキュメントを返す（`views::page_with_user()`）

`views::render()` 関数がこれを自動処理：

```rust
pub fn render(is_htmx: bool, title: &str, body: Markup, locale: &str, user_name: Option<&str>) -> Markup {
    if is_htmx { titled(title, body) }
    else { page_with_user(title, body, locale, user_name) }
}
```

### `hx-ext="json-enc"`

一部のフォームは `json-enc` 拡張で `application/json` を送信する。これらのハンドラは `Json<T>` エクストラクタを使用。`json-enc` が数値を文字列で送る場合に対応するカスタム `deserialize_string_or_i32` serde デシリアライザがある。

### i18n（rust-i18n）

ロケールファイル：`locales/en.yml`、`locales/ja.yml`、`locales/zh-CN.yml`、`locales/zh-TW.yml`。`Locale` エクストラクタが `lang` Cookie を読み取り（デフォルト `"en"`）、ビュー関数に渡す：

```rust
html! {
    h1 { (t!("homepage.my_quizzes", locale = locale)) }
}
```

### エラーハンドリング

`rejections.rs` の `AppError` には3つのバリアント：

| バリアント | ステータス | 用途 |
|------------|-----------|------|
| `Internal(&'static str)` | 500 | サーバーエラー（実際のエラーをログ出力） |
| `Unauthorized` | 401 | ログインページへリダイレクト |
| `Input(&'static str)` | 400 | 不正なユーザー入力 |

`ResultExt` トレイトで簡潔に変換：

```rust
let quizzes = state.db.quizzes(user.id)
    .await
    .reject("could not get quizzes")?;       // → AppError::Internal

let questions = parse_json(input)
    .reject_input("failed to decode quiz")?;  // → AppError::Input
```

`reject()` は元のエラーを `tracing::error!` レベルでログ出力し、クライアントには一般的なメッセージのみ返す。

### 静的ファイル

`include_dir!("static")` でバイナリにコンパイル時に埋め込み。`/static/{path}` で配信（`Cache-Control: max-age=3600, must-revalidate`）。

---

## 7. トラブルシューティング

| 症状 | 原因 | 修正 |
|------|------|------|
| 起動時 `connection refused` | PostgreSQL 未起動 | `docker compose up -d` |
| ポート 5432 が使用中 | 別の PostgreSQL プロセス | `sudo service postgresql stop` → `docker compose up -d` |
| `STATUS_STACK_OVERFLOW`（Windows） | デバッグ時の argon2 スタック消費 | `cargo clean && cargo build` |
| ビルドが反映されない（Windows） | exe がロック | `taskkill /IM quizinart.exe /F` → `cargo clean && cargo build` |
| `docker: permission denied`（Linux） | docker グループ未所属 | `sudo usermod -aG docker $USER` → 再ログイン |
| `sqlx prepare --check` 失敗："not up to date" | SQL 変更後 `.sqlx/` 未更新 | `cargo sqlx prepare` → `.sqlx/` をコミット |
| `sqlx prepare --check` 失敗："connecting to database" | `.env` 不在または DB 停止 | `cp .env.example .env` → `docker compose up -d` |
| E2E `net::ERR_CONNECTION_REFUSED` | サーバー未起動 | まず `cargo run` を手動実行して確認 |
| E2E：HTMX 操作でタイムアウト | レスポンス待機不足 | `Promise.all([page.waitForResponse(...), page.click(...)])` で囲む |
| マイグレーションエラー："column already exists" | 非冪等なマイグレーション SQL | `ADD COLUMN IF NOT EXISTS` / `CREATE TABLE IF NOT EXISTS` を使用 |
| `git push` が pre-push hook で拒否 | 5つのチェックのいずれかが失敗 | 各チェックを個別に実行（[Pre-push hook](#pre-push-hook) 参照） |
