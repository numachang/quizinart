-- Delete quiz sessions and related data by session ID
-- ⚠️ 実行すると学習データが消えます。対象IDを確認してから実行してください。
--
-- 使い方:
--   特定セッション削除: IN (1, 2) のIDを変更
--   全セッション削除:   WHERE句を削除

DELETE FROM user_answers WHERE session_id IN (1, 2);
DELETE FROM session_questions WHERE session_id IN (1, 2);
DELETE FROM quiz_sessions WHERE id IN (1, 2);
