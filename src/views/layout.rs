use maud::{html, Markup, DOCTYPE};

use crate::utils;

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

fn header() -> Markup {
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

pub fn page(title: &str, body: Markup) -> Markup {
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
            (header())
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
