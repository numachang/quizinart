use color_eyre::{eyre::OptionExt, Result};
use futures::{future, StreamExt, TryStreamExt};
use libsql::params;
use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::SeedableRng;
use ulid::Ulid;

use super::models::QuizSessionModel;
use super::Db;

impl Db {
    pub async fn session_name_exists(&self, name: &str, quiz_id: i32) -> Result<bool> {
        let conn = self.db.connect()?;
        let mut rows = conn
            .query(
                "SELECT COUNT(*) FROM quiz_sessions WHERE name = ? AND quiz_id = ?",
                params![name, quiz_id],
            )
            .await?;

        if let Some(row) = rows.next().await? {
            let count: i32 = row.get(0)?;
            Ok(count > 0)
        } else {
            Ok(false)
        }
    }

    pub async fn create_session(
        &self,
        name: &str,
        quiz_id: i32,
        question_count: i32,
        selection_mode: &str,
    ) -> Result<String> {
        if self.session_name_exists(name, quiz_id).await? {
            return Err(color_eyre::eyre::eyre!(
                "Session name '{}' is already in use for this quiz. Please choose a different name.",
                name
            ));
        }

        let session_token = Ulid::new().to_string();
        let token_str = session_token.as_str();
        let conn = self.db.connect()?;

        let shuffle_seed = rand::random::<i32>();

        let session_id = conn
            .query(
                "INSERT INTO quiz_sessions (name, session_token, quiz_id, shuffle_seed, question_count, selection_mode) VALUES (?, ?, ?, ?, ?, ?) RETURNING id",
                params![name, token_str, quiz_id, shuffle_seed, question_count, selection_mode],
            )
            .await?
            .next()
            .await?
            .ok_or_eyre("could not get session id")?
            .get::<i32>(0)?;

        // Select questions based on mode
        let selected_ids = self
            .select_questions(&conn, quiz_id, question_count, selection_mode, shuffle_seed)
            .await?;

        // Save to session_questions table
        for (idx, question_id) in selected_ids.iter().enumerate() {
            conn.execute(
                "INSERT INTO session_questions (session_id, question_id, question_number) VALUES (?, ?, ?)",
                params![session_id, question_id, idx as i32],
            )
            .await?;
        }

        tracing::info!(
            "session created for quiz={quiz_id}: session_id={session_id}, questions={}, mode={selection_mode}",
            selected_ids.len()
        );
        Ok(session_token)
    }

    async fn select_questions(
        &self,
        conn: &libsql::Connection,
        quiz_id: i32,
        question_count: i32,
        selection_mode: &str,
        shuffle_seed: i32,
    ) -> Result<Vec<i32>> {
        let mut rng = StdRng::seed_from_u64(shuffle_seed as u64);

        match selection_mode {
            "unanswered" => {
                // Get questions that have never been asked in any session
                let mut unanswered: Vec<i32> = conn
                    .query(
                        r#"
                        SELECT id FROM questions
                        WHERE quiz_id = ? AND id NOT IN (
                            SELECT DISTINCT question_id FROM session_questions
                            JOIN quiz_sessions ON quiz_sessions.id = session_questions.session_id
                            WHERE quiz_sessions.quiz_id = ?
                        )
                        ORDER BY id
                        "#,
                        params![quiz_id, quiz_id],
                    )
                    .await?
                    .into_stream()
                    .map_ok(|r| r.get::<i32>(0).expect("could not get question id"))
                    .filter_map(|r| future::ready(r.ok()))
                    .collect::<Vec<_>>()
                    .await;

                unanswered.shuffle(&mut rng);

                if (unanswered.len() as i32) >= question_count {
                    unanswered.truncate(question_count as usize);
                    Ok(unanswered)
                } else {
                    // Fill remaining with random questions (not already selected)
                    let needed = question_count as usize - unanswered.len();
                    let mut all_ids = self.get_all_question_ids(conn, quiz_id).await?;
                    all_ids.shuffle(&mut rng);
                    let already_selected: std::collections::HashSet<i32> =
                        unanswered.iter().cloned().collect();
                    let fill: Vec<i32> = all_ids
                        .into_iter()
                        .filter(|id| !already_selected.contains(id))
                        .take(needed)
                        .collect();
                    unanswered.extend(fill);
                    Ok(unanswered)
                }
            }
            "incorrect" => {
                // Get questions that were answered incorrectly, sorted by accuracy (worst first)
                let mut incorrect: Vec<i32> = conn
                    .query(
                        r#"
                        SELECT question_id FROM question_stats
                        WHERE quiz_id = ? AND times_incorrect > 0
                        ORDER BY accuracy ASC, times_incorrect DESC
                        "#,
                        params![quiz_id],
                    )
                    .await?
                    .into_stream()
                    .map_ok(|r| r.get::<i32>(0).expect("could not get question id"))
                    .filter_map(|r| future::ready(r.ok()))
                    .collect::<Vec<_>>()
                    .await;

                incorrect.shuffle(&mut rng);

                if (incorrect.len() as i32) >= question_count {
                    incorrect.truncate(question_count as usize);
                    Ok(incorrect)
                } else {
                    let needed = question_count as usize - incorrect.len();
                    let mut all_ids = self.get_all_question_ids(conn, quiz_id).await?;
                    all_ids.shuffle(&mut rng);
                    let already_selected: std::collections::HashSet<i32> =
                        incorrect.iter().cloned().collect();
                    let fill: Vec<i32> = all_ids
                        .into_iter()
                        .filter(|id| !already_selected.contains(id))
                        .take(needed)
                        .collect();
                    incorrect.extend(fill);
                    Ok(incorrect)
                }
            }
            _ => {
                // "random" mode: shuffle all questions and take question_count
                let mut all_ids = self.get_all_question_ids(conn, quiz_id).await?;
                all_ids.shuffle(&mut rng);
                all_ids.truncate(question_count as usize);
                Ok(all_ids)
            }
        }
    }

    async fn get_all_question_ids(
        &self,
        conn: &libsql::Connection,
        quiz_id: i32,
    ) -> Result<Vec<i32>> {
        Ok(conn
            .query(
                "SELECT id FROM questions WHERE quiz_id = ? ORDER BY id",
                params![quiz_id],
            )
            .await?
            .into_stream()
            .map_ok(|r| r.get::<i32>(0).expect("could not get question id"))
            .filter_map(|r| future::ready(r.ok()))
            .collect::<Vec<_>>()
            .await)
    }

    pub async fn sessions_count(&self, quiz_id: i32) -> Result<i32> {
        let conn = self.db.connect()?;
        Ok(conn
            .query(
                "SELECT count(*) FROM quiz_sessions WHERE quiz_id = ?",
                params![quiz_id],
            )
            .await?
            .next()
            .await?
            .ok_or_eyre("could not get sessions count")?
            .get::<i32>(0)?)
    }

    pub async fn get_session(&self, token: &str) -> Result<QuizSessionModel> {
        let conn = self.db.connect()?;
        conn.query(
            "SELECT id, quiz_id, name, question_count, selection_mode FROM quiz_sessions WHERE session_token = ?",
            params![token],
        )
        .await?
        .next()
        .await?
        .map(|r| QuizSessionModel {
            id: r.get::<i32>(0).expect("failed to get session id"),
            quiz_id: r.get::<i32>(1).expect("failed to get quiz id"),
            name: r.get::<String>(2).expect("failed to get session name"),
            question_count: r.get::<Option<i32>>(3).ok().flatten(),
            selection_mode: r.get::<Option<String>>(4).ok().flatten(),
        })
        .ok_or_eyre("could not get session")
    }

    pub async fn get_session_by_id(&self, session_id: i32) -> Result<QuizSessionModel> {
        let conn = self.db.connect()?;
        conn.query(
            "SELECT id, quiz_id, name, question_count, selection_mode FROM quiz_sessions WHERE id = ?",
            params![session_id],
        )
        .await?
        .next()
        .await?
        .map(|r| QuizSessionModel {
            id: r.get::<i32>(0).expect("failed to get session id"),
            quiz_id: r.get::<i32>(1).expect("failed to get quiz id"),
            name: r.get::<String>(2).expect("failed to get session name"),
            question_count: r.get::<Option<i32>>(3).ok().flatten(),
            selection_mode: r.get::<Option<String>>(4).ok().flatten(),
        })
        .ok_or_eyre("could not get session")
    }

    /// 回答済み問題数を返す（= 次の未回答問題の question_number）
    pub async fn current_question_index(&self, session_id: i32) -> Result<i32> {
        let conn = self.db.connect()?;
        Ok(conn
            .query(
                "SELECT COUNT(*) FROM session_questions WHERE session_id = ? AND is_correct IS NOT NULL",
                params![session_id],
            )
            .await?
            .next()
            .await?
            .ok_or_eyre("could not get current question index")?
            .get::<i32>(0)?)
    }

    pub async fn create_session_with_questions(
        &self,
        name: &str,
        quiz_id: i32,
        question_ids: &[i32],
        selection_mode: &str,
    ) -> Result<String> {
        let session_token = Ulid::new().to_string();
        let token_str = session_token.as_str();
        let conn = self.db.connect()?;
        let question_count = question_ids.len() as i32;

        let session_id = conn
            .query(
                "INSERT INTO quiz_sessions (name, session_token, quiz_id, shuffle_seed, question_count, selection_mode) VALUES (?, ?, ?, 0, ?, ?) RETURNING id",
                params![name, token_str, quiz_id, question_count, selection_mode],
            )
            .await?
            .next()
            .await?
            .ok_or_eyre("could not get session id")?
            .get::<i32>(0)?;

        for (idx, question_id) in question_ids.iter().enumerate() {
            conn.execute(
                "INSERT INTO session_questions (session_id, question_id, question_number) VALUES (?, ?, ?)",
                params![session_id, question_id, idx as i32],
            )
            .await?;
        }

        tracing::info!(
            "session created with specific questions: session_id={session_id}, questions={question_count}, mode={selection_mode}"
        );
        Ok(session_token)
    }

    pub async fn delete_session(&self, session_id: i32) -> Result<()> {
        let conn = self.db.connect()?;
        conn.execute(
            "DELETE FROM quiz_sessions WHERE id = ?",
            params![session_id],
        )
        .await?;
        tracing::info!("deleted session {session_id}");
        Ok(())
    }

    pub async fn rename_session(
        &self,
        session_id: i32,
        new_name: &str,
        quiz_id: i32,
    ) -> Result<()> {
        if self.session_name_exists(new_name, quiz_id).await? {
            return Err(color_eyre::eyre::eyre!(
                "Session name '{}' is already in use for this quiz.",
                new_name
            ));
        }
        let conn = self.db.connect()?;
        conn.execute(
            "UPDATE quiz_sessions SET name = ? WHERE id = ?",
            params![new_name, session_id],
        )
        .await?;
        tracing::info!("renamed session {session_id} to '{new_name}'");
        Ok(())
    }

    pub async fn find_incomplete_session(
        &self,
        name: &str,
        quiz_id: i32,
    ) -> Result<Option<(i32, String)>> {
        let conn = self.db.connect()?;

        let mut rows = conn.query(
            "SELECT session_id, session_token FROM session_stats WHERE name = ? AND quiz_id = ? AND is_complete = 0 ORDER BY session_id DESC",
            params![name, quiz_id],
        )
        .await?;

        if let Some(row) = rows.next().await? {
            let session_id = row.get::<i32>(0).expect("failed to get session id");
            let session_token = row.get::<String>(1).expect("failed to get session token");

            tracing::info!(
                "Found incomplete session {} for user '{}'",
                session_id,
                name
            );
            return Ok(Some((session_id, session_token)));
        }

        tracing::info!("No incomplete session found for user '{}'", name);
        Ok(None)
    }
}
