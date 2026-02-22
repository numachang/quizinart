use color_eyre::Result;

use super::models::{DailyAccuracy, QuestionStatsModel, SessionReportModel};
use super::Db;

impl Db {
    pub async fn get_questions_report(&self, quiz_id: i32) -> Result<Vec<QuestionStatsModel>> {
        let report = sqlx::query_as!(
            QuestionStatsModel,
            r#"
            SELECT q.question AS question, COUNT(*) AS "correct_answers!"
            FROM questions q
            JOIN session_questions sq ON sq.question_id = q.id AND sq.is_correct = TRUE
            WHERE q.quiz_id = $1
            GROUP BY q.question
            "#,
            quiz_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(report)
    }

    pub async fn get_sessions_report(&self, quiz_id: i32) -> Result<Vec<SessionReportModel>> {
        let report = sqlx::query_as!(
            SessionReportModel,
            r#"
            SELECT
                session_id AS "id!",
                name AS "name!",
                session_token AS "session_token!",
                correct_answers AS "score!",
                total_questions AS "total_questions!",
                answered_questions AS "answered_questions!",
                is_complete AS "is_complete!",
                question_count,
                selection_mode
            FROM session_stats
            WHERE quiz_id = $1
            ORDER BY session_id DESC
            "#,
            quiz_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(report)
    }

    pub async fn get_daily_accuracy(&self, quiz_id: i32) -> Result<Vec<DailyAccuracy>> {
        let accuracy = sqlx::query_as!(
            DailyAccuracy,
            r#"
            SELECT SUBSTR(s.name, 1, 10) AS "date_label!",
                   ROUND(SUM(CASE WHEN sq.is_correct THEN 1 ELSE 0 END)::NUMERIC * 100.0 / COUNT(*), 1)::FLOAT8 AS "accuracy!"
            FROM session_questions sq
            JOIN quiz_sessions s ON s.id = sq.session_id
            WHERE s.quiz_id = $1 AND sq.is_correct IS NOT NULL
            GROUP BY SUBSTR(s.name, 1, 10)
            ORDER BY SUBSTR(s.name, 1, 10) ASC
            "#,
            quiz_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(accuracy)
    }
}
