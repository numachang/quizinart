use quizinart::db::Db;

pub async fn create_test_db() -> Db {
    use std::sync::atomic::{AtomicU32, Ordering};
    static COUNTER: AtomicU32 = AtomicU32::new(0);
    let id = COUNTER.fetch_add(1, Ordering::SeqCst);

    dotenvy::dotenv().ok();
    let base_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgres://quizinart:password@localhost:5432/quizinart".to_string()
    });

    // Use a unique schema per test to isolate state
    let schema = format!("test_{}_{}", std::process::id(), id);
    let url = format!("{}?options=-c%20search_path%3D{}", base_url, schema);

    // Create the schema first using a plain connection
    let admin_pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(1)
        .connect(&base_url)
        .await
        .expect("connect to postgres for schema setup");
    sqlx::query(&format!("CREATE SCHEMA IF NOT EXISTS {schema}"))
        .execute(&admin_pool)
        .await
        .expect("create test schema");
    drop(admin_pool);

    Db::new(url).await.expect("failed to create test database")
}
