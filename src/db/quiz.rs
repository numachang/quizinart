use color_eyre::{eyre::OptionExt, Result};
use libsql::params;

use super::helpers;
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
        let conn = self.db.connect()?;

        // 1. Insert quiz (1 round-trip)
        let quiz_id = conn
            .query(
                "INSERT INTO quizzes (name, user_id) VALUES (?, ?) RETURNING id",
                params![quiz_name, user_id],
            )
            .await?
            .next()
            .await?
            .ok_or_eyre("could not get quiz id")?
            .get::<i32>(0)?;

        if questions.is_empty() {
            tracing::info!("new quiz created with id: {quiz_id} for user_id: {user_id}");
            return Ok(quiz_id);
        }

        // 2. Batch INSERT all questions (1 round-trip)
        let mut q_placeholders = Vec::with_capacity(questions.len());
        let mut q_params: Vec<libsql::Value> = Vec::with_capacity(questions.len() * 4);

        for q in &questions {
            q_placeholders.push("(?, ?, ?, ?)");
            q_params.push(libsql::Value::from(q.question.clone()));
            q_params.push(
                q.category
                    .as_ref()
                    .map(|c| libsql::Value::from(c.clone()))
                    .unwrap_or(libsql::Value::Null),
            );
            q_params.push(libsql::Value::from(q.is_multiple_choice));
            q_params.push(libsql::Value::from(quiz_id));
        }

        let q_sql = format!(
            "INSERT INTO questions (question, category, is_multiple_choice, quiz_id) VALUES {}",
            q_placeholders.join(", ")
        );
        conn.execute(&q_sql, q_params).await?;

        // 3. Retrieve question IDs in insertion order (1 round-trip)
        let mut rows = conn
            .query(
                "SELECT id FROM questions WHERE quiz_id = ? ORDER BY id",
                params![quiz_id],
            )
            .await?;
        let mut question_ids = Vec::with_capacity(questions.len());
        while let Some(row) = rows.next().await? {
            question_ids.push(row.get::<i32>(0)?);
        }

        // 4. Batch INSERT all options (1 round-trip)
        let mut o_placeholders = Vec::new();
        let mut o_params: Vec<libsql::Value> = Vec::new();

        for (q, &q_id) in questions.iter().zip(question_ids.iter()) {
            for opt in &q.options {
                o_placeholders.push("(?, ?, ?, ?)");
                o_params.push(libsql::Value::from(opt.text.clone()));
                o_params.push(libsql::Value::from(opt.is_answer));
                o_params.push(
                    opt.explanation
                        .as_ref()
                        .map(|e| libsql::Value::from(e.clone()))
                        .unwrap_or(libsql::Value::Null),
                );
                o_params.push(libsql::Value::from(q_id));
            }
        }

        if !o_placeholders.is_empty() {
            let o_sql = format!(
                "INSERT INTO options (option, is_answer, explanation, question_id) VALUES {}",
                o_placeholders.join(", ")
            );
            conn.execute(&o_sql, o_params).await?;
        }

        tracing::info!("new quiz created with id: {quiz_id} for user_id: {user_id}");
        Ok(quiz_id)
    }

    pub async fn quizzes(&self, user_id: i32) -> Result<Vec<Quiz>> {
        let conn = self.db.connect()?;
        helpers::query_all(
            &conn,
            r#"
            SELECT
              quizzes.id AS id,
              quizzes.name AS name,
              COUNT(questions.id) AS count
            FROM
              quizzes
              JOIN questions ON questions.quiz_id = quizzes.id
            WHERE
              quizzes.user_id = ?
            GROUP BY
              quizzes.name
            "#,
            params![user_id],
        )
        .await
    }

    pub async fn delete_quiz(&self, quiz_id: i32, user_id: i32) -> Result<()> {
        let conn = self.db.connect()?;

        conn.execute(
            "DELETE FROM quizzes WHERE id = ? AND user_id = ?",
            params![quiz_id, user_id],
        )
        .await?;

        tracing::info!("quiz deleted with id: {quiz_id} by user_id: {user_id}");
        Ok(())
    }

    pub async fn quiz_name(&self, quiz_id: i32) -> Result<String> {
        let conn = self.db.connect()?;

        let quiz_name = conn
            .query("SELECT name FROM quizzes WHERE id = ?", params![quiz_id])
            .await?
            .next()
            .await?
            .ok_or_eyre("could not get quiz name")?
            .get::<String>(0)?;

        Ok(quiz_name)
    }

    /// Verify that a quiz belongs to the given user
    pub async fn verify_quiz_owner(&self, quiz_id: i32, user_id: i32) -> Result<bool> {
        let conn = self.db.connect()?;
        let row = conn
            .query(
                "SELECT 1 FROM quizzes WHERE id = ? AND user_id = ?",
                params![quiz_id, user_id],
            )
            .await?
            .next()
            .await?;
        Ok(row.is_some())
    }
}
