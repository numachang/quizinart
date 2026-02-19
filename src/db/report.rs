use color_eyre::Result;
use libsql::params;

use super::helpers;
use super::models::{DailyAccuracy, QuestionStatsModel, SessionReportModel};
use super::Db;

impl Db {
    pub async fn get_questions_report(&self, quiz_id: i32) -> Result<Vec<QuestionStatsModel>> {
        let conn = self.db.connect()?;
        helpers::query_all(
            &conn,
            r#"
            SELECT q.question AS question, COUNT(*) AS correct_answers
            FROM questions q
            JOIN session_questions sq ON sq.question_id = q.id AND sq.is_correct = 1
            WHERE q.quiz_id = ?
            GROUP BY q.question
            "#,
            params![quiz_id],
        )
        .await
    }

    pub async fn get_sessions_report(&self, quiz_id: i32) -> Result<Vec<SessionReportModel>> {
        let conn = self.db.connect()?;
        helpers::query_all(
            &conn,
            r#"
            SELECT
                session_id AS id,
                name,
                session_token,
                correct_answers AS score,
                total_questions,
                answered_questions,
                is_complete,
                question_count,
                selection_mode
            FROM session_stats
            WHERE quiz_id = ?
            ORDER BY session_id DESC
            "#,
            params![quiz_id],
        )
        .await
    }

    pub async fn get_daily_accuracy(&self, quiz_id: i32) -> Result<Vec<DailyAccuracy>> {
        let conn = self.db.connect()?;
        helpers::query_all(
            &conn,
            r#"
            SELECT SUBSTR(s.name, 1, 10) AS date_label,
                   ROUND(CAST(SUM(CASE WHEN sq.is_correct = 1 THEN 1 ELSE 0 END) AS REAL) * 100.0 / COUNT(*), 1) AS accuracy
            FROM session_questions sq
            JOIN quiz_sessions s ON s.id = sq.session_id
            WHERE s.quiz_id = ? AND sq.is_correct IS NOT NULL
            GROUP BY SUBSTR(s.name, 1, 10)
            ORDER BY SUBSTR(s.name, 1, 10) ASC
            "#,
            params![quiz_id],
        )
        .await
    }
}
