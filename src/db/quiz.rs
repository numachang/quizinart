use color_eyre::{eyre::OptionExt, Result};

use super::models::Quiz;
use super::Db;
use crate::models::Questions;

impl Db {
    /// Insert a quiz with all its questions and options in batch to avoid N+1 round-trips.
    pub async fn load_quiz(
        &self,
        quiz_name: String,
        questions: Questions,
        user_id: i32,
    ) -> Result<i32> {
        // 1. Insert quiz (1 round-trip)
        let quiz_id: i32 =
            sqlx::query_scalar("INSERT INTO quizzes (name, user_id) VALUES ($1, $2) RETURNING id")
                .bind(&quiz_name)
                .bind(user_id)
                .fetch_one(&self.pool)
                .await?;

        if questions.is_empty() {
            tracing::info!("new quiz created with id: {quiz_id} for user_id: {user_id}");
            return Ok(quiz_id);
        }

        // 2. Batch INSERT all questions via UNNEST (1 round-trip)
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
        .execute(&self.pool)
        .await?;

        // 3. Retrieve question IDs in insertion order (1 round-trip)
        let question_ids: Vec<i32> =
            sqlx::query_scalar("SELECT id FROM questions WHERE quiz_id = $1 ORDER BY id")
                .bind(quiz_id)
                .fetch_all(&self.pool)
                .await?;

        // 4. Batch INSERT all options via UNNEST (1 round-trip)
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
            .execute(&self.pool)
            .await?;
        }

        tracing::info!("new quiz created with id: {quiz_id} for user_id: {user_id}");
        Ok(quiz_id)
    }

    pub async fn quizzes(&self, user_id: i32) -> Result<Vec<Quiz>> {
        let quizzes = sqlx::query_as::<_, Quiz>(
            r#"
            SELECT
              quizzes.id AS id,
              quizzes.name AS name,
              COUNT(questions.id) AS count
            FROM
              quizzes
              JOIN questions ON questions.quiz_id = quizzes.id
            WHERE
              quizzes.user_id = $1
            GROUP BY
              quizzes.id, quizzes.name
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(quizzes)
    }

    pub async fn delete_quiz(&self, quiz_id: i32, user_id: i32) -> Result<()> {
        sqlx::query("DELETE FROM quizzes WHERE id = $1 AND user_id = $2")
            .bind(quiz_id)
            .bind(user_id)
            .execute(&self.pool)
            .await?;

        tracing::info!("quiz deleted with id: {quiz_id} by user_id: {user_id}");
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

    /// Verify that a quiz belongs to the given user
    pub async fn verify_quiz_owner(&self, quiz_id: i32, user_id: i32) -> Result<bool> {
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM quizzes WHERE id = $1 AND user_id = $2)",
        )
        .bind(quiz_id)
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(exists)
    }
}
