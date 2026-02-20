# マルチテナント化 + 認証導入 改修計画（Render + Turso 前提）

## 目的

現行の「管理者が1つのクイズ空間を管理し、全ユーザーが同じURL/同じ履歴を共有する」設計から、
**ユーザー単位でクイズ・受験履歴・結果を分離する設計**へ移行する。

- データ分離: ユーザーごとにクイズ資産と学習履歴を分離
- 認証: 低運用コストかつ安全性の高い方式を採用
- 配備: Render（アプリ）+ Turso（DB）で安定運用

## 計画の前提（コードベース整合性）

- 本計画は **作成時点の `work` ブランチ最新コミット**を基準に記述。
- この環境には `origin` リモートが設定されておらず、外部の「最新 main」との自動突合は未実施。
- 実装着手時は必ず以下を先に実施する。
  1. リモート `main` を fetch
  2. `main` へ rebase/merge
  3. ルーティング・DB スキーマ差分を再確認
  4. 本ドキュメントとの差分を更新

---

## 現状の課題

1. URL を知っていれば同一クイズ空間にアクセスできる
2. 履歴・統計が共有され、個人利用としての整合性が崩れる
3. 管理者権限と受験者権限の境界が曖昧
4. 本番公開時にデータ漏えい・誤操作のリスクが高い

---

## 認証方式の推奨

### 推奨: Google OAuth（OIDC）

最小実装での安全性と運用性のバランスが良い。

- パスワード保管・リセット機能が不要
- MFA/不正検知を Google 側に委譲できる
- 将来 Apple/GitHub 追加時も OIDC の枠組みで拡張しやすい

### 代替: メール + パスワード

初期実装は可能だが、以下を自前で持つ必要があり負荷が高い。

- パスワードハッシュ管理（Argon2）
- リセットフロー、メール送信、アカウント凍結対策
- ブルートフォース対策、追加監査

> 結論: 「技術的にハイカロリーではなく、でもセキュア」に最も合うのは **Google OAuth(OIDC)**。

---

## データモデル変更（最小構成）

### 新規テーブル

- `users`
  - `id` (uuid)
  - `provider` (`google`)
  - `provider_user_id`
  - `email`
  - `display_name`
  - `created_at`, `updated_at`

- `user_sessions`（ログインセッション管理）
  - `id`（ランダムトークン）
  - `user_id`
  - `expires_at`
  - `created_at`

### 既存テーブルへの追加

- `quizzes.user_id`（NOT NULL）
- `quiz_sessions.user_id`（NOT NULL）
- `answers.user_id`（NOT NULL）
- （必要なら）`bookmarks.user_id`

### 制約

- 主要クエリは必ず `WHERE user_id = ?` を含む
- 複合インデックス追加（例: `quizzes(user_id, id)`）
- 外部キーで `user_id` 整合性を担保

---

## アプリケーション改修ポイント

1. **認証ミドルウェア追加**
   - 未ログイン時は `/login` へリダイレクト
   - セッション Cookie は `HttpOnly` / `Secure` / `SameSite=Lax`
   - **直リンクアクセス対策**: ナビゲーション経由ではなく URL 直打ちでも、同じガードを必ず適用

2. **ルーティング分離**
   - 公開トップ: ランディング + ログイン導線
   - 認証済み領域: ダッシュボード、クイズ管理、受験、結果

3. **全 DB アクセスのユーザー境界化**
   - CRUD 全てで `user_id` を受け取り、他人データを参照不可にする

4. **管理機能の再定義**
   - 従来の単一管理者概念を廃止
   - 各ユーザーが「自分のクイズの管理者」になる

5. **監査ログ（任意）**
   - 重要操作（クイズ削除、再生成等）を `audit_logs` に記録

### ページ直リンク時の認証・認可制御（必須）

「画面から遷移した時だけチェックする」のでは不十分。**各エンドポイントで毎回**実施する。

- 未認証ユーザーが保護ページへ直リンク
  - 挙動: `/login` へ 302（return_to 付き）
- 認証済みユーザーが他人リソースへ直リンク（`/quizzes/:id` など）
  - 挙動: 404（存在秘匿）または 403（方針統一）
- API 直叩き（POST/PUT/DELETE）
  - 挙動: セッション + CSRF + `user_id` 所有チェックを通らない限り拒否

#### ルート単位ガード方針

- 公開: `/`, `/login`, `/auth/google/*`, `/health`
- 要認証（全て直リンク保護）:
  - `/dashboard`
  - `/quizzes/*`
  - `/sessions/*`
  - `/results/*`
  - `/api/*`（状態変更は CSRF 必須）

#### 実装ルール（抜け漏れ防止）

1. ハンドラーの最初で `current_user` を解決（失敗時即リダイレクト/401）
2. `:id` を受け取る処理は必ず `WHERE id = ? AND user_id = ?`
3. 書き込み系は `user_id` をリクエスト値から受け取らず、サーバー側で注入
4. テンプレート表示前に所有権チェック済みであることを保証

---

## 移行計画（段階的）

### Phase 1: 認証基盤

- OIDC クライアント設定
- `/auth/google/login` と `/auth/google/callback` 実装
- `users`, `user_sessions` 作成
- ログイン/ログアウト UI 追加

### Phase 2: マルチテナント化

- 既存テーブルへ `user_id` 追加
- 全クエリに `user_id` 条件を導入
- 既存 handlers をユーザー文脈対応

### Phase 3: 既存データ移行

- 既存データを「移行用 owner ユーザー」に紐づけ
- マイグレーション後の整合性検証

### Phase 4: 本番運用対応

- Render 環境変数整備
- Turso 接続確認（TLS、接続文字列、トークン）
- セキュリティヘッダー/Cookie 設定の最終確認

---

## Render + Turso 運用設定メモ

- Render 環境変数
  - `APP_URL`
  - `GOOGLE_CLIENT_ID`
  - `GOOGLE_CLIENT_SECRET`
  - `SESSION_SECRET`
  - `DATABASE_URL`（Turso URL）
  - `DATABASE_AUTH_TOKEN`

- Google Console
  - Authorized redirect URI: `https://<your-domain>/auth/google/callback`

- Turso
  - 本番 DB とステージング DB を分離
  - 破壊的マイグレーション前にバックアップ

---

## セキュリティチェックリスト

- [ ] セッション ID は十分長いランダム値
- [ ] Cookie は `HttpOnly + Secure + SameSite`
- [ ] 認証・認可失敗時に情報過多なエラーを返さない
- [ ] CSRF 対策（状態変更エンドポイント）
- [ ] レートリミット（ログイン開始/コールバック）
- [ ] 監査可能なログ（PII マスク）

---

## 実装優先順位（おすすめ）

1. Google OAuth ログイン導入
2. `users` / `user_sessions` マイグレーション
3. `quizzes` と `quiz_sessions` の `user_id` 分離
4. 画面導線の整理（ログイン前後）
5. 回答履歴・統計の完全分離

---

## 受け入れ条件（Definition of Done）

- ユーザー A の作成したクイズ・履歴を、ユーザー B から一切参照できない
- 未ログインで管理/受験 API にアクセスできない
- ログイン〜受験〜結果確認までの主要フローが既存同等以上で動作
- Render 本番環境で Google OAuth コールバックが成功する
- Turso 上でマイグレーションが再現可能
