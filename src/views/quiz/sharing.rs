use crate::{db::SharedQuizInfo, names};
use maud::{html, Markup};
use rust_i18n::t;

pub fn shared_quiz_page(info: &SharedQuizInfo, already_in_library: bool, locale: &str) -> Markup {
    html! {
        h1 { (info.name) }
        article {
            p {
                (t!("share.shared_by", locale = locale))
                strong { (info.owner_name) }
            }
            p {
                (info.question_count)
                (t!("share.questions_suffix", locale = locale))
            }
            @if already_in_library {
                p style="color: #28a745; font-weight: 500;" {
                    (t!("share.already_in_library", locale = locale))
                }
                button hx-get=(names::quiz_dashboard_url(&info.public_id))
                       hx-push-url="true"
                       hx-target="main"
                       style="width: fit-content; background-color: #007bff; color: white;" {
                    (t!("share.go_to_dashboard", locale = locale))
                }
            } @else {
                button hx-post=(names::add_to_library_url(&info.public_id))
                       hx-target="main"
                       hx-swap="innerHTML"
                       style="width: fit-content; background-color: #007bff; color: white; font-weight: 500;" {
                    (t!("share.add_to_library", locale = locale))
                }
            }
        }
    }
}

pub fn share_toggle_icon(public_id: &str, is_shared: bool, locale: &str) -> Markup {
    let (icon, title, style) = if is_shared {
        (
            "public",
            t!("share.shared_label", locale = locale).to_string(),
            "cursor: pointer; color: #28a745; font-size: 1.2rem; opacity: 0.8; transition: opacity 0.15s;",
        )
    } else {
        (
            "public_off",
            t!("share.share_btn", locale = locale).to_string(),
            "cursor: pointer; font-size: 1.2rem; opacity: 0.5; transition: opacity 0.15s;",
        )
    };
    html! {
        a."material-symbols-rounded"
          id=(format!("share-{public_id}"))
          data-share-toggle=""
          hx-post=(names::toggle_share_url(public_id))
          hx-target="this"
          hx-swap="outerHTML"
          title=(title)
          style=(style) {
            (icon)
        }
    }
}

pub fn shared_quiz_not_available(locale: &str) -> Markup {
    html! {
        h1 { (t!("share.not_available_title", locale = locale)) }
        p { (t!("share.not_available_desc", locale = locale)) }
        button hx-get="/"
               hx-push-url="true"
               hx-target="main"
               style="width: fit-content;" {
            (t!("share.back_to_home", locale = locale))
        }
    }
}
