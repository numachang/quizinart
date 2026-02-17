use color_eyre::{eyre::OptionExt, Result};
use futures::{future, StreamExt, TryStreamExt};
use libsql::params;

use super::Db;
use super::models::{AnswerModel, CategoryStats};

impl Db {
    pub async fn is_question_answered(&self, session_id: i32, question_id: i32) -> Result<bool> {
        let conn = self.db.connect()?;
        let result = conn
            .query(
                "SELECT COUNT(*) FROM user_answers WHERE session_id = ? AND question_id = ?",
                params![session_id, question_id],
            )
            .await?
            .next()
            .await?
            .ok_or_eyre("failed to count answers")?
            .get::<i32>(0)?;

        Ok(result > 0)
    }

    pub async fn get_selected_answers(&self, session_id: i32, question_id: i32) -> Result<Vec<i32>> {
        let conn = self.db.connect()?;
        let mut rows = conn
            .query(
                "SELECT option_id FROM user_answers WHERE session_id = ? AND question_id = ?",
                params![session_id, question_id],
            )
            .await?;

        let mut option_ids = Vec::new();
        while let Some(row) = rows.next().await? {
            option_ids.push(row.get::<i32>(0)?);
        }

        Ok(option_ids)
    }

    pub async fn create_answer(
        &self,
        session_id: i32,
        question_id: i32,
        option_id: i32,
        is_correct: bool,
    ) -> Result<()> {
        let conn = self.db.connect()?;

        let rows = conn
            .execute(
                "INSERT INTO user_answers (is_correct, option_id, question_id, session_id) VALUES (?, ?, ?, ?)",
                params![is_correct, option_id, question_id, session_id],
            )
            .await?;

        tracing::info!(
            "answer created for session={session_id} question={question_id}: {rows:?}"
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
        let conn = self.db.connect()?;
        conn.execute(
            "UPDATE session_questions SET is_correct = ? WHERE session_id = ? AND question_id = ?",
            params![is_correct, session_id, question_id],
        )
        .await?;

        Ok(())
    }

    /// 正解数カウント（問題単位で正確）
    pub async fn correct_answers(&self, session_id: i32) -> Result<i32> {
        let conn = self.db.connect()?;
        Ok(conn
            .query(
                "SELECT COUNT(*) FROM session_questions WHERE session_id = ? AND is_correct = 1",
                params![session_id],
            )
            .await?
            .next()
            .await?
            .ok_or_eyre("could not get correct answers count")?
            .get::<i32>(0)?)
    }

    pub async fn get_answers(&self, session_id: i32) -> Result<Vec<AnswerModel>> {
        let conn = self.db.connect()?;
        Ok(conn
            .query(
                r#"
            SELECT q.question, sq.is_correct, sq.question_number
            FROM session_questions sq
            JOIN questions q ON sq.question_id = q.id
            WHERE sq.session_id = ? AND sq.is_correct IS NOT NULL
            ORDER BY sq.question_number"#,
                params![session_id],
            )
            .await?
            .into_stream()
            .map_ok(|r| AnswerModel {
                question: r.get::<String>(0).expect("failed to get answer question"),
                is_correct: r.get::<bool>(1).expect("failed to get answer correctness"),
                question_idx: r.get::<i32>(2).expect("failed to get question index"),
            })
            .filter_map(|r| future::ready(r.ok()))
            .collect::<Vec<_>>()
            .await)
    }

    pub async fn get_incorrect_questions(&self, session_id: i32) -> Result<Vec<i32>> {
        let conn = self.db.connect()?;
        Ok(conn
            .query(
                "SELECT DISTINCT question_id FROM session_questions WHERE session_id = ? AND is_correct = 0",
                params![session_id],
            )
            .await?
            .into_stream()
            .map_ok(|r| r.get::<i32>(0).expect("could not get question id"))
            .filter_map(|r| future::ready(r.ok()))
            .collect::<Vec<_>>()
            .await)
    }

    pub async fn get_category_stats(&self, session_id: i32) -> Result<Vec<CategoryStats>> {
        let conn = self.db.connect()?;
        Ok(conn
            .query(
                r#"
                SELECT
                    q.category,
                    COUNT(*) as total,
                    SUM(CASE WHEN sq.is_correct THEN 1 ELSE 0 END) as correct,
                    ROUND(CAST(SUM(CASE WHEN sq.is_correct THEN 1 ELSE 0 END) AS REAL) * 100.0 / COUNT(*), 1) as accuracy
                FROM session_questions sq
                JOIN questions q ON sq.question_id = q.id
                WHERE sq.session_id = ? AND q.category IS NOT NULL AND sq.is_correct IS NOT NULL
                GROUP BY q.category
                "#,
                params![session_id],
            )
            .await?
            .into_stream()
            .map_ok(|r| CategoryStats {
                category: r.get::<String>(0).expect("could not get category"),
                total: r.get::<i32>(1).expect("could not get total"),
                correct: r.get::<i32>(2).expect("could not get correct"),
                accuracy: r.get::<f64>(3).expect("could not get accuracy"),
            })
            .filter_map(|r| future::ready(r.ok()))
            .collect::<Vec<_>>()
            .await)
    }
}
