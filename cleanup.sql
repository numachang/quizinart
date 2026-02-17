-- Delete old submissions and related data
DELETE FROM answers WHERE submission_id IN (1, 2);
DELETE FROM question_sequence WHERE submission_id IN (1, 2);
DELETE FROM submissions WHERE id IN (1, 2);
