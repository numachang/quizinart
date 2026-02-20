use color_eyre::Result;
use libsql::params;

struct Migration {
    version: &'static str,
    sql: &'static str,
}

const MIGRATIONS: &[Migration] = &[
    Migration {
        version: "V1",
        sql: include_str!("../../migrations/V1__init.sql"),
    },
    Migration {
        version: "V2",
        sql: include_str!("../../migrations/V2__add_bookmarks.sql"),
    },
    Migration {
        version: "V3",
        sql: include_str!("../../migrations/V3__add_safe_integrity_guards.sql"),
    },
    Migration {
        version: "V4",
        sql: include_str!("../../migrations/V4__add_users.sql"),
    },
    Migration {
        version: "V5",
        sql: include_str!("../../migrations/V5__add_email_verification.sql"),
    },
    Migration {
        version: "V6",
        sql: include_str!("../../migrations/V6__add_password_reset.sql"),
    },
];

pub async fn run(conn: &libsql::Connection) -> Result<()> {
    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS schema_migrations (
            version TEXT PRIMARY KEY,
            applied_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        )
        "#,
        (),
    )
    .await?;

    for migration in MIGRATIONS {
        let already_applied = conn
            .query(
                "SELECT version FROM schema_migrations WHERE version = ?",
                params![migration.version],
            )
            .await?
            .next()
            .await?
            .is_some();

        if already_applied {
            continue;
        }

        conn.execute_batch(migration.sql).await?;
        conn.execute(
            "INSERT INTO schema_migrations (version) VALUES (?)",
            params![migration.version],
        )
        .await?;

        tracing::info!(version = migration.version, "applied database migration");
    }

    Ok(())
}
