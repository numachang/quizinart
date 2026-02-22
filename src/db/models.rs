// Database model structs

#[derive(Clone, sqlx::FromRow)]
pub struct AuthUser {
    pub id: i32,
    pub email: String,
    pub display_name: String,
}

#[derive(sqlx::FromRow)]
pub struct Quiz {
    pub id: i32,
    pub public_id: String,
    pub name: String,
    pub count: i64,
    pub last_session_id: Option<i32>,
}

pub struct QuestionModel {
    pub question: String,
    pub is_multiple_choice: bool,
    pub options: Vec<QuestionOptionModel>,
}

#[derive(sqlx::FromRow)]
pub struct QuestionOptionModel {
    pub id: i32,
    pub is_answer: bool,
    pub option: String,
    pub explanation: Option<String>,
}

#[derive(sqlx::FromRow)]
pub struct QuizSessionModel {
    pub id: i32,
    pub quiz_id: i32,
    pub name: String,
    pub question_count: Option<i32>,
    pub selection_mode: Option<String>,
}

#[derive(sqlx::FromRow)]
pub struct AnswerModel {
    pub question: String,
    pub is_correct: bool,
    pub question_idx: i32,
    pub is_bookmarked: bool,
}

#[derive(sqlx::FromRow)]
pub struct QuestionStatsModel {
    pub question: String,
    pub correct_answers: i64,
}

#[derive(sqlx::FromRow)]
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

#[derive(sqlx::FromRow)]
pub struct CategoryStats {
    pub category: String,
    pub total: i64,
    pub correct: i64,
    pub accuracy: f64,
}

#[derive(sqlx::FromRow)]
pub struct QuizOverallStats {
    pub total_questions: i64,
    pub unique_asked: i64,
    pub total_correct: i64,
    pub total_answered: i64,
}

#[derive(sqlx::FromRow)]
pub struct DailyAccuracy {
    pub date_label: String,
    pub accuracy: f64,
}

#[derive(sqlx::FromRow)]
pub struct QuizCategoryOverallStats {
    pub category: String,
    pub total_in_category: i64,
    pub unique_asked: i64,
    pub total_correct: i64,
    pub total_answered: i64,
}

/// Combined question context from session_questions + questions + quizzes (single JOIN query)
#[derive(sqlx::FromRow)]
pub struct QuestionContext {
    pub quiz_name: String,
    pub quiz_public_id: String,
    pub question_id: i32,
    pub question: String,
    pub is_multiple_choice: bool,
    pub is_answered: bool,
    pub is_bookmarked: bool,
    pub questions_count: i32,
}

/// Option with selection status for the current session (avoids separate selected_answers query)
#[derive(sqlx::FromRow)]
pub struct OptionWithSelection {
    pub id: i32,
    pub is_answer: bool,
    pub option: String,
    pub explanation: Option<String>,
    pub is_selected: bool,
}
