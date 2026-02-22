use color_eyre::Result;

use super::models::{AnswerModel, CategoryStats};
use super::Db;

impl Db {
    pub async fn is_question_answered(&self, session_id: i32, question_id: i32) -> Result<bool> {
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM user_answers WHERE session_id = $1 AND question_id = $2)",
        )
        .bind(session_id)
        .bind(question_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(exists)
    }

    pub async fn get_selected_answers(
        &self,
        session_id: i32,
        question_id: i32,
    ) -> Result<Vec<i32>> {
        let option_ids: Vec<i32> = sqlx::query_scalar(
            "SELECT option_id FROM user_answers WHERE session_id = $1 AND question_id = $2",
        )
        .bind(session_id)
        .bind(question_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(option_ids)
    }

    pub async fn create_answer(
        &self,
        session_id: i32,
        question_id: i32,
        option_id: i32,
        is_correct: bool,
    ) -> Result<()> {
        let result = sqlx::query(
            "INSERT INTO user_answers (is_correct, option_id, question_id, session_id) VALUES ($1, $2, $3, $4)",
        )
        .bind(is_correct)
        .bind(option_id)
        .bind(question_id)
        .bind(session_id)
        .execute(&self.pool)
        .await?;

        tracing::info!(
            "answer created for session={session_id} question={question_id}: {:?}",
            result.rows_affected()
        );

        Ok(())
    }

    /// Batch insert answers for multiple selected options in a single round-trip using UNNEST.
    pub async fn create_answers_batch(
        &self,
        session_id: i32,
        question_id: i32,
        option_ids: &[i32],
        is_correct: bool,
    ) -> Result<()> {
        if option_ids.is_empty() {
            return Ok(());
        }

        let result = sqlx::query(
            r#"
            INSERT INTO user_answers (is_correct, option_id, question_id, session_id)
            SELECT $1, o, $3, $4
            FROM UNNEST($2::INT4[]) AS t(o)
            "#,
        )
        .bind(is_correct)
        .bind(option_ids)
        .bind(question_id)
        .bind(session_id)
        .execute(&self.pool)
        .await?;

        tracing::info!(
            "batch answers created for session={session_id} question={question_id}: {:?}",
            result.rows_affected()
        );

        Ok(())
    }

    /// session_questions の is_correct を更新
    pub async fn update_question_result(
        &self,
        session_id: i32,
        question_id: i32,
        is_correct: bool,
    ) -> Result<()> {
        sqlx::query(
            "UPDATE session_questions SET is_correct = $1 WHERE session_id = $2 AND question_id = $3",
        )
        .bind(is_correct)
        .bind(session_id)
        .bind(question_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// 正解数カウント（問題単位で正確）
    pub async fn correct_answers(&self, session_id: i32) -> Result<i32> {
        let count: i32 = sqlx::query_scalar(
            "SELECT COUNT(*)::INT FROM session_questions WHERE session_id = $1 AND is_correct = TRUE",
        )
        .bind(session_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(count)
    }

    pub async fn get_answers(&self, session_id: i32) -> Result<Vec<AnswerModel>> {
        let answers = sqlx::query_as::<_, AnswerModel>(
            r#"
            SELECT q.question AS question, sq.is_correct AS is_correct, sq.question_number AS question_idx,
                   sq.is_bookmarked AS is_bookmarked
            FROM session_questions sq
            JOIN questions q ON sq.question_id = q.id
            WHERE sq.session_id = $1 AND sq.is_correct IS NOT NULL
            ORDER BY sq.question_number
            "#,
        )
        .bind(session_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(answers)
    }

    pub async fn get_incorrect_questions(&self, session_id: i32) -> Result<Vec<i32>> {
        let ids: Vec<i32> = sqlx::query_scalar(
            "SELECT DISTINCT question_id FROM session_questions WHERE session_id = $1 AND is_correct = FALSE",
        )
        .bind(session_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(ids)
    }

    pub async fn get_category_stats(&self, session_id: i32) -> Result<Vec<CategoryStats>> {
        let stats = sqlx::query_as::<_, CategoryStats>(
            r#"
            SELECT
                q.category AS category,
                COUNT(*) AS total,
                SUM(CASE WHEN sq.is_correct THEN 1 ELSE 0 END) AS correct,
                ROUND(SUM(CASE WHEN sq.is_correct THEN 1 ELSE 0 END)::NUMERIC * 100.0 / COUNT(*), 1)::FLOAT8 AS accuracy
            FROM session_questions sq
            JOIN questions q ON sq.question_id = q.id
            WHERE sq.session_id = $1 AND q.category IS NOT NULL AND sq.is_correct IS NOT NULL
            GROUP BY q.category
            "#,
        )
        .bind(session_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(stats)
    }
}
