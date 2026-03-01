use super::selection_mode_label;
use crate::{
    db::{
        AnswerModel, CategoryStats, DailyAccuracy, QuizCategoryOverallStats, QuizOverallStats,
        SessionReportModel,
    },
    names,
};
use maud::{html, Markup};
use rust_i18n::t;

pub struct DashboardData {
    pub quiz_name: String,
    pub quiz_id: String,
    pub sessions_count: i32,
    pub overall: QuizOverallStats,
    pub cat_stats: Vec<QuizCategoryOverallStats>,
    pub daily_accuracy: Vec<DailyAccuracy>,
    pub study_time_ms: i64,
}

pub struct SessionHistoryData {
    pub quiz_name: String,
    pub quiz_id: String,
    pub sessions: Vec<SessionReportModel>,
}

pub struct SessionResultData {
    pub session_name: String,
    pub session_id: i32,
    pub quiz_id: String,
    pub quiz_name: String,
    pub selection_mode: String,
    pub questions_count: i32,
    pub answered_count: i32,
    pub is_complete: bool,
    pub correct_answers: i32,
    pub answers: Vec<AnswerModel>,
    pub category_stats: Vec<CategoryStats>,
    pub study_time_ms: i64,
}

pub fn dashboard(data: DashboardData, locale: &str) -> Markup {
    let overall_accuracy = if data.overall.total_answered > 0 {
        data.overall.total_correct as f64 * 100.0 / data.overall.total_answered as f64
    } else {
        0.0
    };

    html! {
        a."back-link" hx-get="/" hx-push-url="true" hx-target="main" href="#" {
            span."material-symbols-rounded" { "arrow_back" }
            (t!("dashboard.back_to_quiz_list", locale = locale))
        }
        h1 { (data.quiz_name) }

        div style="display:flex; gap:1rem; margin-bottom:1rem; flex-wrap:wrap;" {
            button hx-get=(names::quiz_page_url(&data.quiz_id))
                   hx-push-url="true"
                   hx-target="main"
                   style="width: fit-content; background: var(--btn-gradient); color: white; border: none; font-weight: 500;" {
                (t!("dashboard.start_new", locale = locale))
            }
            button hx-get=(names::quiz_session_history_url(&data.quiz_id))
                   hx-push-url="true"
                   hx-target="main"
                   style="width: fit-content;" {
                (t!("dashboard.open_session_history", locale = locale))
            }
        }

        article {
            h4 { (t!("dashboard.overall_stats", locale = locale)) }
            div style="display:flex; align-items:center; gap:2rem; flex-wrap:wrap; justify-content:center;" {
                @if data.overall.total_answered > 0 {
                    div style="text-align:center;" {
                        canvas id="answered-chart" width="150" height="150" {}
                        div style="font-size:0.85rem; margin-top:0.25rem; color: var(--color-muted);" {
                            (t!("dashboard.questions_asked", locale = locale))
                        }
                    }
                    div style="text-align:center;" {
                        canvas id="accuracy-chart" width="150" height="150" {}
                        div style="font-size:0.85rem; margin-top:0.25rem; color: var(--color-muted);" {
                            (t!("dashboard.accuracy", locale = locale))
                        }
                    }
                }
                div style="min-width:180px;" {
                    table style="margin:0;" {
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
                            @if data.study_time_ms > 0 {
                                tr {
                                    td { (t!("dashboard.study_time", locale = locale)) }
                                    td { strong { (format_study_time(data.study_time_ms)) } }
                                }
                            }
                            @let remaining = data.overall.total_questions - data.overall.unique_asked;
                            tr {
                                td { (t!("dashboard.est_remaining", locale = locale)) }
                                td { strong {
                                    @if remaining > 0 {
                                        @let avg_ms = if data.overall.unique_asked > 0 && data.study_time_ms > 0 {
                                            data.study_time_ms / data.overall.unique_asked
                                        } else {
                                            30_000
                                        };
                                        @let est_ms = avg_ms * remaining;
                                        "~" (format_study_time(est_ms))
                                    } @else {
                                        (t!("dashboard.all_answered", locale = locale))
                                    }
                                } }
                            }
                        }
                    }
                }
            }
        }

        @let has_answered_cats = data.cat_stats.iter().any(|c| c.total_answered > 0);
        @if data.overall.total_answered > 0 || has_answered_cats || !data.daily_accuracy.is_empty() {
            (charts_data(&data.cat_stats, &data.daily_accuracy, &data.overall, locale))
        }
        @if !data.daily_accuracy.is_empty() || has_answered_cats {
            div style="display:flex; gap:1rem; flex-wrap:wrap;" {
                @if !data.daily_accuracy.is_empty() {
                    article style="flex:1; min-width:300px;" {
                        h4 { (t!("dashboard.daily_trend_title", locale = locale)) }
                        div style="position: relative; width: 100%; max-height: 300px;" {
                            canvas id="daily-chart" {}
                        }
                    }
                }
                @if has_answered_cats {
                    article style="flex:1; min-width:300px;" {
                        h4 { (t!("dashboard.radar_title", locale = locale)) }
                        div style="position: relative; width: 100%; max-height: 350px;" {
                            canvas id="radar-chart" {}
                        }
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
                                        small style="color: var(--color-muted);" {
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

    }
}

pub fn session_history(data: SessionHistoryData, locale: &str) -> Markup {
    html! {
        h1 { (data.quiz_name) }
        div style="margin-bottom: 1rem;" {
            button hx-get=(names::quiz_dashboard_url(&data.quiz_id))
                   hx-push-url="true"
                   hx-target="main"
                   style="width: fit-content;" {
                (t!("dashboard.back_to_dashboard", locale = locale))
            }
        }
        article {
            h4 { (t!("dashboard.session_history", locale = locale)) }
            @if data.sessions.is_empty() {
                p { (t!("dashboard.no_sessions", locale = locale)) }
            } @else {
                (session_history_table(&data.sessions, locale))
            }
        }

        div style="margin-top: 2rem;" {
            button hx-get=(names::quiz_dashboard_url(&data.quiz_id))
                   hx-push-url="true"
                   hx-target="main"
                   style="width: fit-content;" {
                (t!("dashboard.back_to_dashboard", locale = locale))
            }
        }
    }
}

fn session_history_table(sessions: &[SessionReportModel], locale: &str) -> Markup {
    html! {
        table {
            thead { tr {
                th { (t!("dashboard.name", locale = locale)) }
                th { (t!("dashboard.mode", locale = locale)) }
                th { (t!("dashboard.progress", locale = locale)) }
                th { (t!("dashboard.score", locale = locale)) }
                th { (t!("dashboard.status", locale = locale)) }
                th { (t!("dashboard.actions", locale = locale)) }
            } }
            tbody {
                @for s in sessions {
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
                                span."badge-status badge-complete" { (t!("dashboard.complete", locale = locale)) }
                            } @else {
                                span."badge-status badge-progress" { (t!("dashboard.in_progress", locale = locale)) }
                            }
                        }
                        td style="white-space: nowrap;" {
                            a."material-symbols-rounded"
                              data-rename-name=(s.name)
                              data-rename-url=(names::rename_session_url(s.id))
                              title=(t!("dashboard.rename_btn", locale = locale))
                              style="cursor: pointer; font-size: 1.2rem; opacity: 0.5; transition: opacity 0.15s; margin-right: 0.5rem;" {
                                "edit"
                            }
                            a."material-symbols-rounded"
                              hx-delete=(names::delete_session_url(s.id))
                              hx-target="main"
                              hx-swap="innerHTML"
                              hx-confirm=(t!("dashboard.delete_session_confirm", locale = locale))
                              title=(t!("dashboard.delete_btn", locale = locale))
                              style="cursor: pointer; color: var(--pico-del-color, #dc3545); font-size: 1.2rem; opacity: 0.5; transition: opacity 0.15s;" {
                                "delete"
                            }
                        }
                    }
                }
            }
        }

        dialog id="rename-dialog" {
            article {
                p { (t!("dashboard.rename_prompt", locale = locale)) }
                input id="rename-input" type="text" required="true" autocomplete="off";
                input id="rename-url" type="hidden";
                footer style="display: flex; gap: 0.5rem; justify-content: flex-end;" {
                    button data-dialog-close="rename-dialog"
                           class="secondary" {
                        (t!("quiz.abandon_cancel", locale = locale))
                    }
                    button data-rename-submit="" {
                        (t!("dashboard.rename_btn", locale = locale))
                    }
                }
            }
        }
    }
}

fn charts_data(
    cat_stats: &[QuizCategoryOverallStats],
    daily: &[DailyAccuracy],
    overall: &QuizOverallStats,
    locale: &str,
) -> Markup {
    let unique_asked = overall.unique_asked;
    let total_questions = overall.total_questions;
    let remaining_questions = total_questions - unique_asked;
    let total_correct = overall.total_correct;
    let total_answered = overall.total_answered;
    let total_incorrect = total_answered - total_correct;
    let overall_accuracy = if total_answered > 0 {
        (total_correct as f64 * 1000.0 / total_answered as f64).round() / 10.0
    } else {
        0.0
    };

    let radar_labels: Vec<&str> = cat_stats
        .iter()
        .filter(|c| c.total_answered > 0)
        .map(|c| c.category.as_str())
        .collect();
    let radar_data: Vec<f64> = cat_stats
        .iter()
        .filter(|c| c.total_answered > 0)
        .map(|c| (c.total_correct as f64 * 1000.0 / c.total_answered as f64).round() / 10.0)
        .collect();

    let daily_labels: Vec<&str> = daily.iter().map(|d| d.date_label.as_str()).collect();
    let daily_data: Vec<f64> = daily.iter().map(|d| d.accuracy).collect();

    let config = serde_json::json!({
        "uniqueAsked": unique_asked,
        "remainingQuestions": remaining_questions,
        "answeredCenter": format!("{}/{}", unique_asked, total_questions),
        "totalCorrect": total_correct,
        "totalIncorrect": total_incorrect,
        "accuracyCenter": format!("{:.1}%", overall_accuracy),
        "radarLabels": radar_labels,
        "radarData": radar_data,
        "dailyLabels": daily_labels,
        "dailyData": daily_data,
        "yLabel": t!("dashboard.progress_graph_yaxis", locale = locale).to_string(),
        "xLabel": t!("dashboard.daily_trend_xaxis", locale = locale).to_string(),
    });

    html! {
        div id="chart-data" data-config=(config.to_string()) style="display:none;" {}
    }
}

pub fn format_study_time(ms: i64) -> String {
    let total_secs = ms / 1000;
    let hours = total_secs / 3600;
    let minutes = (total_secs % 3600) / 60;
    let seconds = total_secs % 60;
    if hours > 0 {
        format!("{hours}:{minutes:02}:{seconds:02}")
    } else {
        format!("{minutes}:{seconds:02}")
    }
}

pub fn session_result(data: SessionResultData, locale: &str) -> Markup {
    let mode_label = selection_mode_label(&data.selection_mode, locale);
    let incorrect_count = data.answers.iter().filter(|a| !a.is_correct).count();
    let bookmarked_count = data.answers.iter().filter(|a| a.is_bookmarked).count();
    let percentage = if data.answered_count > 0 {
        data.correct_answers as f64 * 100.0 / data.answered_count as f64
    } else {
        0.0
    };

    html! {
        h5 { mark { (data.quiz_name) } }
        p style="color: var(--color-muted); font-size: 0.9rem;" {
            (t!("result.mode", locale = locale)) strong { (mode_label) }
            (t!("result.questions", locale = locale)) strong { (data.questions_count) }
        }

        @if !data.is_complete {
            article style="background-color: var(--color-warning-bg); border: 2px solid var(--color-warning); padding: 1rem; border-radius: 8px;" {
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
            @if data.study_time_ms > 0 {
                p {
                    (t!("result.study_time", locale = locale))
                    mark { (format_study_time(data.study_time_ms)) }
                }
            }
        }

        @if incorrect_count > 0 && data.is_complete {
            article."article-narrow" {
                h4 { (t!("result.retry_title", locale = locale)) }
                p {
                    (t!("result.wrong_count_1", locale = locale))
                    (incorrect_count)
                    (t!("result.wrong_count_2", locale = locale))
                }
                button hx-post=(format!("/retry-incorrect/{}", data.session_id))
                       hx-target="main"
                       hx-swap="innerHTML"
                       style="width: fit-content; background-color: var(--color-danger); color: white; font-weight: 500;" {
                    (t!("result.retry_btn_1", locale = locale))
                    (incorrect_count)
                    (t!("result.retry_btn_2", locale = locale))
                }
            }
        }

        @if bookmarked_count > 0 && data.is_complete {
            article."article-narrow" {
                h4 { (t!("result.retry_bookmarked_title", locale = locale)) }
                p {
                    (t!("result.bookmarked_count_1", locale = locale))
                    (bookmarked_count)
                    (t!("result.bookmarked_count_2", locale = locale))
                }
                button hx-post=(format!("/retry-bookmarked/{}", data.session_id))
                       hx-target="main"
                       hx-swap="innerHTML"
                       style="width: fit-content; background-color: var(--color-warning); color: white; font-weight: 500;" {
                    (t!("result.retry_bookmarked_btn_1", locale = locale))
                    (bookmarked_count)
                    (t!("result.retry_bookmarked_btn_2", locale = locale))
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
                    th { span."material-symbols-rounded" style="font-size: 1.1rem;" { "bookmark" } }
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
                            td {
                                span."material-symbols-rounded" style=(if a.is_correct { "color: var(--color-success); font-size: 1.1rem;" } else { "color: var(--color-danger); font-size: 1.1rem;" }) {
                                    (if a.is_correct { "check_circle" } else { "cancel" })
                                }
                            }
                            td {
                                @if a.is_bookmarked {
                                    span."material-symbols-rounded" style="font-size: 1.1rem;" {
                                        "bookmark"
                                    }
                                }
                            }
                         }
                    }
                }
            }
        }

        div style="margin-top: 2rem;" {
            button hx-get=(names::quiz_dashboard_url(&data.quiz_id))
                   hx-push-url="true"
                   hx-target="main"
                   style="width: fit-content;" {
                (t!("result.back_to_dashboard", locale = locale))
            }
        }
    }
}
