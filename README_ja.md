# Quizinart

[English](README.md)

セルフホスト型の学習特化クイズアプリケーション。Rust、HTMX、libSQL で構築。

[frectonz/quizzy](https://github.com/frectonz/quizzy) をフォークし、効果的な学習のための機能を追加しています。

## 特徴

- **スマート出題** — 未出題・不正解・ランダムから選択可能
- **セッション再開** — 名前を入力するだけで、別のデバイスからでも続きから再開
- **選択肢ごとの解説** — すべての回答選択肢に詳細な解説を付与可能
- **カテゴリ別統計** — ダッシュボードで分野ごとの正答率を確認
- **不正解リトライ** — 間違えた問題だけで新しいセッションを即座に作成
- **マルチデバイス** — PC・スマートフォンどちらからでも利用可能
- **アカウント不要** — 名前を入力するだけで学習開始

## 技術スタック

| レイヤー | 技術 |
|---------|------|
| バックエンド | Rust, Warp, Maud |
| フロントエンド | HTMX, PicoCSS |
| データベース | libSQL (SQLite互換、リモートはTurso) |

## クイックスタート

### 前提条件

- **Rust** 1.70 以上 — [rustup.rs](https://rustup.rs/)

### ローカル実行

```bash
cd quizzy

# ローカルSQLiteで起動（Tursoアカウント不要）
URL="file:local.db" AUTH_TOKEN="" ADDRESS="127.0.0.1:1414" RUST_LOG=info cargo run
```

ブラウザで http://127.0.0.1:1414 を開きます。

### 初回セットアップ

1. 初回アクセス時に管理者パスワードを設定
2. **Create Quiz** をクリックし、名前を付けてクイズ JSON ファイルをアップロード
3. クイズページで名前を入力して学習開始

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
quizzy/
├── src/
│   ├── main.rs                # エントリーポイント、ルーティング、認証
│   ├── db/                    # データベース層
│   │   ├── schema.rs          # テーブル定義
│   │   ├── session.rs         # セッション CRUD
│   │   ├── question.rs        # 問題・統計
│   │   ├── answer.rs          # 回答記録
│   │   ├── admin.rs           # 管理者認証
│   │   ├── quiz.rs            # クイズ管理
│   │   └── report.rs          # レポート
│   ├── handlers/              # HTTPハンドラー
│   │   ├── quiz.rs            # クイズフローロジック
│   │   └── homepage.rs        # ランディングページ
│   ├── views/                 # Maud HTMLテンプレート
│   │   ├── layout.rs          # ページシェル
│   │   ├── quiz.rs            # クイズ画面
│   │   └── homepage.rs        # ホーム画面
│   ├── names.rs               # ルート・Cookie定数
│   ├── utils.rs               # ヘルパー
│   └── statics.rs             # 静的ファイル配信
├── static/                    # CSS, JS, 画像
└── Cargo.toml
```

## デプロイ

### 環境変数

| 変数 | 説明 | 例 |
|------|------|-----|
| `URL` | libSQL接続文字列 | `file:local.db` または `libsql://your-db.turso.io` |
| `AUTH_TOKEN` | Turso認証トークン（ローカルは空） | `eyJhbGc...` |
| `ADDRESS` | リッスンアドレス | `127.0.0.1:1414` |
| `RUST_LOG` | ログレベル | `info` |

### Render.com へのデプロイ

1. GitHub にプッシュ
2. [render.com](https://render.com) で **Web Service** を作成
3. 環境を **Docker** に設定、ルートディレクトリは `quizzy`
4. 環境変数を追加（`URL`, `AUTH_TOKEN`, `ADDRESS=0.0.0.0:1414`）

### Turso（リモートデータベース）の使用

```bash
turso db create my-quiz
turso db show my-quiz           # URLを取得
turso db tokens create my-quiz  # トークンを取得
```

## ライセンス

MIT

## クレジット

- クイズエンジンは frectonz の [Quizzy](https://github.com/frectonz/quizzy) がベース
- Claude Code で構築
