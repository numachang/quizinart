use color_eyre::{eyre::OptionExt, Result};
use sqlx::Row;

use super::models::{
    QuestionModel, QuestionOptionModel, QuizCategoryOverallStats, QuizOverallStats,
};
use super::Db;

impl Db {
    pub async fn get_question(&self, question_id: i32) -> Result<QuestionModel> {
        let row = sqlx::query("SELECT question, is_multiple_choice FROM questions WHERE id = $1")
            .bind(question_id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or_eyre("could not get question")?;

        let question: String = row.get("question");
        let is_multiple_choice: bool = row.get("is_multiple_choice");

        let options: Vec<QuestionOptionModel> = sqlx::query_as::<_, QuestionOptionModel>(
            "SELECT id, is_answer, option, explanation FROM options WHERE question_id = $1",
        )
        .bind(question_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(QuestionModel {
            question,
            is_multiple_choice,
            options,
        })
    }

    pub async fn question_id_from_idx(&self, quiz_id: i32, question_idx: i32) -> Result<i32> {
        let question_id: i32 =
            sqlx::query_scalar("SELECT id FROM questions WHERE quiz_id = $1 LIMIT 1 OFFSET $2")
                .bind(quiz_id)
                .bind(question_idx)
                .fetch_optional(&self.pool)
                .await?
                .ok_or_eyre("no question id found")?;

        Ok(question_id)
    }

    pub async fn questions_count(&self, quiz_id: i32) -> Result<i32> {
        let count: i32 =
            sqlx::query_scalar("SELECT COUNT(*)::INT FROM questions WHERE quiz_id = $1")
                .bind(quiz_id)
                .fetch_one(&self.pool)
                .await?;

        Ok(count)
    }

    pub async fn get_question_by_idx(&self, session_id: i32, idx: i32) -> Result<i32> {
        let question_id: i32 = sqlx::query_scalar(
            "SELECT question_id FROM session_questions WHERE session_id = $1 AND question_number = $2",
        )
        .bind(session_id)
        .bind(idx)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_eyre("no question id found")?;

        Ok(question_id)
    }

    pub async fn get_available_categories(&self, quiz_id: i32) -> Result<Vec<String>> {
        let categories: Vec<String> = sqlx::query_scalar(
            "SELECT DISTINCT category FROM questions WHERE quiz_id = $1 AND category IS NOT NULL ORDER BY category",
        )
        .bind(quiz_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(categories)
    }

    pub async fn questions_count_for_session(&self, session_id: i32) -> Result<i32> {
        let count: i32 =
            sqlx::query_scalar("SELECT COUNT(*)::INT FROM session_questions WHERE session_id = $1")
                .bind(session_id)
                .fetch_one(&self.pool)
                .await?;

        Ok(count)
    }

    pub async fn get_quiz_overall_stats(&self, quiz_id: i32) -> Result<QuizOverallStats> {
        let stats = sqlx::query_as::<_, QuizOverallStats>(
            r#"
            SELECT
                (SELECT COUNT(*) FROM questions WHERE quiz_id = $1) AS total_questions,
                COUNT(DISTINCT sq.question_id) AS unique_asked,
                COALESCE(SUM(CASE WHEN sq.is_correct THEN 1 ELSE 0 END), 0) AS total_correct,
                COUNT(*) AS total_answered
            FROM session_questions sq
            JOIN quiz_sessions s ON s.id = sq.session_id
            WHERE s.quiz_id = $1 AND sq.is_correct IS NOT NULL
            "#,
        )
        .bind(quiz_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(stats)
    }

    pub async fn get_quiz_category_stats(
        &self,
        quiz_id: i32,
    ) -> Result<Vec<QuizCategoryOverallStats>> {
        let stats = sqlx::query_as::<_, QuizCategoryOverallStats>(
            r#"
            SELECT
                q.category AS category,
                COUNT(DISTINCT q.id) AS total_in_category,
                COUNT(DISTINCT CASE WHEN sq.is_correct IS NOT NULL THEN sq.question_id END) AS unique_asked,
                COALESCE(SUM(CASE WHEN sq.is_correct THEN 1 ELSE 0 END), 0) AS total_correct,
                COUNT(CASE WHEN sq.is_correct IS NOT NULL THEN 1 END) AS total_answered
            FROM questions q
            LEFT JOIN session_questions sq ON sq.question_id = q.id
            WHERE q.quiz_id = $1 AND q.category IS NOT NULL
            GROUP BY q.category
            ORDER BY q.category
            "#,
        )
        .bind(quiz_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(stats)
    }

    pub async fn get_correct_option_ids(&self, question_id: i32) -> Result<Vec<i32>> {
        let ids: Vec<i32> = sqlx::query_scalar(
            "SELECT id FROM options WHERE question_id = $1 AND is_answer = TRUE",
        )
        .bind(question_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(ids)
    }
}
