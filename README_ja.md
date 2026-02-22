# Quizinart

[English](README.md)

学習特化クイズアプリケーション。Rust、HTMX、PostgreSQL で構築。

[frectonz/quizzy](https://github.com/frectonz/quizzy) をフォークし、効果的な学習のための機能を追加しています。

## スクリーンショット

| ダッシュボード | クイズ画面 | 結果画面 |
|--------------|-----------|---------|
| ![ダッシュボード](docs/screenshots/dashboard.png) | ![クイズ](docs/screenshots/quiz-answer.png) | ![結果](docs/screenshots/session-results.png) |

ライト/ダークテーマ切り替え、多言語対応（English, 日本語, 简体中文, 繁體中文）：

![テーマ・言語切り替え](docs/screenshots/theme-language.png)

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
| バックエンド | Rust, Axum, Maud |
| フロントエンド | HTMX, PicoCSS |
| データベース | PostgreSQL（Docker ローカル、本番は Neon） |

## クイックスタート

### 前提条件

環境構築の詳細手順（Rust、Docker、Node.js、Playwright）は **[docs/setup_ja.md](docs/setup_ja.md)** を参照してください。（[English](docs/setup.md)）

### ローカル実行

```bash
# 環境設定ファイルをコピー（必要に応じて編集）
cp .env.example .env

# 起動（.env を自動読み込み）
cargo run --manifest-path quizinart/Cargo.toml
```

`.env.example` にはローカル開発用のデフォルト値（ポート1414）が含まれています。`docker compose up -d` で PostgreSQL コンテナを起動してから実行してください。

ブラウザで http://127.0.0.1:1414 を開きます。

### 初回セットアップ

1. 初回アクセス時に管理者パスワードを設定
2. **Create Quiz** をクリックし、名前を付けてクイズ JSON ファイルをアップロード
3. クイズページで名前を入力して学習開始

## サンプルクイズ

`samples/` ディレクトリに一般教育のサンプル問題ファイル（各30問・6カテゴリ）が含まれています：

| ファイル | 言語 |
|---------|------|
| `samples/general-education-en.json` | English |
| `samples/general-education-ja.json` | 日本語 |
| `samples/general-education-zh-CN.json` | 简体中文 |
| `samples/general-education-zh-TW.json` | 繁體中文 |

管理者パスワード設定後、**クイズ作成** から名前を付けてこれらのファイルをアップロードすれば、すぐに試せます。

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
├── samples/                   # サンプルクイズ JSON ファイル
├── static/                    # CSS, JS, 画像
└── Cargo.toml
```

## ライセンス

MIT

## クレジット

- クイズエンジンは frectonz の [Quizzy](https://github.com/frectonz/quizzy) がベース
- Claude Code で構築
