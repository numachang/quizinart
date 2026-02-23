use color_eyre::{eyre::OptionExt, Result};
use ulid::Ulid;

use super::models::{Quiz, SharedQuizInfo};
use super::Db;
use crate::models::Questions;

impl Db {
    /// Insert a quiz with all its questions and options atomically in a transaction.
    /// Uses UNNEST batch inserts to avoid N+1 round-trips.
    /// Returns the public_id (ULID) of the newly created quiz.
    pub async fn load_quiz(
        &self,
        quiz_name: String,
        questions: Questions,
        user_id: i32,
    ) -> Result<String> {
        let public_id = Ulid::new().to_string();
        let mut tx = self.pool.begin().await?;

        // 1. Insert quiz with owner_id and public_id
        let quiz_id: i32 = sqlx::query_scalar!(
            "INSERT INTO quizzes (name, owner_id, public_id) VALUES ($1, $2, $3) RETURNING id",
            quiz_name,
            user_id,
            public_id
        )
        .fetch_one(&mut *tx)
        .await?;

        // 1b. Add to user's library
        sqlx::query!(
            "INSERT INTO user_quizzes (user_id, quiz_id) VALUES ($1, $2)",
            user_id,
            quiz_id
        )
        .execute(&mut *tx)
        .await?;

        if questions.is_empty() {
            tx.commit().await?;
            tracing::info!("new quiz created with id: {quiz_id} for user_id: {user_id}");
            return Ok(public_id);
        }

        // 2. Batch INSERT all questions via UNNEST
        let q_texts: Vec<String> = questions.iter().map(|q| q.question.clone()).collect();
        let q_categories: Vec<Option<String>> =
            questions.iter().map(|q| q.category.clone()).collect();
        let q_multiple: Vec<bool> = questions.iter().map(|q| q.is_multiple_choice).collect();
        let q_quiz_ids: Vec<i32> = vec![quiz_id; questions.len()];

        sqlx::query!(
            r#"
            INSERT INTO questions (question, category, is_multiple_choice, quiz_id)
            SELECT * FROM UNNEST($1::TEXT[], $2::TEXT[], $3::BOOL[], $4::INT4[])
            "#,
            &q_texts,
            &q_categories as &[Option<String>],
            &q_multiple,
            &q_quiz_ids
        )
        .execute(&mut *tx)
        .await?;

        // 3. Retrieve question IDs in insertion order
        let question_ids: Vec<i32> = sqlx::query_scalar!(
            "SELECT id FROM questions WHERE quiz_id = $1 ORDER BY id",
            quiz_id
        )
        .fetch_all(&mut *tx)
        .await?;

        // 4. Batch INSERT all options via UNNEST
        let mut o_texts = Vec::new();
        let mut o_is_answers = Vec::new();
        let mut o_explanations: Vec<Option<String>> = Vec::new();
        let mut o_question_ids = Vec::new();

        for (q, &q_id) in questions.iter().zip(question_ids.iter()) {
            for opt in &q.options {
                o_texts.push(opt.text.clone());
                o_is_answers.push(opt.is_answer);
                o_explanations.push(opt.explanation.clone());
                o_question_ids.push(q_id);
            }
        }

        if !o_texts.is_empty() {
            sqlx::query!(
                r#"
                INSERT INTO options (option, is_answer, explanation, question_id)
                SELECT * FROM UNNEST($1::TEXT[], $2::BOOL[], $3::TEXT[], $4::INT4[])
                "#,
                &o_texts,
                &o_is_answers,
                &o_explanations as &[Option<String>],
                &o_question_ids
            )
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;

        tracing::info!("new quiz created with id: {quiz_id} for user_id: {user_id}");
        Ok(public_id)
    }

    pub async fn quizzes(&self, user_id: i32) -> Result<Vec<Quiz>> {
        let quizzes = sqlx::query_as!(
            Quiz,
            r#"
            SELECT
              quizzes.id AS id,
              quizzes.public_id AS "public_id!",
              quizzes.name AS name,
              COUNT(DISTINCT questions.id) AS "count!",
              MAX(qs.id) AS last_session_id,
              quizzes.is_shared AS "is_shared!",
              (quizzes.owner_id = $1) AS "is_owner!"
            FROM
              user_quizzes
              JOIN quizzes ON quizzes.id = user_quizzes.quiz_id
              JOIN questions ON questions.quiz_id = quizzes.id
              LEFT JOIN quiz_sessions qs ON qs.quiz_id = quizzes.id AND qs.user_id = $1
            WHERE
              user_quizzes.user_id = $1
            GROUP BY
              quizzes.id, quizzes.public_id, quizzes.name, quizzes.is_shared, quizzes.owner_id
            ORDER BY
              last_session_id DESC NULLS LAST,
              quizzes.id DESC
            "#,
            user_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(quizzes)
    }

    pub async fn quiz_has_other_users(&self, public_id: &str, owner_id: i32) -> Result<bool> {
        let exists: bool = sqlx::query_scalar!(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM user_quizzes uq
                JOIN quizzes q ON q.id = uq.quiz_id
                WHERE q.public_id = $1 AND uq.user_id != $2
            )
            "#,
            public_id,
            owner_id
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(false);

        Ok(exists)
    }

    pub async fn delete_quiz(&self, public_id: &str, user_id: i32) -> Result<()> {
        sqlx::query!(
            "DELETE FROM quizzes WHERE public_id = $1 AND owner_id = $2",
            public_id,
            user_id
        )
        .execute(&self.pool)
        .await?;

        tracing::info!("quiz deleted with public_id: {public_id} by user_id: {user_id}");
        Ok(())
    }

    pub async fn quiz_name(&self, quiz_id: i32) -> Result<String> {
        let name: String = sqlx::query_scalar!("SELECT name FROM quizzes WHERE id = $1", quiz_id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or_eyre("could not get quiz name")?;

        Ok(name)
    }

    /// Resolve a public_id (ULID) to the internal quiz id.
    pub async fn resolve_quiz_id(&self, public_id: &str) -> Result<i32> {
        let id: i32 = sqlx::query_scalar!("SELECT id FROM quizzes WHERE public_id = $1", public_id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or_eyre("quiz not found")?;

        Ok(id)
    }

    /// Look up the public_id for a quiz given its internal id.
    pub async fn quiz_public_id(&self, quiz_id: i32) -> Result<String> {
        let public_id: String = sqlx::query_scalar!(
            r#"SELECT public_id AS "public_id!" FROM quizzes WHERE id = $1"#,
            quiz_id
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or_eyre("quiz not found")?;

        Ok(public_id)
    }

    pub async fn rename_quiz(&self, public_id: &str, name: &str, user_id: i32) -> Result<()> {
        sqlx::query!(
            "UPDATE quizzes SET name = $1 WHERE public_id = $2 AND owner_id = $3",
            name,
            public_id,
            user_id
        )
        .execute(&self.pool)
        .await?;

        tracing::info!("quiz renamed with public_id: {public_id} by user_id: {user_id}");
        Ok(())
    }

    /// Verify that a quiz belongs to the given user (owner check)
    pub async fn verify_quiz_owner(&self, public_id: &str, user_id: i32) -> Result<bool> {
        let exists: bool = sqlx::query_scalar!(
            "SELECT EXISTS(SELECT 1 FROM quizzes WHERE public_id = $1 AND owner_id = $2)",
            public_id,
            user_id
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(false);

        Ok(exists)
    }

    /// Toggle the is_shared flag for a quiz owned by the given user.
    /// Returns the new value of is_shared.
    pub async fn toggle_share(&self, public_id: &str, user_id: i32) -> Result<bool> {
        let is_shared: bool = sqlx::query_scalar!(
            r#"UPDATE quizzes SET is_shared = NOT is_shared
               WHERE public_id = $1 AND owner_id = $2
               RETURNING is_shared AS "is_shared!""#,
            public_id,
            user_id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(is_shared)
    }

    /// Check if a quiz is shared by its internal id.
    pub async fn is_quiz_shared_by_id(&self, quiz_id: i32) -> Result<bool> {
        let shared: bool =
            sqlx::query_scalar!("SELECT is_shared FROM quizzes WHERE id = $1", quiz_id)
                .fetch_optional(&self.pool)
                .await?
                .unwrap_or(false);

        Ok(shared)
    }

    /// Get shared quiz info by public_id (for the shared quiz page).
    pub async fn get_shared_quiz(&self, public_id: &str) -> Result<Option<SharedQuizInfo>> {
        let row = sqlx::query_as!(
            SharedQuizInfo,
            r#"
            SELECT
                q.id,
                q.public_id AS "public_id!",
                q.name,
                q.is_shared,
                u.display_name AS owner_name,
                COUNT(qu.id) AS "question_count!"
            FROM quizzes q
            JOIN users u ON u.id = q.owner_id
            JOIN questions qu ON qu.quiz_id = q.id
            WHERE q.public_id = $1
            GROUP BY q.id, q.public_id, q.name, q.is_shared, u.display_name
            "#,
            public_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row)
    }

    /// Check if a quiz is already in the user's library.
    pub async fn user_has_quiz(&self, user_id: i32, quiz_id: i32) -> Result<bool> {
        let exists: bool = sqlx::query_scalar!(
            "SELECT EXISTS(SELECT 1 FROM user_quizzes WHERE user_id = $1 AND quiz_id = $2)",
            user_id,
            quiz_id
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(false);

        Ok(exists)
    }

    /// Add a quiz to the user's library (idempotent).
    pub async fn add_quiz_to_library(&self, user_id: i32, quiz_id: i32) -> Result<()> {
        sqlx::query!(
            "INSERT INTO user_quizzes (user_id, quiz_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
            user_id,
            quiz_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// List all shared quizzes with owner info and question count.
    pub async fn list_shared_quizzes(&self) -> Result<Vec<SharedQuizInfo>> {
        let rows = sqlx::query_as!(
            SharedQuizInfo,
            r#"
            SELECT
                q.id,
                q.public_id AS "public_id!",
                q.name,
                q.is_shared,
                u.display_name AS owner_name,
                COUNT(qu.id) AS "question_count!"
            FROM quizzes q
            JOIN users u ON u.id = q.owner_id
            JOIN questions qu ON qu.quiz_id = q.id
            WHERE q.is_shared = true
            GROUP BY q.id, q.public_id, q.name, q.is_shared, u.display_name
            ORDER BY q.id DESC
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    /// Get all quiz IDs in a user's library.
    pub async fn user_quiz_ids(&self, user_id: i32) -> Result<Vec<i32>> {
        let ids = sqlx::query_scalar!(
            "SELECT quiz_id FROM user_quizzes WHERE user_id = $1",
            user_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(ids)
    }

    /// Search shared quizzes by name (ILIKE).
    pub async fn search_shared_quizzes(&self, query: &str) -> Result<Vec<SharedQuizInfo>> {
        let pattern = format!("%{query}%");
        let rows = sqlx::query_as!(
            SharedQuizInfo,
            r#"
            SELECT
                q.id,
                q.public_id AS "public_id!",
                q.name,
                q.is_shared,
                u.display_name AS owner_name,
                COUNT(qu.id) AS "question_count!"
            FROM quizzes q
            JOIN users u ON u.id = q.owner_id
            JOIN questions qu ON qu.quiz_id = q.id
            WHERE q.is_shared = true
              AND q.name ILIKE $1
            GROUP BY q.id, q.public_id, q.name, q.is_shared, u.display_name
            ORDER BY q.id DESC
            "#,
            pattern
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }
}
