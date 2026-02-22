# Development Environment Setup

[日本語](setup_ja.md)

## Prerequisites

### Windows

Run the following in an **administrator** terminal:

#### 1. Rust toolchain

```powershell
winget install Rustlang.Rustup
```

#### 2. C++ Build Tools (required by Rust linker)

```powershell
winget install Microsoft.VisualStudio.2022.BuildTools --override "--wait --passive --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"
```

Open a new terminal and verify:

```powershell
rustup --version
cargo --version
```

#### 3. Docker Desktop (required for local PostgreSQL)

```powershell
winget install Docker.DockerDesktop
```

After installation, open Docker Desktop and enable **WSL 2 based engine** in Settings → General.

Verify:

```powershell
docker --version
docker compose version
```

#### 4. Node.js (for E2E tests)

```powershell
winget install Volta.Volta
volta install node
```

#### 5. Playwright browsers (for E2E tests)

Run in the project directory:

```powershell
cd quizinart
npx playwright install --with-deps chromium
```

---

### WSL / Linux

#### 1. Rust toolchain

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
```

#### 2. Build tools (required by libsql-ffi / pq)

```bash
sudo apt update && sudo apt install -y build-essential pkg-config libssl-dev
```

#### 3. Docker (required for local PostgreSQL)

Install Docker Engine (not Docker Desktop) on WSL Ubuntu:

```bash
# Add Docker's official GPG key and repository
sudo apt-get install -y ca-certificates curl
sudo install -m 0755 -d /etc/apt/keyrings
sudo curl -fsSL https://download.docker.com/linux/ubuntu/gpg -o /etc/apt/keyrings/docker.asc
echo "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.asc] https://download.docker.com/linux/ubuntu $(. /etc/os-release && echo "$VERSION_CODENAME") stable" | sudo tee /etc/apt/sources.list.d/docker.list > /dev/null
sudo apt-get update
sudo apt-get install -y docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin

# Run docker without sudo
sudo usermod -aG docker $USER
newgrp docker
```

Verify:

```bash
docker --version
docker compose version
```

#### 4. Node.js (for E2E tests)

```bash
curl -fsSL https://deb.nodesource.com/setup_22.x | sudo -E bash -
sudo apt install -y nodejs
```

#### 5. Playwright browsers (for E2E tests)

Run in the project directory:

```bash
cd quizinart
npx playwright install --with-deps chromium
```

---

## First-time Setup

### 1. Clone

```bash
git clone https://github.com/numachang/quizinart-salesforce-platform-admin.git
cd quizinart-salesforce-platform-admin
git clone https://github.com/numachang/quizinart.git
```

### 2. Copy the env file

```bash
cp .env.example .env
# Edit .env as needed
```

### 3. Start local PostgreSQL

```bash
docker compose up -d
```

This starts a PostgreSQL container on port 5432.
Default credentials are defined in `docker-compose.yml`.

To stop:

```bash
docker compose down
```

To wipe the database volume:

```bash
docker compose down -v
```

### 4. Start the application

```bash
cargo run --manifest-path quizinart/Cargo.toml
```

Open http://127.0.0.1:1414 in your browser.

### 5. First visit

1. Set an admin password on first visit
2. Click **Create Quiz**, name it, and upload `data/salesforce_quiz.json`
3. Enter your name on the quiz page and start

---

## Troubleshooting

| Symptom | Cause | Fix |
|---------|-------|-----|
| `STATUS_STACK_OVERFLOW` on startup | Remote DB connection + argon2 stack usage | `cargo clean && cargo build` |
| Build not reflecting changes (Windows) | exe locked / stale cache | `taskkill /IM quizinart.exe /F` → `cargo clean && cargo build` |
| Docker container exits immediately | Port 5432 already in use | Stop any running PostgreSQL: `sudo service postgresql stop` |
| `docker: permission denied` (Linux) | User not in docker group | `sudo usermod -aG docker $USER` then log out and back in |
