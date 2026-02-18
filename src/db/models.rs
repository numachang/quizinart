// Database model structs

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

pub struct QuestionOptionModel {
    pub id: i32,
    pub is_answer: bool,
    pub option: String,
    pub explanation: Option<String>,
}

pub struct QuizSessionModel {
    pub id: i32,
    pub quiz_id: i32,
    pub name: String,
    pub question_count: Option<i32>,
    pub selection_mode: Option<String>,
}

pub struct AnswerModel {
    pub question: String,
    pub is_correct: bool,
    pub question_idx: i32,
}

pub struct QuestionStatsModel {
    pub question: String,
    pub correct_answers: i32,
}

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

pub struct CategoryStats {
    pub category: String,
    pub total: i32,
    pub correct: i32,
    pub accuracy: f64,
}

pub struct QuizOverallStats {
    pub total_questions: i32,
    pub unique_asked: i32,
    pub total_correct: i32,
    pub total_answered: i32,
}

pub struct SessionCategoryAccuracy {
    pub session_id: i32,
    pub session_name: String,
    pub category: String,
    pub accuracy: f64,
}

pub struct QuizCategoryOverallStats {
    pub category: String,
    pub total_in_category: i32,
    pub unique_asked: i32,
    pub total_correct: i32,
    pub total_answered: i32,
}
