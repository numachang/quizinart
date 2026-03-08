use maud::{html, Markup, DOCTYPE};
use rust_i18n::t;

use crate::names;

pub struct NavUser<'a> {
    pub display_name: &'a str,
    pub is_admin: bool,
}

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

fn header(locale: &str, nav_user: Option<&NavUser<'_>>) -> Markup {
    html! {
        header {
            nav {
                ul {
                    li."secondary" {
                        a href="/"
                          hx-get="/"
                          hx-target="main"
                          hx-push-url="true" {
                            strong { "Quizinart" }
                        }
                    }
                    @if nav_user.is_some() {
                        li."secondary"."nav-feature-link" {
                            (super::components::nav_link(
                                names::MARKETPLACE_URL,
                                html! { (t!("layout.marketplace", locale = locale)) },
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
                            span."material-symbols-rounded" { "menu" }
                        }
                    }
                }
                ul id="nav-menu" {
                    @if nav_user.is_some() {
                        li."secondary"."nav-menu-mobile-only" {
                            (super::components::nav_link(
                                names::MARKETPLACE_URL,
                                html! { (t!("layout.marketplace", locale = locale)) },
                            ))
                        }
                    }
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
                    @if let Some(user) = nav_user {
                        // Mobile: account, admin, logout directly in hamburger
                        li."secondary"."nav-menu-mobile-only" {
                            (super::components::nav_link(
                                names::ACCOUNT_URL,
                                html! { (user.display_name) },
                            ))
                        }
                        @if user.is_admin {
                            li."secondary"."nav-menu-mobile-only" {
                                (super::components::nav_link(
                                    names::ADMIN_URL,
                                    html! {
                                        span."material-symbols-rounded" style="font-size: 1rem; vertical-align: middle;" { "admin_panel_settings" }
                                        " " (t!("admin.go_to_admin", locale = locale))
                                    },
                                ))
                            }
                        }
                        li."secondary"."nav-menu-mobile-only" {
                            a hx-post=(names::LOGOUT_URL)
                              hx-swap="none"
                              href="#" {
                                (t!("homepage.logout", locale = locale))
                            }
                        }
                        // Desktop: settings gear dropdown
                        li."secondary"."settings-dropdown-item" {
                            div."settings-dropdown" {
                                button
                                    id="settings-toggle"
                                    class="settings-toggle"
                                    aria-label=(t!("layout.settings_menu", locale = locale))
                                    aria-expanded="false"
                                    aria-controls="settings-menu" {
                                    span."material-symbols-rounded" { "settings" }
                                }
                                div id="settings-menu" class="settings-menu" {
                                    (super::components::nav_link(
                                        names::ACCOUNT_URL,
                                        html! { (user.display_name) },
                                    ))
                                    @if user.is_admin {
                                        (super::components::nav_link(
                                            names::ADMIN_URL,
                                            html! {
                                                span."material-symbols-rounded" style="font-size: 1rem; vertical-align: middle;" { "admin_panel_settings" }
                                                " " (t!("admin.go_to_admin", locale = locale))
                                            },
                                        ))
                                    }
                                    a hx-post=(names::LOGOUT_URL)
                                      hx-swap="none"
                                      href="#" {
                                        (t!("homepage.logout", locale = locale))
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn confirm_dialog(locale: &str) -> Markup {
    html! {
        dialog id="confirm-dialog" {
            article {
                p data-confirm-message {}
                footer style="display: flex; justify-content: flex-end; gap: 0.5rem;" {
                    button."secondary" data-confirm-cancel {
                        (t!("layout.cancel", locale = locale))
                    }
                    button data-confirm-ok {
                        (t!("layout.ok", locale = locale))
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

pub fn page_with_user(
    title: &str,
    body: Markup,
    locale: &str,
    nav_user: Option<&NavUser<'_>>,
) -> Markup {
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
            div id="htmx-progress" {}
            (header(locale, nav_user))
            (main(body))
            (confirm_dialog(locale))
        }
    }
}

pub fn render(
    is_htmx: bool,
    title: &str,
    body: Markup,
    locale: &str,
    nav_user: Option<&NavUser<'_>>,
) -> Markup {
    if is_htmx {
        titled(title, body)
    } else {
        page_with_user(title, body, locale, nav_user)
    }
}

pub fn titled(title: &str, body: Markup) -> Markup {
    html! {
        title { (title) " - Quizinart" }
        (body)
    }
}
