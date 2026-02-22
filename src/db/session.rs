use color_eyre::{eyre::OptionExt, Result};
use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::SeedableRng;
use ulid::Ulid;

use super::models::QuizSessionModel;
use super::Db;

impl Db {
    pub async fn session_name_exists(&self, name: &str, quiz_id: i32) -> Result<bool> {
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM quiz_sessions WHERE name = $1 AND quiz_id = $2)",
        )
        .bind(name)
        .bind(quiz_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(exists)
    }

    /// Returns `(session_token, session_id)`.
    pub async fn create_session(
        &self,
        name: &str,
        quiz_id: i32,
        question_count: i32,
        selection_mode: &str,
        user_id: i32,
    ) -> Result<(String, i32)> {
        if self.session_name_exists(name, quiz_id).await? {
            return Err(color_eyre::eyre::eyre!(
                "Session name '{}' is already in use for this quiz. Please choose a different name.",
                name
            ));
        }

        let session_token = Ulid::new().to_string();
        let shuffle_seed = rand::random::<i32>();

        // Select questions before transaction (read-only)
        let selected_ids = self
            .select_questions(quiz_id, question_count, selection_mode, shuffle_seed)
            .await?;

        // Transaction: insert session + session_questions atomically
        let mut tx = self.pool.begin().await?;

        let session_id: i32 = sqlx::query_scalar(
            "INSERT INTO quiz_sessions (name, session_token, quiz_id, shuffle_seed, question_count, selection_mode, user_id) VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING id",
        )
        .bind(name)
        .bind(&session_token)
        .bind(quiz_id)
        .bind(shuffle_seed)
        .bind(question_count)
        .bind(selection_mode)
        .bind(user_id)
        .fetch_one(&mut *tx)
        .await?;

        Self::batch_insert_session_questions_tx(&mut tx, session_id, &selected_ids).await?;

        tx.commit().await?;

        tracing::info!(
            "session created for quiz={quiz_id}: session_id={session_id}, mode={selection_mode}, user_id={user_id}"
        );
        Ok((session_token, session_id))
    }

    /// Batch insert session_questions using a transaction executor.
    async fn batch_insert_session_questions_tx(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        session_id: i32,
        question_ids: &[i32],
    ) -> Result<()> {
        if question_ids.is_empty() {
            return Ok(());
        }

        sqlx::query(
            r#"
            INSERT INTO session_questions (session_id, question_id, question_number)
            SELECT $1, q, (n - 1)::INT
            FROM UNNEST($2::INT4[]) WITH ORDINALITY AS t(q, n)
            "#,
        )
        .bind(session_id)
        .bind(question_ids)
        .execute(&mut **tx)
        .await?;

        Ok(())
    }

    async fn select_questions(
        &self,
        quiz_id: i32,
        question_count: i32,
        selection_mode: &str,
        shuffle_seed: i32,
    ) -> Result<Vec<i32>> {
        let mut rng = StdRng::seed_from_u64(shuffle_seed as u64);

        match selection_mode {
            "unanswered" => {
                let mut unanswered: Vec<i32> = sqlx::query_scalar(
                    r#"
                    SELECT id FROM questions
                    WHERE quiz_id = $1 AND id NOT IN (
                        SELECT DISTINCT question_id FROM session_questions
                        JOIN quiz_sessions ON quiz_sessions.id = session_questions.session_id
                        WHERE quiz_sessions.quiz_id = $1
                    )
                    ORDER BY id
                    "#,
                )
                .bind(quiz_id)
                .fetch_all(&self.pool)
                .await?;

                unanswered.shuffle(&mut rng);

                if (unanswered.len() as i32) >= question_count {
                    unanswered.truncate(question_count as usize);
                    Ok(unanswered)
                } else {
                    let needed = question_count as usize - unanswered.len();
                    let mut all_ids = self.get_all_question_ids(quiz_id).await?;
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
                let mut incorrect: Vec<i32> = sqlx::query_scalar(
                    r#"
                    SELECT question_id FROM question_stats
                    WHERE quiz_id = $1 AND times_incorrect > 0
                    ORDER BY accuracy ASC, times_incorrect DESC
                    "#,
                )
                .bind(quiz_id)
                .fetch_all(&self.pool)
                .await?;

                incorrect.shuffle(&mut rng);

                if (incorrect.len() as i32) >= question_count {
                    incorrect.truncate(question_count as usize);
                    Ok(incorrect)
                } else {
                    let needed = question_count as usize - incorrect.len();
                    let mut all_ids = self.get_all_question_ids(quiz_id).await?;
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
                let mut all_ids = self.get_all_question_ids(quiz_id).await?;
                all_ids.shuffle(&mut rng);
                all_ids.truncate(question_count as usize);
                Ok(all_ids)
            }
        }
    }

    async fn get_all_question_ids(&self, quiz_id: i32) -> Result<Vec<i32>> {
        let ids = sqlx::query_scalar("SELECT id FROM questions WHERE quiz_id = $1 ORDER BY id")
            .bind(quiz_id)
            .fetch_all(&self.pool)
            .await?;

        Ok(ids)
    }

    pub async fn sessions_count(&self, quiz_id: i32) -> Result<i32> {
        let count: i32 =
            sqlx::query_scalar("SELECT COUNT(*)::INT FROM quiz_sessions WHERE quiz_id = $1")
                .bind(quiz_id)
                .fetch_one(&self.pool)
                .await?;

        Ok(count)
    }

    pub async fn get_session(&self, token: &str) -> Result<QuizSessionModel> {
        let session = sqlx::query_as::<_, QuizSessionModel>(
            "SELECT id, quiz_id, name, question_count, selection_mode FROM quiz_sessions WHERE session_token = $1",
        )
        .bind(token)
        .fetch_one(&self.pool)
        .await?;

        Ok(session)
    }

    pub async fn get_session_by_id(&self, session_id: i32) -> Result<QuizSessionModel> {
        let session = sqlx::query_as::<_, QuizSessionModel>(
            "SELECT id, quiz_id, name, question_count, selection_mode FROM quiz_sessions WHERE id = $1",
        )
        .bind(session_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(session)
    }

    /// 回答済み問題数を返す（= 次の未回答問題の question_number）
    pub async fn current_question_index(&self, session_id: i32) -> Result<i32> {
        let count: i32 = sqlx::query_scalar(
            "SELECT COUNT(*)::INT FROM session_questions WHERE session_id = $1 AND is_correct IS NOT NULL",
        )
        .bind(session_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(count)
    }

    pub async fn create_session_with_questions(
        &self,
        name: &str,
        quiz_id: i32,
        question_ids: &[i32],
        selection_mode: &str,
        user_id: i32,
    ) -> Result<String> {
        let mut seen = std::collections::HashSet::new();
        let deduped_question_ids: Vec<i32> = question_ids
            .iter()
            .copied()
            .filter(|id| seen.insert(*id))
            .collect();

        let session_token = Ulid::new().to_string();
        let question_count = deduped_question_ids.len() as i32;

        // Transaction: insert session + session_questions atomically
        let mut tx = self.pool.begin().await?;

        let session_id: i32 = sqlx::query_scalar(
            "INSERT INTO quiz_sessions (name, session_token, quiz_id, shuffle_seed, question_count, selection_mode, user_id) VALUES ($1, $2, $3, 0, $4, $5, $6) RETURNING id",
        )
        .bind(name)
        .bind(&session_token)
        .bind(quiz_id)
        .bind(question_count)
        .bind(selection_mode)
        .bind(user_id)
        .fetch_one(&mut *tx)
        .await?;

        Self::batch_insert_session_questions_tx(&mut tx, session_id, &deduped_question_ids).await?;

        tx.commit().await?;

        tracing::info!(
            "session created with specific questions: session_id={session_id}, questions={question_count}, mode={selection_mode}"
        );
        Ok(session_token)
    }

    pub async fn delete_session(&self, session_id: i32) -> Result<()> {
        sqlx::query("DELETE FROM quiz_sessions WHERE id = $1")
            .bind(session_id)
            .execute(&self.pool)
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

        sqlx::query("UPDATE quiz_sessions SET name = $1 WHERE id = $2")
            .bind(new_name)
            .bind(session_id)
            .execute(&self.pool)
            .await?;

        tracing::info!("renamed session {session_id} to '{new_name}'");
        Ok(())
    }

    pub async fn is_question_bookmarked(&self, session_id: i32, question_id: i32) -> Result<bool> {
        let bookmarked: bool = sqlx::query_scalar(
            "SELECT is_bookmarked FROM session_questions WHERE session_id = $1 AND question_id = $2",
        )
        .bind(session_id)
        .bind(question_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_eyre("session_question not found")?;

        Ok(bookmarked)
    }

    /// ブックマーク状態をトグルし、新しい状態を返す
    pub async fn toggle_bookmark(&self, session_id: i32, question_id: i32) -> Result<bool> {
        sqlx::query(
            "UPDATE session_questions SET is_bookmarked = NOT is_bookmarked WHERE session_id = $1 AND question_id = $2",
        )
        .bind(session_id)
        .bind(question_id)
        .execute(&self.pool)
        .await?;

        self.is_question_bookmarked(session_id, question_id).await
    }

    pub async fn get_bookmarked_questions(&self, session_id: i32) -> Result<Vec<i32>> {
        let ids: Vec<i32> = sqlx::query_scalar(
            "SELECT question_id FROM session_questions WHERE session_id = $1 AND is_bookmarked = TRUE",
        )
        .bind(session_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(ids)
    }

    pub async fn find_incomplete_session(
        &self,
        name: &str,
        quiz_id: i32,
    ) -> Result<Option<(i32, String)>> {
        let row = sqlx::query_as::<_, (i32, String)>(
            "SELECT session_id, session_token FROM session_stats WHERE name = $1 AND quiz_id = $2 AND is_complete = FALSE ORDER BY session_id DESC",
        )
        .bind(name)
        .bind(quiz_id)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some((session_id, session_token)) => {
                tracing::info!(
                    "Found incomplete session {} for user '{}'",
                    session_id,
                    name
                );
                Ok(Some((session_id, session_token)))
            }
            None => {
                tracing::info!("No incomplete session found for user '{}'", name);
                Ok(None)
            }
        }
    }

    /// Verify that a session belongs to the given user
    pub async fn verify_session_owner(&self, session_id: i32, user_id: i32) -> Result<bool> {
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM quiz_sessions WHERE id = $1 AND user_id = $2)",
        )
        .bind(session_id)
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(exists)
    }
}
