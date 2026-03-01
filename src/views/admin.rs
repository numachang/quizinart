use crate::db::AdminUserStats;
use crate::views::quiz as quiz_views;
use maud::{html, Markup};
use rust_i18n::t;

pub fn dashboard(users: &[AdminUserStats], locale: &str) -> Markup {
    html! {
        h1 { (t!("admin.title", locale = locale)) }

        article {
            table {
                thead { tr {
                    th { (t!("admin.user", locale = locale)) }
                    th { (t!("admin.quiz_count", locale = locale)) }
                    th { (t!("admin.progress", locale = locale)) }
                    th { (t!("admin.study_time", locale = locale)) }
                } }
                tbody {
                    @for u in users {
                        @let progress_pct = if u.total_questions > 0 {
                            u.unique_asked as f64 * 100.0 / u.total_questions as f64
                        } else {
                            0.0
                        };
                        tr {
                            td { (u.display_name) }
                            td { (u.quiz_count) }
                            td {
                                (u.unique_asked) " / " (u.total_questions)
                                " (" (format!("{:.0}%", progress_pct)) ")"
                            }
                            td { (quiz_views::format_study_time(u.total_study_time_ms)) }
                        }
                    }
                }
            }
        }
    }
}
