use color_eyre::Result;
use ulid::Ulid;

use super::models::AdminUserStats;
use super::Db;

impl Db {
    pub async fn admin_password(&self) -> Result<Option<String>> {
        let password: Option<String> =
            sqlx::query_scalar!("SELECT password FROM admin WHERE id = 1")
                .fetch_optional(&self.pool)
                .await?;

        Ok(password)
    }

    pub async fn set_admin_password(&self, password: String) -> Result<()> {
        sqlx::query!(
            "INSERT INTO admin (id, password) VALUES (1, $1) ON CONFLICT(id) DO UPDATE SET password = EXCLUDED.password",
            password
        )
        .execute(&self.pool)
        .await?;

        tracing::info!("new admin password set");
        Ok(())
    }

    pub async fn create_admin_session(&self) -> Result<String> {
        let session = Ulid::new().to_string();

        sqlx::query!("INSERT INTO admin_sessions (id) VALUES ($1)", session)
            .execute(&self.pool)
            .await?;

        tracing::info!("new admin session {session:?} created");
        Ok(session)
    }

    pub async fn admin_session_exists(&self, session: String) -> Result<bool> {
        let exists: bool = sqlx::query_scalar!(
            "SELECT EXISTS(SELECT 1 FROM admin_sessions WHERE id = $1)",
            session
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(false);

        tracing::info!("admin session {session:?} exists: {exists}");
        Ok(exists)
    }

    /// Get all users with their learning statistics for the admin dashboard.
    pub async fn get_all_users_with_stats(&self) -> Result<Vec<AdminUserStats>> {
        let stats = sqlx::query_as!(
            AdminUserStats,
            r#"
            SELECT
                u.id,
                u.display_name,
                (SELECT COUNT(*) FROM user_quizzes uq WHERE uq.user_id = u.id) AS "quiz_count!",
                (SELECT COUNT(DISTINCT sq.question_id)
                 FROM session_questions sq
                 JOIN quiz_sessions qs ON qs.id = sq.session_id
                 WHERE qs.user_id = u.id AND sq.is_correct IS NOT NULL
                ) AS "unique_asked!",
                (SELECT COUNT(*)
                 FROM questions q
                 JOIN user_quizzes uq2 ON q.quiz_id = uq2.quiz_id
                 WHERE uq2.user_id = u.id
                ) AS "total_questions!",
                COALESCE((
                    SELECT SUM(ua.duration_ms)::BIGINT
                    FROM user_answers ua
                    JOIN quiz_sessions qs2 ON qs2.id = ua.session_id
                    WHERE qs2.user_id = u.id
                ), 0) AS "total_study_time_ms!"
            FROM users u
            ORDER BY u.id
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(stats)
    }
}
