use color_eyre::{eyre::OptionExt, Result};
use futures::{future, StreamExt, TryStreamExt};
use libsql::params;

use super::models::Quiz;
use super::Db;
use crate::models::{Question, Questions};

impl Db {
    pub async fn load_quiz(&self, quiz_name: String, questions: Questions) -> Result<i32> {
        let conn = self.db.connect()?;
        let quiz_id = conn
            .query(
                "INSERT INTO quizzes (name) VALUES (?) RETURNING id",
                params![quiz_name],
            )
            .await?
            .next()
            .await?
            .ok_or_eyre("could not get quiz id")?
            .get::<i32>(0)?;

        for Question {
            question,
            category,
            is_multiple_choice,
            options,
        } in questions
        {
            let question_id = conn
                .query(
                    "INSERT INTO questions (question, category, is_multiple_choice, quiz_id) VALUES (?, ?, ?, ?) RETURNING id",
                    params![question, category.as_deref(), is_multiple_choice, quiz_id],
                )
                .await?
                .next()
                .await?
                .ok_or_eyre("could not get question id")?
                .get::<i32>(0)?;

            for option in options {
                conn.execute(
                    "INSERT INTO options (option, is_answer, explanation, question_id) VALUES (?, ?, ?, ?)",
                    params![option.text, option.is_answer, option.explanation, question_id],
                )
                .await?;
            }
        }

        tracing::info!("new quiz created with id: {quiz_id}");
        Ok(quiz_id)
    }

    pub async fn quizzes(&self) -> Result<Vec<Quiz>> {
        let conn = self.db.connect()?;

        let quizzes = conn
            .query(
                r#"
        SELECT
          quizzes.id,
          quizzes.name,
          COUNT(questions.id) AS question_count
        FROM
          quizzes
          JOIN questions ON questions.quiz_id = quizzes.id
        GROUP BY
          quizzes.name
                "#,
                (),
            )
            .await?
            .into_stream()
            .map_ok(|r| Quiz {
                id: r.get::<i32>(0).expect("could not get quiz id"),
                name: r.get::<String>(1).expect("could not get quiz name"),
                count: r.get::<i32>(2).expect("could not get questions count"),
            })
            .filter_map(|r| future::ready(r.ok()))
            .collect::<Vec<_>>()
            .await;

        Ok(quizzes)
    }

    pub async fn delete_quiz(&self, quiz_id: i32) -> Result<()> {
        let conn = self.db.connect()?;

        conn.execute("DELETE FROM quizzes WHERE id = ?", params![quiz_id])
            .await?;

        tracing::info!("quiz deleted with id: {quiz_id}");
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
}
