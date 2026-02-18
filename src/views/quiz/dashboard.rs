use maud::{html, Markup};
use rust_i18n::t;
use crate::{
    db::{
        AnswerModel, CategoryStats, QuizCategoryOverallStats,
        QuizOverallStats, SessionReportModel,
    },
    names,
};
use super::selection_mode_label;

pub struct DashboardData {
    pub quiz_name: String,
    pub quiz_id: i32,
    pub sessions_count: i32,
    pub sessions: Vec<SessionReportModel>,
    pub overall: QuizOverallStats,
    pub cat_stats: Vec<QuizCategoryOverallStats>,
}

pub struct SessionResultData {
    pub session_name: String,
    pub session_id: i32,
    pub quiz_id: i32,
    pub quiz_name: String,
    pub selection_mode: String,
    pub questions_count: i32,
    pub answered_count: i32,
    pub is_complete: bool,
    pub correct_answers: i32,
    pub answers: Vec<AnswerModel>,
    pub category_stats: Vec<CategoryStats>,
}

pub fn dashboard(data: DashboardData, locale: &str) -> Markup {
    let overall_accuracy = if data.overall.total_answered > 0 {
        data.overall.total_correct as f64 * 100.0 / data.overall.total_answered as f64
    } else {
        0.0
    };

    html! {
        h1 { (data.quiz_name) }

        div style="margin-bottom: 1rem;" {
            button hx-get=(names::quiz_page_url(data.quiz_id))
                   hx-push-url="true"
                   hx-target="main"
                   style="width: fit-content; background-color: #007bff; color: white; font-weight: 500;" {
                (t!("dashboard.start_new", locale = locale))
            }
        }

        article {
            h4 { (t!("dashboard.overall_stats", locale = locale)) }
            table {
                tbody {
                    tr {
                        td { (t!("dashboard.total_questions", locale = locale)) }
                        td { strong { (data.overall.total_questions) } }
                    }
                    tr {
                        td { (t!("dashboard.questions_asked", locale = locale)) }
                        td { strong { (data.overall.unique_asked) } " / " (data.overall.total_questions) }
                    }
                    tr {
                        td { (t!("dashboard.total_answers", locale = locale)) }
                        td { strong { (data.overall.total_answered) } }
                    }
                    tr {
                        td { (t!("dashboard.accuracy", locale = locale)) }
                        td { strong { (format!("{:.1}%", overall_accuracy)) }
                            " (" (data.overall.total_correct) " / " (data.overall.total_answered) ")"
                        }
                    }
                    tr {
                        td { (t!("dashboard.sessions", locale = locale)) }
                        td { strong { (data.sessions_count) } }
                    }
                }
            }
        }

        @if !data.cat_stats.is_empty() {
            article {
                h4 { (t!("dashboard.category_stats", locale = locale)) }
                table {
                    thead { tr {
                        th { (t!("dashboard.category", locale = locale)) }
                        th { (t!("dashboard.questions", locale = locale)) }
                        th { (t!("dashboard.asked", locale = locale)) }
                        th { (t!("dashboard.accuracy", locale = locale)) }
                    } }
                    tbody {
                        @for c in &data.cat_stats {
                            @let acc = if c.total_answered > 0 {
                                c.total_correct as f64 * 100.0 / c.total_answered as f64
                            } else {
                                0.0
                            };
                            tr {
                                td { (c.category) }
                                td { (c.total_in_category) }
                                td { (c.unique_asked) " / " (c.total_in_category) }
                                td {
                                    (format!("{:.1}%", acc))
                                    @if c.total_answered > 0 {
                                        small style="color: #666;" {
                                            " (" (c.total_correct) "/" (c.total_answered) ")"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        @if !data.sessions.is_empty() {
            article {
                h4 { (t!("dashboard.session_history", locale = locale)) }
                table {
                    thead { tr {
                        th { (t!("dashboard.name", locale = locale)) }
                        th { (t!("dashboard.mode", locale = locale)) }
                        th { (t!("dashboard.progress", locale = locale)) }
                        th { (t!("dashboard.score", locale = locale)) }
                        th { (t!("dashboard.status", locale = locale)) }
                    } }
                    tbody {
                        @for s in &data.sessions {
                            tr {
                                td { (s.name) }
                                td {
                                    (selection_mode_label(s.selection_mode.as_deref().unwrap_or("random"), locale))
                                }
                                td {
                                    @if s.is_complete {
                                        a hx-get=(names::results_url(s.id))
                                           hx-push-url="true"
                                           hx-target="main"
                                           href="#" {
                                            (s.answered_questions) "/" (s.total_questions)
                                        }
                                    } @else {
                                        a hx-get=(names::resume_session_url(s.id, &s.session_token))
                                           hx-push-url="true"
                                           hx-target="main"
                                           href="#" {
                                            (s.answered_questions) "/" (s.total_questions)
                                        }
                                    }
                                }
                                td {
                                    a hx-get=(names::results_url(s.id))
                                       hx-push-url="true"
                                       hx-target="main"
                                       href="#" {
                                        (s.score) "/" (s.answered_questions)
                                    }
                                }
                                td {
                                    @if s.is_complete {
                                        span style="color: #28a745; font-weight: 500;" { (t!("dashboard.complete", locale = locale)) }
                                    } @else {
                                        span style="color: #6c757d; font-weight: 500;" { (t!("dashboard.in_progress", locale = locale)) }
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

pub fn session_result(data: SessionResultData, locale: &str) -> Markup {
    let mode_label = selection_mode_label(&data.selection_mode, locale);
    let incorrect_count = data.answers.iter().filter(|a| !a.is_correct).count();
    let percentage = if data.answered_count > 0 {
        data.correct_answers as f64 * 100.0 / data.answered_count as f64
    } else {
        0.0
    };

    html! {
        h5 { mark { (data.quiz_name) } }
        p style="color: #666; font-size: 0.9rem;" {
            (t!("result.mode", locale = locale)) strong { (mode_label) }
            (t!("result.questions", locale = locale)) strong { (data.questions_count) }
        }

        @if !data.is_complete {
            article style="background-color: #fff3cd; border: 2px solid #f0ad4e; padding: 1rem; border-radius: 8px;" {
                h4 { (t!("result.quiz_in_progress", locale = locale)) }
                p {
                    (t!("result.answered_of_1", locale = locale))
                    mark { (data.answered_count) }
                    (t!("result.answered_of_2", locale = locale))
                    mark { (data.questions_count) }
                    (t!("result.answered_of_3", locale = locale))
                }
                p { (t!("result.partial_results", locale = locale)) }
            }
        }

        h1 {
            mark { (data.session_name) }
            @if !data.is_complete {
                (t!("result.progress_report", locale = locale))
            }
        }

        article {
            h4 { (t!("result.score", locale = locale)) }
            p {
                (t!("result.correct_label", locale = locale)) mark { (data.correct_answers) }
                (t!("result.answered_label", locale = locale)) mark { (data.answered_count) }
                @if data.is_complete {
                    (t!("result.total_label", locale = locale)) mark { (data.questions_count) }
                }
                " (" mark { (format!("{:.0}%", percentage)) } ")"
            }
        }

        @if incorrect_count > 0 && data.is_complete {
            article style="width: fit-content;" {
                h4 { (t!("result.retry_title", locale = locale)) }
                p {
                    (t!("result.wrong_count_1", locale = locale))
                    (incorrect_count)
                    (t!("result.wrong_count_2", locale = locale))
                }
                button hx-post=(format!("/retry-incorrect/{}", data.session_id))
                       hx-target="main"
                       hx-swap="innerHTML"
                       style="width: fit-content; background-color: #dc3545; color: white; font-weight: 500;" {
                    (t!("result.retry_btn_1", locale = locale))
                    (incorrect_count)
                    (t!("result.retry_btn_2", locale = locale))
                }
            }
        }

        @if !data.category_stats.is_empty() {
            article {
                h4 { (t!("result.category_perf", locale = locale)) }
                table {
                    thead { tr {
                        th { (t!("dashboard.category", locale = locale)) }
                        th { (t!("result.correct_total", locale = locale)) }
                        th { (t!("dashboard.accuracy", locale = locale)) }
                    } }
                    tbody {
                        @for stat in &data.category_stats {
                            tr {
                                td { (stat.category) }
                                td { (stat.correct) " / " (stat.total) }
                                td { (format!("{:.1}%", stat.accuracy)) }
                            }
                        }
                    }
                }
            }
        }

        article {
            h4 { (t!("result.all_questions", locale = locale)) }
            table {
                thead { tr {
                    th { (t!("result.question_hash", locale = locale)) }
                    th { (t!("result.question_col", locale = locale)) }
                    th { (t!("result.correct_col", locale = locale)) }
                } }
                tbody {
                    @for a in &data.answers {
                        @let url = if data.is_complete {
                            format!("/question/{}?question_idx={}&from=report", data.session_id, a.question_idx)
                        } else {
                            format!("/question/{}?question_idx={}&from=report&current_idx={}", data.session_id, a.question_idx, data.answered_count)
                        };
                        tr style="cursor: pointer;"
                           hx-get=(url)
                           hx-push-url="true"
                           hx-target="main" {
                            td { (a.question_idx + 1) }
                            td { (a.question) }
                            td { (if a.is_correct { "\u{1F7E2}" } else { "\u{1F534}" }) }
                         }
                    }
                }
            }
        }

        div style="margin-top: 2rem;" {
            button hx-get=(names::quiz_dashboard_url(data.quiz_id))
                   hx-push-url="true"
                   hx-target="main"
                   style="width: fit-content;" {
                (t!("result.back_to_dashboard", locale = locale))
            }
        }
    }
}
