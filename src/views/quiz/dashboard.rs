use std::collections::{BTreeMap, BTreeSet};

use super::selection_mode_label;
use crate::{
    db::{
        AnswerModel, CategoryStats, QuizCategoryOverallStats, QuizOverallStats,
        SessionCategoryAccuracy, SessionReportModel,
    },
    names,
};
use maud::{html, Markup, PreEscaped};
use rust_i18n::t;

pub struct DashboardData {
    pub quiz_name: String,
    pub quiz_id: i32,
    pub sessions_count: i32,
    pub sessions: Vec<SessionReportModel>,
    pub overall: QuizOverallStats,
    pub cat_stats: Vec<QuizCategoryOverallStats>,
    pub trends: Vec<SessionCategoryAccuracy>,
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

        @if !data.trends.is_empty() {
            article {
                h4 { (t!("dashboard.progress_graph", locale = locale)) }
                div style="position: relative; width: 100%; max-height: 400px;" {
                    canvas id="progress-chart" {}
                }
                (progress_chart_script(&data.trends, locale))
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
                        th { (t!("dashboard.actions", locale = locale)) }
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
                                td style="white-space: nowrap;" {
                                    @let safe_name = serde_json::to_string(&s.name).unwrap_or_default();
                                    @let prompt_label = serde_json::to_string(&t!("dashboard.rename_prompt", locale = locale).to_string()).unwrap_or_default();
                                    @let rename_js = format!(
                                        "var n=prompt({},{});if(n)htmx.ajax('PATCH','{}',{{target:'main',swap:'innerHTML',values:{{name:n}}}})",
                                        prompt_label, safe_name, names::rename_session_url(s.id),
                                    );
                                    button onclick=(rename_js)
                                           style="width:fit-content;padding:0.25rem 0.5rem;font-size:0.8rem;margin-right:0.25rem;" {
                                        (t!("dashboard.rename_btn", locale = locale))
                                    }
                                    button hx-delete=(names::delete_session_url(s.id))
                                           hx-target="main"
                                           hx-swap="innerHTML"
                                           hx-confirm=(t!("dashboard.delete_session_confirm", locale = locale))
                                           style="width:fit-content;padding:0.25rem 0.5rem;font-size:0.8rem;background-color:#dc3545;color:white;" {
                                        (t!("dashboard.delete_btn", locale = locale))
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

const CHART_COLORS: &[&str] = &[
    "#4e79a7", "#f28e2b", "#e15759", "#76b7b2", "#59a14f", "#edc948", "#b07aa1", "#ff9da7",
    "#9c755f", "#bab0ac",
];

fn progress_chart_script(trends: &[SessionCategoryAccuracy], locale: &str) -> Markup {
    // Collect ordered session labels and categories
    let mut session_labels: Vec<String> = Vec::new();
    let mut session_id_order: Vec<i32> = Vec::new();
    let mut categories: BTreeSet<String> = BTreeSet::new();

    for t in trends {
        if !session_id_order.contains(&t.session_id) {
            session_id_order.push(t.session_id);
            session_labels.push(t.session_name.clone());
        }
        categories.insert(t.category.clone());
    }

    // Build lookup: (session_id, category) -> accuracy
    let mut lookup: BTreeMap<(i32, &str), f64> = BTreeMap::new();
    for t in trends {
        lookup.insert((t.session_id, &t.category), t.accuracy);
    }

    // Build datasets JSON
    let labels_json = serde_json::to_string(&session_labels).unwrap_or_default();
    let mut datasets_json = String::from("[");
    for (i, cat) in categories.iter().enumerate() {
        let color = CHART_COLORS[i % CHART_COLORS.len()];
        let data_points: Vec<String> = session_id_order
            .iter()
            .map(|sid| match lookup.get(&(*sid, cat.as_str())) {
                Some(v) => format!("{v}"),
                None => "null".to_string(),
            })
            .collect();
        let cat_json = serde_json::to_string(cat).unwrap_or_default();
        if i > 0 {
            datasets_json.push(',');
        }
        datasets_json.push_str(&format!(
            "{{label:{cat_json},data:[{}],borderColor:'{color}',backgroundColor:'{color}',tension:0.3,spanGaps:true}}",
            data_points.join(",")
        ));
    }
    datasets_json.push(']');

    let y_label =
        serde_json::to_string(&t!("dashboard.progress_graph_yaxis", locale = locale).to_string())
            .unwrap_or_default();
    let session_label =
        serde_json::to_string(&t!("dashboard.progress_graph_session", locale = locale).to_string())
            .unwrap_or_default();

    let script = format!(
        r#"(function(){{
var s=document.createElement('script');
s.src='/static/chart.min.js';
s.onload=function(){{
var ctx=document.getElementById('progress-chart');
if(!ctx)return;
new Chart(ctx,{{type:'line',data:{{labels:{labels_json},datasets:{datasets_json}}},options:{{responsive:true,plugins:{{legend:{{position:'bottom'}}}},scales:{{y:{{min:0,max:100,title:{{display:true,text:{y_label}}}}},x:{{title:{{display:true,text:{session_label}}}}}}}}}}});
}};
document.head.appendChild(s);
}})()"#
    );

    html! {
        (PreEscaped(format!("<script>{script}</script>")))
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

        @if bookmarked_count > 0 && data.is_complete {
            article style="width: fit-content;" {
                h4 { (t!("result.retry_bookmarked_title", locale = locale)) }
                p {
                    (t!("result.bookmarked_count_1", locale = locale))
                    (bookmarked_count)
                    (t!("result.bookmarked_count_2", locale = locale))
                }
                button hx-post=(format!("/retry-bookmarked/{}", data.session_id))
                       hx-target="main"
                       hx-swap="innerHTML"
                       style="width: fit-content; background-color: #f0ad4e; color: white; font-weight: 500;" {
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
                    th { (t!("result.bookmark_col", locale = locale)) }
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
                            td { @if a.is_bookmarked { "\u{1F516}" } }
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
