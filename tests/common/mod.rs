use quizinart::db::Db;

pub async fn create_test_db() -> Db {
    use std::sync::atomic::{AtomicU32, Ordering};
    static COUNTER: AtomicU32 = AtomicU32::new(0);
    let id = COUNTER.fetch_add(1, Ordering::SeqCst);
    let path =
        std::env::temp_dir().join(format!("quizinart_test_{}_{}.db", std::process::id(), id));
    // Clean up leftover file from previous runs
    let _ = std::fs::remove_file(&path);
    let url = format!("file:{}", path.display());
    Db::new(url).await.expect("failed to create test database")
}
