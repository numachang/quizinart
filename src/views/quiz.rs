use maud::{html, Markup};
use crate::names;

pub fn session_name_error_page(session_name: &str, quiz_id: i32) -> Markup {
    html! {
        article style="margin-top: 2rem;" {
            header {
                h2 { "❌ Session Name Already Exists" }
            }
            p style="color: #d32f2f; font-weight: 500;" {
                "The session name '"
                strong { (session_name) }
                "' is already in use for this quiz."
            }
            p {
                "Please choose a different session name. Session names must be unique for each quiz."
            }
            hr;
            h3 { "Suggestions:" }
            ul {
                li { "Add a date: " code { (session_name) "_2026_02_16" } }
                li { "Add a number: " code { (session_name) "_2" } }
                li { "Add a suffix: " code { (session_name) "_retry" } }
            }
            hr;
            button hx-get=(names::quiz_page_url(quiz_id))
                   hx-push-url="true"
                   hx-target="main"
                   style="width: fit-content; background-color: #007bff; color: white; margin-top: 1rem;" {
                "← Try Again with Different Name"
            }
        }
    }
}

// Note: Other view functions from handlers/quiz.rs can be moved here incrementally
// For now, we're keeping the complex view functions (question, page, dashboard)
// in handlers/quiz.rs until we can properly refactor them to separate data fetching from rendering
