use color_eyre::Result;

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

pub async fn run(pool: &sqlx::PgPool) -> Result<()> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS schema_migrations (
            version TEXT PRIMARY KEY,
            applied_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
        )
        "#,
    )
    .execute(pool)
    .await?;

    for migration in MIGRATIONS {
        let already_applied: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM schema_migrations WHERE version = $1)"
        )
        .bind(migration.version)
        .fetch_one(pool)
        .await?;

        if already_applied {
            continue;
        }

        // Execute multi-statement SQL using raw_sql
        sqlx::raw_sql(migration.sql).execute(pool).await?;

        sqlx::query("INSERT INTO schema_migrations (version) VALUES ($1)")
            .bind(migration.version)
            .execute(pool)
            .await?;

        tracing::info!(version = migration.version, "applied database migration");
    }

    Ok(())
}
