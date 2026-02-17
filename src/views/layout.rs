use maud::{html, Markup, DOCTYPE};
use rust_i18n::t;

use crate::{names, utils};

fn css() -> Markup {
    html! {
        link rel="stylesheet" href="/static/pico.min.css";
        link rel="stylesheet" href="/static/index.css";
    }
}

fn js() -> Markup {
    html! {
        script src="/static/htmx/htmx.min.js" {}
        script src="/static/htmx/ext/json-enc.js" {}
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

fn header(locale: &str) -> Markup {
    html! {
        header {
            nav {
                ul {
                    li."secondary" {
                        a href="/" {
                            strong { "Quizinart" }
                        }
                    }
                }
                ul {
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
                    li."secondary" { (utils::VERSION) }
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
            (header(locale))
            (main(body))
        }
    }
}

pub fn titled(title: &str, body: Markup) -> Markup {
    html! {
        title { (title) " - Quizinart" }
        (body)
    }
}
