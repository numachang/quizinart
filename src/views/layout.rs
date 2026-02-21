use maud::{html, Markup, PreEscaped, DOCTYPE};
use rust_i18n::t;

use crate::names;

fn css() -> Markup {
    html! {
        link rel="stylesheet" href="/static/pico.min.css";
        link rel="stylesheet" href="/static/index.css";
    }
}

const THEME_SCRIPT: &str = r#"
(() => {
    const key = "quizinart-theme";

    const getPreferredTheme = () => {
        const saved = localStorage.getItem(key);
        if (saved === "light" || saved === "dark" || saved === "system") {
            return saved;
        }
        return "system";
    };

    const resolveTheme = (mode) => {
        if (mode === "system") {
            return window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
        }
        return mode;
    };

    const applyTheme = (mode) => {
        const theme = resolveTheme(mode);
        document.documentElement.setAttribute("data-theme", theme);
        document.documentElement.setAttribute("data-theme-mode", mode);
    };

    const mode = getPreferredTheme();
    applyTheme(mode);

    document.addEventListener("DOMContentLoaded", () => {
        const select = document.querySelector(".theme-select");
        if (!select) { return; }

        select.value = mode;
        select.addEventListener("change", (event) => {
            const selectedMode = event.target.value;
            localStorage.setItem(key, selectedMode);
            applyTheme(selectedMode);
        });
    });

    window.matchMedia("(prefers-color-scheme: dark)").addEventListener("change", () => {
        const currentMode = getPreferredTheme();
        if (currentMode === "system") {
            applyTheme(currentMode);
        }
    });
})();
"#;

const MENU_SCRIPT: &str = r#"
(() => {
    document.addEventListener("DOMContentLoaded", () => {
        const btn = document.getElementById("nav-toggle");
        const menu = document.getElementById("nav-menu");
        if (!btn || !menu) return;

        btn.addEventListener("click", (e) => {
            e.stopPropagation();
            const open = menu.classList.toggle("open");
            btn.setAttribute("aria-expanded", open);
        });

        document.addEventListener("click", (e) => {
            if (!menu.classList.contains("open")) return;
            if (!menu.contains(e.target) && e.target !== btn) {
                menu.classList.remove("open");
                btn.setAttribute("aria-expanded", "false");
            }
        });

        document.addEventListener("keydown", (e) => {
            if (e.key === "Escape" && menu.classList.contains("open")) {
                menu.classList.remove("open");
                btn.setAttribute("aria-expanded", "false");
                btn.focus();
            }
        });
    });
})();
"#;

fn js() -> Markup {
    html! {
        script { (PreEscaped(THEME_SCRIPT)) }
        script { (PreEscaped(MENU_SCRIPT)) }
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
