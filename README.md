# Quizinart

[日本語](README_ja.md)

A self-hosted, learning-focused quiz application. Built with Rust, HTMX, and libSQL.

Forked from [frectonz/quizzy](https://github.com/frectonz/quizzy) and extended with features for effective study.

## Features

- **Smart question selection** — choose from unanswered, previously incorrect, or random questions
- **Session resume** — pick up where you left off by name, across devices
- **Per-option explanations** — every answer choice can have a detailed explanation
- **Category statistics** — track accuracy by category on the dashboard
- **Retry incorrect** — instantly create a new session from missed questions
- **Multi-device** — responsive UI works on desktop and mobile
- **No account required** — just enter a name to start

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Backend | Rust, Warp, Maud |
| Frontend | HTMX, PicoCSS |
| Database | libSQL (SQLite-compatible, Turso for remote) |

## Quick Start

### Prerequisites

- **Rust** 1.70+ — [rustup.rs](https://rustup.rs/)

### Run locally

```bash
cd quizinart

# Start with local SQLite (no Turso account needed)
URL="file:local.db" AUTH_TOKEN="" ADDRESS="127.0.0.1:1414" RUST_LOG=info cargo run
```

Open http://127.0.0.1:1414 in your browser.

### First-time setup

1. Set an admin password on first visit
2. Click **Create Quiz**, name it, and upload a quiz JSON file
3. Go to the quiz page, enter your name, and start

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
│   ├── main.rs                # Entry point, routes, auth
│   ├── db/                    # Database layer
│   │   ├── schema.rs          # Table definitions
│   │   ├── session.rs         # Session CRUD
│   │   ├── question.rs        # Questions & stats
│   │   ├── answer.rs          # Answer recording
│   │   ├── admin.rs           # Admin auth
│   │   ├── quiz.rs            # Quiz management
│   │   └── report.rs          # Reporting
│   ├── handlers/              # HTTP handlers
│   │   ├── quiz.rs            # Quiz flow logic
│   │   └── homepage.rs        # Landing page
│   ├── views/                 # Maud HTML templates
│   │   ├── layout.rs          # Page shell
│   │   ├── quiz.rs            # Quiz views
│   │   └── homepage.rs        # Home views
│   ├── names.rs               # Route & cookie constants
│   ├── utils.rs               # Helpers
│   └── statics.rs             # Static file serving
├── static/                    # CSS, JS, images
└── Cargo.toml
```

## Deployment

### Environment Variables

| Variable | Description | Example |
|----------|-------------|---------|
| `URL` | libSQL connection string | `file:local.db` or `libsql://your-db.turso.io` |
| `AUTH_TOKEN` | Turso auth token (empty for local) | `eyJhbGc...` |
| `ADDRESS` | Listen address | `127.0.0.1:1414` |
| `RUST_LOG` | Log level | `info` |

### Deploy to Render.com

1. Push to GitHub
2. Create a **Web Service** on [render.com](https://render.com)
3. Set environment: **Docker**, root directory: `quizinart`
4. Add environment variables (`URL`, `AUTH_TOKEN`, `ADDRESS=0.0.0.0:1414`)

### Using Turso (remote database)

```bash
turso db create my-quiz
turso db show my-quiz       # get the URL
turso db tokens create my-quiz  # get the token
```

## License

MIT

## Credits

- Quiz engine based on [Quizzy](https://github.com/frectonz/quizzy) by frectonz
- Built with Claude Code
