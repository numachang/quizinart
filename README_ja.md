# Quizinart

[English](README.md)

本気で学ぶ人のためのセルフホスト型クイズアプリ。自分の問題をインポートし、学習の進捗を追跡し、間違えた問題だけを集中的に復習 — すべてどのデバイスからでも。

Rust、HTMX、PostgreSQL で構築。[frectonz/quizzy](https://github.com/frectonz/quizzy) をフォークし、大幅に拡張しています。

## スクリーンショット

| ダッシュボード | クイズ画面 | 結果画面 |
|--------------|-----------|---------|
| ![ダッシュボード](docs/screenshots/dashboard.png) | ![クイズ](docs/screenshots/quiz-answer.png) | ![結果](docs/screenshots/session-results.png) |

ライト/ダークテーマ切り替え、多言語対応（English, 日本語, 简体中文, 繁體中文）：

![テーマ・言語切り替え](docs/screenshots/theme-language.png)

## なぜ Quizinart？

多くのクイズアプリはゲーム性を重視しますが、Quizinart は**学習効率**に特化しています。弱点を特定し、必要な問題だけを復習し、数字で進捗を確認できます。

## 特徴

### 効率的に学ぶ
- **スマート出題** — 未出題・不正解・順番通り・ランダムから出題方式を選択
- **ブックマーク** — セッション中に気になる問題をフラグして後から見返せる
- **不正解リトライ** — 間違えた問題だけで新しいセッションを即座に作成
- **ブックマークリトライ** — フラグした問題だけでセッションを作成
- **選択肢ごとの解説** — 正解だけでなく、すべての選択肢に詳細な解説を付与可能

### 進捗を把握する
- **カテゴリ別統計** — ダッシュボードで分野ごとの正答率を確認
- **セッション履歴** — 過去のセッションを閲覧・リネーム・削除
- **いつでも再開** — 途中のセッションを続きから再開

### 自分のコンテンツを持ち込む
- **JSON インポート** — シンプルな JSON 形式で問題をアップロード
- **複数クイズ管理** — 必要なだけクイズセットを作成・管理
- **単一選択 & 複数選択** — 両方の出題形式に対応

### マルチユーザー & マルチデバイス
- **ユーザーアカウント** — メールアドレスとパスワードで登録（メール認証はオプション）
- **パスワードリセット** — パスワードを忘れてもメールでリセット可能
- **レスポンシブ UI** — PC・スマートフォンどちらからでも快適に利用
- **多言語対応** — English, 日本語, 简体中文, 繁體中文

## 技術スタック

| レイヤー | 技術 |
|---------|------|
| バックエンド | Rust, Axum, Maud |
| フロントエンド | HTMX, PicoCSS |
| データベース | PostgreSQL（Docker ローカル、本番は Neon） |
| 認証 | Argon2 パスワードハッシュ、セッション Cookie |
| メール | Resend（オプション、メール認証・パスワードリセット用） |

## クイックスタート

### 前提条件

環境構築の詳細手順（Rust、Docker、Node.js、Playwright）は **[docs/setup_ja.md](docs/setup_ja.md)** を参照してください。（[English](docs/setup.md)）

日常の開発ワークフローについては **[docs/developer-guide_ja.md](docs/developer-guide_ja.md)** を参照してください。（[English](docs/developer-guide.md)）

### ローカル実行

```bash
# PostgreSQL コンテナを起動
docker compose up -d

# 環境設定ファイルをコピー（必要に応じて編集）
cp .env.example .env

# アプリを起動（.env を自動読み込み）
cargo run
```

ブラウザで http://127.0.0.1:1414 を開きます。

### 初回セットアップ

1. アカウントを登録（メールアドレス + パスワード）
2. クイズを作成 — **Create Quiz** をクリックし、名前を付けて JSON ファイルをアップロード
3. 学習開始！

## サンプルクイズ

`samples/` ディレクトリに一般教育のサンプル問題ファイル（各30問・6カテゴリ）が含まれています：

| ファイル | 言語 |
|---------|------|
| `samples/general-education-en.json` | English |
| `samples/general-education-ja.json` | 日本語 |
| `samples/general-education-zh-CN.json` | 简体中文 |
| `samples/general-education-zh-TW.json` | 繁體中文 |

アカウント登録後、**クイズ作成** から名前を付けてこれらのファイルをアップロードすれば、すぐに試せます。

## クイズ JSON 形式

問題は1つの JSON ファイルからインポートします。各問題には問題文、カテゴリ、解説付きの選択肢、複数選択フラグが含まれます：

```json
[
  {
    "question": "フランスの首都はどこですか？",
    "category": "地理",
    "isMultipleChoice": false,
    "options": [
      { "text": "ベルリン", "isAnswer": false, "explanation": "ベルリンはドイツの首都です。" },
      { "text": "パリ", "isAnswer": true, "explanation": "パリは10世紀からフランスの首都です。" },
      { "text": "マドリード", "isAnswer": false, "explanation": "マドリードはスペインの首都です。" }
    ]
  }
]
```

## プロジェクト構成

```
quizinart/
├── src/
│   ├── main.rs                # エントリーポイント、CLI引数
│   ├── lib.rs                 # ルーティング・ミドルウェア
│   ├── db/                    # データベース層
│   │   ├── models.rs          # 共有データモデル
│   │   ├── session.rs         # セッション CRUD
│   │   ├── question.rs        # 問題・統計
│   │   ├── answer.rs          # 回答記録
│   │   ├── user.rs            # ユーザーアカウント・認証
│   │   ├── quiz.rs            # クイズ管理
│   │   ├── admin.rs           # 管理者操作
│   │   ├── report.rs          # レポート
│   │   ├── helpers.rs         # DBヘルパー
│   │   └── migrations.rs      # マイグレーション実行
│   ├── services/              # ビジネスロジック
│   │   └── auth.rs            # 認証サービス（ログイン、登録、パスワードリセット）
│   ├── handlers/              # HTTPハンドラー
│   │   ├── quiz/              # クイズフロー（ダッシュボード、セッション、問題）
│   │   ├── homepage.rs        # ランディング・認証ページ
│   │   └── account.rs         # アカウント管理
│   ├── views/                 # Maud HTMLテンプレート
│   │   ├── layout.rs          # ページシェル・共有レイアウト
│   │   ├── quiz/              # クイズ画面（ダッシュボード、セッション、問題）
│   │   ├── homepage.rs        # ホーム・認証画面
│   │   ├── account.rs         # アカウント画面
│   │   └── components.rs      # 再利用可能UIコンポーネント
│   ├── email.rs               # メール送信（Resend）
│   ├── extractors.rs          # Axumエクストラクター（認証、ロケール）
│   ├── names.rs               # ルート・Cookie定数
│   ├── utils.rs               # ヘルパー
│   └── statics.rs             # 静的ファイル配信
├── migrations/                # PostgreSQLマイグレーション
├── samples/                   # サンプルクイズ JSON ファイル
├── static/                    # CSS, JS, 画像
├── e2e/                       # Playwright E2Eテスト
└── Cargo.toml
```

## ライセンス

MIT

## クレジット

- クイズエンジンは frectonz の [Quizzy](https://github.com/frectonz/quizzy) がベース
- Claude Code で構築
