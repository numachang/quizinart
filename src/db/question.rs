use color_eyre::{eyre::OptionExt, Result};
use futures::{future, StreamExt, TryStreamExt};
use libsql::params;

use super::Db;
use super::models::{QuestionModel, QuestionOptionModel, QuizOverallStats, QuizCategoryOverallStats};

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

        let options = conn
            .query(
                "SELECT id, option, is_answer, explanation FROM options WHERE question_id = ?",
                params![question_id],
            )
            .await?
            .into_stream()
            .map_ok(|r| QuestionOptionModel {
                id: r.get::<i32>(0).expect("could not get option id"),
                option: r.get::<String>(1).expect("could not get option"),
                is_answer: r.get::<bool>(2).expect("could not get option is_answer"),
                explanation: r.get::<Option<String>>(3).ok().flatten(),
            })
            .filter_map(|r| future::ready(r.ok()))
            .collect::<Vec<_>>()
            .await;

        Ok(QuestionModel { question, is_multiple_choice, options })
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
        Ok(conn
            .query(
                "SELECT DISTINCT category FROM questions WHERE quiz_id = ? AND category IS NOT NULL ORDER BY category",
                params![quiz_id],
            )
            .await?
            .into_stream()
            .map_ok(|r| r.get::<String>(0).expect("could not get category"))
            .filter_map(|r| future::ready(r.ok()))
            .collect::<Vec<_>>()
            .await)
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
        let total_questions = self.questions_count(quiz_id).await?;

        let row = conn
            .query(
                r#"
                SELECT
                    COUNT(DISTINCT sq.question_id) as asked_unique,
                    COALESCE(SUM(CASE WHEN sq.is_correct = 1 THEN 1 ELSE 0 END), 0) as total_correct,
                    COUNT(*) as total_answered
                FROM session_questions sq
                JOIN quiz_sessions s ON s.id = sq.session_id
                WHERE s.quiz_id = ? AND sq.is_correct IS NOT NULL
                "#,
                params![quiz_id],
            )
            .await?
            .next()
            .await?;

        match row {
            Some(r) => Ok(QuizOverallStats {
                total_questions,
                unique_asked: r.get::<i32>(0).unwrap_or(0),
                total_correct: r.get::<i32>(1).unwrap_or(0),
                total_answered: r.get::<i32>(2).unwrap_or(0),
            }),
            None => Ok(QuizOverallStats {
                total_questions,
                unique_asked: 0,
                total_correct: 0,
                total_answered: 0,
            }),
        }
    }

    pub async fn get_quiz_category_stats(&self, quiz_id: i32) -> Result<Vec<QuizCategoryOverallStats>> {
        let conn = self.db.connect()?;
        Ok(conn
            .query(
                r#"
                SELECT
                    q.category,
                    COUNT(DISTINCT q.id) as total_in_category,
                    COUNT(DISTINCT CASE WHEN sq.is_correct IS NOT NULL THEN sq.question_id END) as asked_unique,
                    COALESCE(SUM(CASE WHEN sq.is_correct = 1 THEN 1 ELSE 0 END), 0) as correct_count,
                    COUNT(CASE WHEN sq.is_correct IS NOT NULL THEN 1 END) as answered_count
                FROM questions q
                LEFT JOIN session_questions sq ON sq.question_id = q.id
                WHERE q.quiz_id = ? AND q.category IS NOT NULL
                GROUP BY q.category
                ORDER BY q.category
                "#,
                params![quiz_id],
            )
            .await?
            .into_stream()
            .map_ok(|r| QuizCategoryOverallStats {
                category: r.get::<String>(0).expect("could not get category"),
                total_in_category: r.get::<i32>(1).expect("could not get total_in_category"),
                unique_asked: r.get::<i32>(2).expect("could not get unique_asked"),
                total_correct: r.get::<i32>(3).expect("could not get correct_count"),
                total_answered: r.get::<i32>(4).expect("could not get answered_count"),
            })
            .filter_map(|r| future::ready(r.ok()))
            .collect::<Vec<_>>()
            .await)
    }

    pub async fn get_correct_option_ids(&self, question_id: i32) -> Result<Vec<i32>> {
        let conn = self.db.connect()?;
        Ok(conn
            .query(
                "SELECT id FROM options WHERE question_id = ? AND is_answer = 1",
                params![question_id],
            )
            .await?
            .into_stream()
            .map_ok(|r| r.get::<i32>(0).expect("could not get option id"))
            .filter_map(|r| future::ready(r.ok()))
            .collect::<Vec<_>>()
            .await)
    }
}
