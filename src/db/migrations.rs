use color_eyre::Result;

pub async fn run(pool: &sqlx::PgPool) -> Result<()> {
    sqlx::migrate!("./migrations").run(pool).await?;
    tracing::info!("database migrations applied successfully");
    Ok(())
}
