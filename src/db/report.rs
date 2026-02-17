use color_eyre::Result;
use futures::{future, StreamExt, TryStreamExt};
use libsql::params;

use super::Db;
use super::models::{QuestionStatsModel, SessionReportModel};

impl Db {
    pub async fn get_questions_report(&self, quiz_id: i32) -> Result<Vec<QuestionStatsModel>> {
        let conn = self.db.connect()?;
        Ok(conn
            .query(
                r#"
            SELECT q.question, COUNT(*) as correct_count
            FROM questions q
            JOIN session_questions sq ON sq.question_id = q.id AND sq.is_correct = 1
            WHERE q.quiz_id = ?
            GROUP BY q.question
                "#,
                params![quiz_id],
            )
            .await?
            .into_stream()
            .map_ok(|r| QuestionStatsModel {
                question: r.get::<String>(0).expect("failed to get question"),
                correct_answers: r
                    .get::<i32>(1)
                    .expect("failed to get correct answers count"),
            })
            .filter_map(|r| future::ready(r.ok()))
            .collect::<Vec<_>>()
            .await)
    }

    pub async fn get_sessions_report(
        &self,
        quiz_id: i32,
    ) -> Result<Vec<SessionReportModel>> {
        let conn = self.db.connect()?;
        Ok(conn
            .query(
                r#"
            SELECT
                session_id,
                name,
                session_token,
                correct_answers,
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
            .await?
            .into_stream()
            .map_ok(|r| SessionReportModel {
                id: r.get::<i32>(0).expect("failed to get session id"),
                name: r.get::<String>(1).expect("failed to get session name"),
                session_token: r.get::<String>(2).expect("failed to get session token"),
                score: r.get::<i32>(3).expect("failed to get score"),
                total_questions: r.get::<i32>(4).expect("failed to get total questions"),
                answered_questions: r.get::<i32>(5).expect("failed to get answered questions"),
                is_complete: r.get::<bool>(6).unwrap_or(false),
                question_count: r.get::<Option<i32>>(7).ok().flatten(),
                selection_mode: r.get::<Option<String>>(8).ok().flatten(),
            })
            .filter_map(|r| future::ready(r.ok()))
            .collect::<Vec<_>>()
            .await)
    }
}
