# Quizinart

[日本語](README_ja.md)

A self-hosted quiz app built for serious learners. Import your own questions, track progress over time, and focus on what you got wrong — all from any device.

Built with Rust, HTMX, and PostgreSQL. Forked from [frectonz/quizzy](https://github.com/frectonz/quizzy) and significantly extended.

## Screenshots

| Dashboard | Quiz | Results |
|-----------|------|---------|
| ![Dashboard](docs/screenshots/dashboard.png) | ![Quiz](docs/screenshots/quiz-answer.png) | ![Results](docs/screenshots/session-results.png) |

Light/Dark theme and multi-language support (English, Japanese, Simplified Chinese, Traditional Chinese):

![Theme & Language](docs/screenshots/theme-language.png)

## Why Quizinart?

Most quiz apps focus on gamification. Quizinart focuses on **learning efficiency**: pinpoint your weak spots, revisit only what you need, and measure your progress with hard numbers.

## Features

### Study smarter
- **Smart question selection** — choose from unanswered, previously incorrect, sequential, or random questions
- **Bookmark questions** — flag tricky questions during a session and revisit them later
- **Retry incorrect** — instantly create a new session from only the questions you missed
- **Retry bookmarked** — create a session from only your flagged questions
- **Per-option explanations** — every answer choice can have a detailed explanation, not just the correct one

### Track your progress
- **Category statistics** — see your accuracy broken down by topic on the dashboard
- **Session history** — browse, rename, or delete past sessions
- **Resume anytime** — pick up an incomplete session right where you left off

### Bring your own content
- **JSON import** — upload questions from a simple JSON format
- **Multiple quizzes** — manage as many quiz sets as you need
- **Single & multiple choice** — supports both question types

### Multi-user & multi-device
- **User accounts** — register with email and password, with optional email verification
- **Password reset** — forgot your password? Reset it via email
- **Responsive UI** — works on desktop and mobile
- **Multi-language** — English, Japanese, Simplified Chinese, Traditional Chinese

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Backend | Rust, Axum, Maud |
| Frontend | HTMX, PicoCSS |
| Database | PostgreSQL (Docker locally, Neon for production) |
| Auth | Argon2 password hashing, session cookies |
| Email | Resend (optional, for verification & password reset) |

## Quick Start

### Prerequisites

See **[docs/setup.md](docs/setup.md)** for full environment setup instructions (Rust, Docker, Node.js, Playwright). ([日本語](docs/setup_ja.md))

For day-to-day development workflow, see **[docs/developer-guide.md](docs/developer-guide.md)**. ([日本語](docs/developer-guide_ja.md))

### Run locally

```bash
# Start the local PostgreSQL container
docker compose up -d

# Copy the env sample and edit if needed
cp .env.example .env

# Start the app (reads .env automatically)
cargo run
```

Open http://127.0.0.1:1414 in your browser.

### First-time setup

1. Register an account (email + password)
2. Create a quiz — click **Create Quiz**, name it, and upload a quiz JSON file
3. Start learning!

## Sample Quiz

Sample general education quiz files (30 questions each, 6 categories) are included in the `samples/` directory:

| File | Language |
|------|----------|
| `samples/general-education-en.json` | English |
| `samples/general-education-ja.json` | 日本語 |
| `samples/general-education-zh-CN.json` | 简体中文 |
| `samples/general-education-zh-TW.json` | 繁體中文 |

After registering, click **Create Quiz**, give it a name, and upload one of these files to try it out.

## Quiz JSON Format

Questions are imported from a single JSON file. Each question has a text, category, answer options with optional explanations, and a multiple-choice flag:

```json
[
  {
    "question": "What is the capital of France?",
    "category": "Geography",
    "isMultipleChoice": false,
    "options": [
      { "text": "Berlin", "isAnswer": false, "explanation": "Berlin is the capital of Germany." },
      { "text": "Paris", "isAnswer": true, "explanation": "Paris has been the capital of France since the 10th century." },
      { "text": "Madrid", "isAnswer": false, "explanation": "Madrid is the capital of Spain." }
    ]
  }
]
```

## Project Structure

```
quizinart/
├── src/
│   ├── main.rs                # Entry point, CLI args
│   ├── lib.rs                 # App routing & middleware
│   ├── db/                    # Database layer
│   │   ├── models.rs          # Shared data models
│   │   ├── session.rs         # Session CRUD
│   │   ├── question.rs        # Questions & stats
│   │   ├── answer.rs          # Answer recording
│   │   ├── user.rs            # User accounts & auth
│   │   ├── quiz.rs            # Quiz management
│   │   ├── admin.rs           # Admin operations
│   │   ├── report.rs          # Reporting
│   │   ├── helpers.rs         # DB helpers
│   │   └── migrations.rs      # Migration runner
│   ├── services/              # Business logic
│   │   └── auth.rs            # Auth service (login, register, password reset)
│   ├── handlers/              # HTTP handlers
│   │   ├── quiz/              # Quiz flow (dashboard, session, question)
│   │   ├── homepage.rs        # Landing & auth pages
│   │   └── account.rs         # Account management
│   ├── views/                 # Maud HTML templates
│   │   ├── layout.rs          # Page shell & shared layout
│   │   ├── quiz/              # Quiz views (dashboard, session, question)
│   │   ├── homepage.rs        # Home & auth views
│   │   ├── account.rs         # Account views
│   │   └── components.rs      # Reusable UI components
│   ├── email.rs               # Email sending (Resend)
│   ├── extractors.rs          # Axum extractors (auth, locale)
│   ├── names.rs               # Route & cookie constants
│   ├── utils.rs               # Helpers
│   └── statics.rs             # Static file serving
├── migrations/                # PostgreSQL migrations
├── samples/                   # Sample quiz JSON files
├── static/                    # CSS, JS, images
├── e2e/                       # Playwright E2E tests
└── Cargo.toml
```

## License

MIT

## Credits

- Quiz engine based on [Quizzy](https://github.com/frectonz/quizzy) by frectonz
- Built with Claude Code
