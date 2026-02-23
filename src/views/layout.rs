use maud::{html, Markup, DOCTYPE};
use rust_i18n::t;

use crate::names;

fn css() -> Markup {
    html! {
        link rel="stylesheet" href="/static/pico.min.css";
        link rel="stylesheet" href="/static/index.css";
        link rel="stylesheet" href="https://fonts.googleapis.com/css2?family=Material+Symbols+Rounded:opsz,wght,FILL,GRAD@20..48,100..700,0..1,-50..200&display=swap";
    }
}

fn js() -> Markup {
    html! {
        script src="/static/theme.js" {}
        script src="/static/menu.js" {}
        script src="/static/htmx/htmx.min.js" {}
        script src="/static/htmx/ext/json-enc.js" {}
        script src="/static/app.js" {}
    }
}

fn icon() -> Markup {
    html! {
        link rel="icon" href="/static/img/icon.svg" type="image/svg+xml" {}
    }
}

const LOCALES: &[(&str, &str)] = &[
    ("en", "layout.lang_toggle_en"),
    ("ja", "layout.lang_toggle_ja"),
    ("zh-CN", "layout.lang_toggle_zh_cn"),
    ("zh-TW", "layout.lang_toggle_zh_tw"),
];

const THEMES: &[(&str, &str)] = &[
    ("light", "layout.theme_light"),
    ("dark", "layout.theme_dark"),
    ("system", "layout.theme_system"),
];

fn header(locale: &str, user_name: Option<&str>) -> Markup {
    html! {
        header {
            nav {
                ul {
                    li."secondary" {
                        a href="/" {
                            strong { "Quizinart" }
                        }
                    }
                    @if user_name.is_some() {
                        li."secondary"."nav-feature-link" {
                            (super::components::nav_link(
                                "/",
                                html! { (t!("layout.my_quizzes", locale = locale)) },
                            ))
                        }
                    }
                    li."secondary"."nav-toggle-item" {
                        button
                            id="nav-toggle"
                            class="nav-toggle"
                            aria-label=(t!("layout.menu_toggle", locale = locale))
                            aria-expanded="false"
                            aria-controls="nav-menu" {
                            "\u{2630}"
                        }
                    }
                }
                ul id="nav-menu" {
                    li."secondary" {
                        select."theme-select"
                               name="theme"
                               aria-label=(t!("layout.theme_aria_label", locale = locale)) {
                            @for &(value, label_key) in THEMES {
                                @if value == "system" {
                                    option value=(value) selected { (t!(label_key, locale = locale)) }
                                } @else {
                                    option value=(value) { (t!(label_key, locale = locale)) }
                                }
                            }
                        }
                    }
                    li."secondary" {
                        select."lang-select"
                               name="locale"
                               hx-post=(names::SET_LOCALE_URL)
                               hx-ext="json-enc"
                               hx-include="this"
                               hx-trigger="change"
                               hx-swap="none"
                               aria-label="Language" {
                            @for &(code, label_key) in LOCALES {
                                @if code == locale {
                                    option value=(code) selected { (t!(label_key, locale = locale)) }
                                } @else {
                                    option value=(code) { (t!(label_key, locale = locale)) }
                                }
                            }
                        }
                    }
                    @if let Some(name) = user_name {
                        li."secondary" {
                            (super::components::nav_link(
                                names::ACCOUNT_URL,
                                html! { (name) },
                            ))
                        }
                        li."secondary" {
                            a role="button"
                              class="outline secondary"
                              hx-post=(names::LOGOUT_URL)
                              hx-swap="none"
                              style="padding: 0.25rem 0.5rem; font-size: 0.85rem;" {
                                (t!("homepage.logout", locale = locale))
                            }
                        }
                    }
                }
            }
        }
    }
}

fn main(body: Markup) -> Markup {
    html! {
        main { (body) }
    }
}

pub fn page(title: &str, body: Markup, locale: &str) -> Markup {
    page_with_user(title, body, locale, None)
}

pub fn page_with_user(title: &str, body: Markup, locale: &str, user_name: Option<&str>) -> Markup {
    html! {
        (DOCTYPE)
        head {
            meta charset="utf-8";
            meta name="viewport" content="width=device-width, initial-scale=1";
            meta name="color-scheme" content="light dark";

            (css())
            (js())
            (icon())

            title { (format!("{title} - Quizinart")) }
        }

        body."container" {
            (header(locale, user_name))
            (main(body))
        }
    }
}

pub fn render(
    is_htmx: bool,
    title: &str,
    body: Markup,
    locale: &str,
    user_name: Option<&str>,
) -> Markup {
    if is_htmx {
        titled(title, body)
    } else {
        page_with_user(title, body, locale, user_name)
    }
}

pub fn titled(title: &str, body: Markup) -> Markup {
    html! {
        title { (title) " - Quizinart" }
        (body)
    }
}
