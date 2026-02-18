use color_eyre::{eyre::OptionExt, Result};
use libsql::params;

use super::helpers;
use super::models::{
    QuestionModel, QuestionOptionModel, QuizCategoryOverallStats, QuizOverallStats,
};
use super::Db;

impl Db {
    pub async fn get_question(&self, question_id: i32) -> Result<QuestionModel> {
        let conn = self.db.connect()?;

        let row = conn
            .query(
                "SELECT question, is_multiple_choice FROM questions WHERE id = ?",
                params![question_id],
            )
            .await?
            .next()
            .await?
            .ok_or_eyre("could not get question")?;

        let question = row.get::<String>(0)?;
        let is_multiple_choice = row.get::<bool>(1).unwrap_or(false);

        let options: Vec<QuestionOptionModel> = helpers::query_all(
            &conn,
            "SELECT id, is_answer, option, explanation FROM options WHERE question_id = ?",
            params![question_id],
        )
        .await?;

        Ok(QuestionModel {
            question,
            is_multiple_choice,
            options,
        })
    }

    pub async fn question_id_from_idx(&self, quiz_id: i32, question_idx: i32) -> Result<i32> {
        let conn = self.db.connect()?;
        let question_id = conn
            .query(
                "SELECT id FROM questions WHERE quiz_id = ? LIMIT 1 OFFSET ?",
                params![quiz_id, question_idx],
            )
            .await?
            .next()
            .await?
            .ok_or_eyre("no question id found")?
            .get::<i32>(0)?;

        Ok(question_id)
    }

    pub async fn questions_count(&self, quiz_id: i32) -> Result<i32> {
        let conn = self.db.connect()?;
        Ok(conn
            .query(
                "SELECT count(*) FROM questions WHERE quiz_id = ?",
                params![quiz_id],
            )
            .await?
            .next()
            .await?
            .ok_or_eyre("could not get questions count")?
            .get::<i32>(0)?)
    }

    pub async fn get_question_by_idx(&self, session_id: i32, idx: i32) -> Result<i32> {
        let conn = self.db.connect()?;
        let question_id = conn
            .query(
                "SELECT question_id FROM session_questions WHERE session_id = ? AND question_number = ?",
                params![session_id, idx],
            )
            .await?
            .next()
            .await?
            .ok_or_eyre("no question id found")?
            .get::<i32>(0)?;

        Ok(question_id)
    }

    pub async fn get_available_categories(&self, quiz_id: i32) -> Result<Vec<String>> {
        let conn = self.db.connect()?;
        let mut rows = conn
            .query(
                "SELECT DISTINCT category FROM questions WHERE quiz_id = ? AND category IS NOT NULL ORDER BY category",
                params![quiz_id],
            )
            .await?;

        let mut categories = Vec::new();
        while let Some(row) = rows.next().await? {
            categories.push(row.get::<String>(0)?);
        }
        Ok(categories)
    }

    pub async fn questions_count_for_session(&self, session_id: i32) -> Result<i32> {
        let conn = self.db.connect()?;
        Ok(conn
            .query(
                "SELECT count(*) FROM session_questions WHERE session_id = ?",
                params![session_id],
            )
            .await?
            .next()
            .await?
            .ok_or_eyre("could not get questions count")?
            .get::<i32>(0)?)
    }

    pub async fn get_quiz_overall_stats(&self, quiz_id: i32) -> Result<QuizOverallStats> {
        let conn = self.db.connect()?;

        let row = conn
            .query(
                r#"
                SELECT
                    (SELECT COUNT(*) FROM questions WHERE quiz_id = ?) AS total_questions,
                    COUNT(DISTINCT sq.question_id) AS unique_asked,
                    COALESCE(SUM(CASE WHEN sq.is_correct = 1 THEN 1 ELSE 0 END), 0) AS total_correct,
                    COUNT(*) AS total_answered
                FROM session_questions sq
                JOIN quiz_sessions s ON s.id = sq.session_id
                WHERE s.quiz_id = ? AND sq.is_correct IS NOT NULL
                "#,
                params![quiz_id, quiz_id],
            )
            .await?
            .next()
            .await?;

        match row {
            Some(row) => Ok(libsql::de::from_row::<QuizOverallStats>(&row)?),
            None => Ok(QuizOverallStats {
                total_questions: self.questions_count(quiz_id).await?,
                unique_asked: 0,
                total_correct: 0,
                total_answered: 0,
            }),
        }
    }

    pub async fn get_quiz_category_stats(
        &self,
        quiz_id: i32,
    ) -> Result<Vec<QuizCategoryOverallStats>> {
        let conn = self.db.connect()?;
        helpers::query_all(
            &conn,
            r#"
            SELECT
                q.category AS category,
                COUNT(DISTINCT q.id) AS total_in_category,
                COUNT(DISTINCT CASE WHEN sq.is_correct IS NOT NULL THEN sq.question_id END) AS unique_asked,
                COALESCE(SUM(CASE WHEN sq.is_correct = 1 THEN 1 ELSE 0 END), 0) AS total_correct,
                COUNT(CASE WHEN sq.is_correct IS NOT NULL THEN 1 END) AS total_answered
            FROM questions q
            LEFT JOIN session_questions sq ON sq.question_id = q.id
            WHERE q.quiz_id = ? AND q.category IS NOT NULL
            GROUP BY q.category
            ORDER BY q.category
            "#,
            params![quiz_id],
        )
        .await
    }

    pub async fn get_correct_option_ids(&self, question_id: i32) -> Result<Vec<i32>> {
        let conn = self.db.connect()?;
        let mut rows = conn
            .query(
                "SELECT id FROM options WHERE question_id = ? AND is_answer = 1",
                params![question_id],
            )
            .await?;

        let mut ids = Vec::new();
        while let Some(row) = rows.next().await? {
            ids.push(row.get::<i32>(0)?);
        }
        Ok(ids)
    }
}
