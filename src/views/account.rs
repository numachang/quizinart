use crate::{db::models::AuthUser, names};
use maud::{html, Markup};
use rust_i18n::t;

pub enum ChangePasswordState {
    NoError,
    IncorrectPassword,
    EmptyFields,
    Success,
}

pub fn account_page(user: &AuthUser, state: ChangePasswordState, locale: &str) -> Markup {
    let (error_msg, success_msg) = match state {
        ChangePasswordState::NoError => (None, None),
        ChangePasswordState::IncorrectPassword => (
            Some(t!("account.incorrect_password", locale = locale).to_string()),
            None,
        ),
        ChangePasswordState::EmptyFields => (
            Some(t!("account.empty_fields", locale = locale).to_string()),
            None,
        ),
        ChangePasswordState::Success => (
            None,
            Some(t!("account.password_changed", locale = locale).to_string()),
        ),
    };

    html! {
        h1 { (t!("account.title", locale = locale)) }

        article style="width: fit-content;" {
            label {
                (t!("account.email_label", locale = locale))
                input type="email" value=(user.email) disabled="true";
            }
            label {
                (t!("account.display_name_label", locale = locale))
                input type="text" value=(user.display_name) disabled="true";
            }
        }

        h2 { (t!("account.change_password_title", locale = locale)) }

        @if let Some(ref msg) = success_msg {
            p style="color: var(--pico-ins-color);" { (msg) }
        }

        article style="width: fit-content;" {
            form hx-post=(names::CHANGE_PASSWORD_URL)
                 hx-ext="json-enc"
                 hx-target="main"
                 hx-disabled-elt="find input, find button"
                 hx-swap="innerHTML" {
                label {
                    (t!("account.current_password", locale = locale))
                    @if let Some(ref msg) = error_msg {
                        input name="current_password"
                              type="password"
                              autocomplete="current-password"
                              required="true"
                              placeholder=(t!("account.current_password", locale = locale))
                              aria-invalid="true"
                              aria-label=(t!("account.current_password", locale = locale));
                        small { (msg) }
                    } @else {
                        input name="current_password"
                              type="password"
                              autocomplete="current-password"
                              required="true"
                              placeholder=(t!("account.current_password", locale = locale))
                              aria-label=(t!("account.current_password", locale = locale));
                    }
                }
                label {
                    (t!("account.new_password", locale = locale))
                    input name="new_password"
                          type="password"
                          autocomplete="new-password"
                          required="true"
                          placeholder=(t!("account.new_password", locale = locale))
                          aria-label=(t!("account.new_password", locale = locale));
                }
                button type="submit" { (t!("account.change_password_btn", locale = locale)) }
            }
        }
    }
}
