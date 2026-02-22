# 開発環境構築手順

[English](setup.md)

## 前提条件

### Windows

以下を **管理者権限** のターミナルで実行してください。

#### 1. Rust ツールチェーン

```powershell
winget install Rustlang.Rustup
```

#### 2. C++ ビルドツール（Rust のリンカに必要）

```powershell
winget install Microsoft.VisualStudio.2022.BuildTools --override "--wait --passive --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"
```

新しいターミナルを開いて確認：

```powershell
rustup --version
cargo --version
```

#### 3. Docker Desktop（ローカル PostgreSQL に必要）

```powershell
winget install Docker.DockerDesktop
```

インストール後、Docker Desktop を開き、Settings → General で **WSL 2 based engine** を有効にしてください。

確認：

```powershell
docker --version
docker compose version
```

#### 4. Node.js（E2E テストに必要）

```powershell
winget install Volta.Volta
volta install node
```

#### 5. Playwright ブラウザ（E2E テストに必要）

プロジェクトディレクトリで実行：

```powershell
cd quizinart
npx playwright install --with-deps chromium
```

---

### WSL / Linux

#### 1. Rust ツールチェーン

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
```

#### 2. ビルドツール（libsql-ffi / pq のビルドに必要）

```bash
sudo apt update && sudo apt install -y build-essential pkg-config libssl-dev postgresql-client
```

`postgresql-client` は `psql` コマンドを提供し、ローカルデータベースの直接確認に便利です。

#### 3. Docker（ローカル PostgreSQL に必要）

WSL Ubuntu に Docker Engine（Docker Desktop ではなく）をインストール：

```bash
# Docker の公式 GPG キーとリポジトリを追加
sudo apt-get install -y ca-certificates curl
sudo install -m 0755 -d /etc/apt/keyrings
sudo curl -fsSL https://download.docker.com/linux/ubuntu/gpg -o /etc/apt/keyrings/docker.asc
echo "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.asc] https://download.docker.com/linux/ubuntu $(. /etc/os-release && echo "$VERSION_CODENAME") stable" | sudo tee /etc/apt/sources.list.d/docker.list > /dev/null
sudo apt-get update
sudo apt-get install -y docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin

# sudo なしで docker を実行できるように
sudo usermod -aG docker $USER
newgrp docker
```

確認：

```bash
docker --version
docker compose version
```

#### 4. Node.js（E2E テストに必要）

```bash
curl -fsSL https://deb.nodesource.com/setup_22.x | sudo -E bash -
sudo apt install -y nodejs
```

#### 5. Playwright ブラウザ（E2E テストに必要）

プロジェクトディレクトリで実行：

```bash
cd quizinart
npx playwright install --with-deps chromium
```

---

## 初回セットアップ手順

### 1. クローン

```bash
git clone https://github.com/numachang/quizinart-salesforce-platform-admin.git
cd quizinart-salesforce-platform-admin
git clone https://github.com/numachang/quizinart.git
```

### 2. 環境設定ファイルのコピー

```bash
cp .env.example .env
# 必要に応じて .env を編集
```

### 3. ローカル PostgreSQL の起動

```bash
docker compose up -d
```

ポート 5432 で PostgreSQL コンテナが起動します。
認証情報は `docker-compose.yml` に定義されています。

停止する場合：

```bash
docker compose down
```

データを含めて完全に削除する場合：

```bash
docker compose down -v
```

### 4. アプリの起動

```bash
cargo run --manifest-path quizinart/Cargo.toml
```

ブラウザで http://127.0.0.1:1414 を開きます。

### 5. 初回アクセス

1. 初回アクセス時に管理者パスワードを設定
2. **Create Quiz** をクリックし、名前を付けて `data/salesforce_quiz.json` をアップロード
3. クイズページで名前を入力して学習開始

---

## トラブルシューティング

| 症状 | 原因 | 対処 |
|------|------|------|
| `STATUS_STACK_OVERFLOW` で起動時にクラッシュ | リモート DB 接続 + argon2 でスタック不足 | `cargo clean && cargo build` |
| ビルドが反映されない（Windows） | exe がロック / キャッシュが古い | `taskkill /IM quizinart.exe /F` → `cargo clean && cargo build` |
| Docker コンテナがすぐに終了する | ポート 5432 が使用中 | 既存の PostgreSQL を停止: `sudo service postgresql stop` |
| `docker: permission denied`（Linux） | ユーザーが docker グループに未所属 | `sudo usermod -aG docker $USER` して再ログイン |
