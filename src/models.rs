use serde::Deserialize;

pub type Questions = Vec<Question>;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Question {
    pub question: String,
    pub category: Option<String>,
    #[serde(default)]
    pub is_multiple_choice: bool,
    pub options: Vec<QuestionOption>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuestionOption {
    pub text: String,
    pub is_answer: bool,
    pub explanation: Option<String>,
}
