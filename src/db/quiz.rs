use color_eyre::{eyre::OptionExt, Result};
use ulid::Ulid;

use super::models::Quiz;
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
        let quiz_id: i32 = sqlx::query_scalar(
            "INSERT INTO quizzes (name, owner_id, public_id) VALUES ($1, $2, $3) RETURNING id",
        )
        .bind(&quiz_name)
        .bind(user_id)
        .bind(&public_id)
        .fetch_one(&mut *tx)
        .await?;

        // 1b. Add to user's library
        sqlx::query("INSERT INTO user_quizzes (user_id, quiz_id) VALUES ($1, $2)")
            .bind(user_id)
            .bind(quiz_id)
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

        sqlx::query(
            r#"
            INSERT INTO questions (question, category, is_multiple_choice, quiz_id)
            SELECT * FROM UNNEST($1::TEXT[], $2::TEXT[], $3::BOOL[], $4::INT4[])
            "#,
        )
        .bind(&q_texts)
        .bind(&q_categories)
        .bind(&q_multiple)
        .bind(&q_quiz_ids)
        .execute(&mut *tx)
        .await?;

        // 3. Retrieve question IDs in insertion order
        let question_ids: Vec<i32> =
            sqlx::query_scalar("SELECT id FROM questions WHERE quiz_id = $1 ORDER BY id")
                .bind(quiz_id)
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
            sqlx::query(
                r#"
                INSERT INTO options (option, is_answer, explanation, question_id)
                SELECT * FROM UNNEST($1::TEXT[], $2::BOOL[], $3::TEXT[], $4::INT4[])
                "#,
            )
            .bind(&o_texts)
            .bind(&o_is_answers)
            .bind(&o_explanations)
            .bind(&o_question_ids)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;

        tracing::info!("new quiz created with id: {quiz_id} for user_id: {user_id}");
        Ok(public_id)
    }

    pub async fn quizzes(&self, user_id: i32) -> Result<Vec<Quiz>> {
        let quizzes = sqlx::query_as::<_, Quiz>(
            r#"
            SELECT
              quizzes.id AS id,
              quizzes.public_id AS public_id,
              quizzes.name AS name,
              COUNT(DISTINCT questions.id) AS count,
              MAX(qs.id) AS last_session_id
            FROM
              user_quizzes
              JOIN quizzes ON quizzes.id = user_quizzes.quiz_id
              JOIN questions ON questions.quiz_id = quizzes.id
              LEFT JOIN quiz_sessions qs ON qs.quiz_id = quizzes.id AND qs.user_id = $1
            WHERE
              user_quizzes.user_id = $1
            GROUP BY
              quizzes.id, quizzes.public_id, quizzes.name
            ORDER BY
              last_session_id DESC NULLS LAST,
              quizzes.id DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(quizzes)
    }

    pub async fn quiz_has_other_users(&self, public_id: &str, owner_id: i32) -> Result<bool> {
        let exists: bool = sqlx::query_scalar(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM user_quizzes uq
                JOIN quizzes q ON q.id = uq.quiz_id
                WHERE q.public_id = $1 AND uq.user_id != $2
            )
            "#,
        )
        .bind(public_id)
        .bind(owner_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(exists)
    }

    pub async fn delete_quiz(&self, public_id: &str, user_id: i32) -> Result<()> {
        sqlx::query("DELETE FROM quizzes WHERE public_id = $1 AND owner_id = $2")
            .bind(public_id)
            .bind(user_id)
            .execute(&self.pool)
            .await?;

        tracing::info!("quiz deleted with public_id: {public_id} by user_id: {user_id}");
        Ok(())
    }

    pub async fn quiz_name(&self, quiz_id: i32) -> Result<String> {
        let name: String = sqlx::query_scalar("SELECT name FROM quizzes WHERE id = $1")
            .bind(quiz_id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or_eyre("could not get quiz name")?;

        Ok(name)
    }

    /// Resolve a public_id (ULID) to the internal quiz id.
    pub async fn resolve_quiz_id(&self, public_id: &str) -> Result<i32> {
        let id: i32 = sqlx::query_scalar("SELECT id FROM quizzes WHERE public_id = $1")
            .bind(public_id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or_eyre("quiz not found")?;

        Ok(id)
    }

    /// Look up the public_id for a quiz given its internal id.
    pub async fn quiz_public_id(&self, quiz_id: i32) -> Result<String> {
        let public_id: String = sqlx::query_scalar("SELECT public_id FROM quizzes WHERE id = $1")
            .bind(quiz_id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or_eyre("quiz not found")?;

        Ok(public_id)
    }

    pub async fn rename_quiz(&self, public_id: &str, name: &str, user_id: i32) -> Result<()> {
        sqlx::query("UPDATE quizzes SET name = $1 WHERE public_id = $2 AND owner_id = $3")
            .bind(name)
            .bind(public_id)
            .bind(user_id)
            .execute(&self.pool)
            .await?;

        tracing::info!("quiz renamed with public_id: {public_id} by user_id: {user_id}");
        Ok(())
    }

    /// Verify that a quiz belongs to the given user (owner check)
    pub async fn verify_quiz_owner(&self, public_id: &str, user_id: i32) -> Result<bool> {
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM quizzes WHERE public_id = $1 AND owner_id = $2)",
        )
        .bind(public_id)
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(exists)
    }
}
