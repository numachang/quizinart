// Database model structs

use serde::Deserialize;

#[derive(Clone, Deserialize)]
pub struct AuthUser {
    pub id: i32,
    pub email: String,
    pub display_name: String,
}

#[derive(Deserialize)]
pub struct Quiz {
    pub id: i32,
    pub name: String,
    pub count: i32,
}

pub struct QuestionModel {
    pub question: String,
    pub is_multiple_choice: bool,
    pub options: Vec<QuestionOptionModel>,
}

#[derive(Deserialize)]
pub struct QuestionOptionModel {
    pub id: i32,
    pub is_answer: bool,
    pub option: String,
    pub explanation: Option<String>,
}

#[derive(Deserialize)]
pub struct QuizSessionModel {
    pub id: i32,
    pub quiz_id: i32,
    pub name: String,
    pub question_count: Option<i32>,
    pub selection_mode: Option<String>,
}

#[derive(Deserialize)]
pub struct AnswerModel {
    pub question: String,
    pub is_correct: bool,
    pub question_idx: i32,
    pub is_bookmarked: bool,
}

#[derive(Deserialize)]
pub struct QuestionStatsModel {
    pub question: String,
    pub correct_answers: i32,
}

#[derive(Deserialize)]
pub struct SessionReportModel {
    pub id: i32,
    pub name: String,
    pub score: i32,
    pub total_questions: i32,
    pub answered_questions: i32,
    pub is_complete: bool,
    pub session_token: String,
    pub question_count: Option<i32>,
    pub selection_mode: Option<String>,
}

#[derive(Deserialize)]
pub struct CategoryStats {
    pub category: String,
    pub total: i32,
    pub correct: i32,
    pub accuracy: f64,
}

#[derive(Deserialize)]
pub struct QuizOverallStats {
    pub total_questions: i32,
    pub unique_asked: i32,
    pub total_correct: i32,
    pub total_answered: i32,
}

#[derive(Deserialize)]
pub struct DailyAccuracy {
    pub date_label: String,
    pub accuracy: f64,
}

#[derive(Deserialize)]
pub struct QuizCategoryOverallStats {
    pub category: String,
    pub total_in_category: i32,
    pub unique_asked: i32,
    pub total_correct: i32,
    pub total_answered: i32,
}
