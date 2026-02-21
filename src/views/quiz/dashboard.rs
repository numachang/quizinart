use super::selection_mode_label;
use crate::{
    db::{
        AnswerModel, CategoryStats, DailyAccuracy, QuizCategoryOverallStats, QuizOverallStats,
        SessionReportModel,
    },
    names,
};
use maud::{html, Markup, PreEscaped};
use rust_i18n::t;

pub struct DashboardData {
    pub quiz_name: String,
    pub quiz_id: i32,
    pub sessions_count: i32,
    pub overall: QuizOverallStats,
    pub cat_stats: Vec<QuizCategoryOverallStats>,
    pub daily_accuracy: Vec<DailyAccuracy>,
}

pub struct SessionHistoryData {
    pub quiz_name: String,
    pub quiz_id: i32,
    pub sessions: Vec<SessionReportModel>,
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

        div style="display:flex; gap:1rem; margin-bottom:1rem; flex-wrap:wrap;" {
            button hx-get=(names::quiz_page_url(data.quiz_id))
                   hx-push-url="true"
                   hx-target="main"
                   style="width: fit-content; background-color: #007bff; color: white; font-weight: 500;" {
                (t!("dashboard.start_new", locale = locale))
            }
            button hx-get=(names::quiz_session_history_url(data.quiz_id))
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
                        div style="font-size:0.85rem; margin-top:0.25rem; color:#666;" {
                            (t!("dashboard.questions_asked", locale = locale))
                        }
                    }
                    div style="text-align:center;" {
                        canvas id="accuracy-chart" width="150" height="150" {}
                        div style="font-size:0.85rem; margin-top:0.25rem; color:#666;" {
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
                        }
                    }
                }
            }
        }

        @let has_answered_cats = data.cat_stats.iter().any(|c| c.total_answered > 0);
        @if data.overall.total_answered > 0 || has_answered_cats || !data.daily_accuracy.is_empty() {
            (charts_script(&data.cat_stats, &data.daily_accuracy, &data.overall, locale))
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

    }
}

pub fn session_history(data: SessionHistoryData, locale: &str) -> Markup {
    html! {
        h1 { (data.quiz_name) }
        div style="margin-bottom: 1rem;" {
            button hx-get=(names::quiz_dashboard_url(data.quiz_id))
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
            button hx-get=(names::quiz_dashboard_url(data.quiz_id))
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

fn charts_script(
    cat_stats: &[QuizCategoryOverallStats],
    daily: &[DailyAccuracy],
    overall: &QuizOverallStats,
    locale: &str,
) -> Markup {
    // Overall stats doughnut data
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
    let answered_center_text = format!("{}/{}", unique_asked, total_questions);
    let accuracy_center_text = format!("{:.1}%", overall_accuracy);

    // Radar chart data from cat_stats
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
    let radar_labels_json = serde_json::to_string(&radar_labels).unwrap_or_default();
    let radar_data_json = serde_json::to_string(&radar_data).unwrap_or_default();

    // Daily line chart data
    let daily_labels: Vec<&str> = daily.iter().map(|d| d.date_label.as_str()).collect();
    let daily_data: Vec<f64> = daily.iter().map(|d| d.accuracy).collect();
    let daily_labels_json = serde_json::to_string(&daily_labels).unwrap_or_default();
    let daily_data_json = serde_json::to_string(&daily_data).unwrap_or_default();

    let y_label =
        serde_json::to_string(&t!("dashboard.progress_graph_yaxis", locale = locale).to_string())
            .unwrap_or_default();
    let x_label =
        serde_json::to_string(&t!("dashboard.daily_trend_xaxis", locale = locale).to_string())
            .unwrap_or_default();

    let script = format!(
        r#"(function(){{
var s=document.createElement('script');
s.src='/static/chart.min.js';
s.onload=function(){{
var tc=getComputedStyle(document.documentElement).color;
var ac=document.getElementById('answered-chart');
if(ac)new Chart(ac,{{type:'doughnut',data:{{datasets:[{{data:[{unique_asked},{remaining_questions}],backgroundColor:['#4e79a7','#e0e0e0'],borderWidth:0}}]}},plugins:[{{id:'ct1',afterDraw:function(c){{var x=c.ctx;x.save();x.fillStyle=tc;x.font='bold 1.1rem sans-serif';x.textAlign='center';x.textBaseline='middle';var cx=(c.chartArea.left+c.chartArea.right)/2;var cy=(c.chartArea.top+c.chartArea.bottom)/2;x.fillText('{answered_center_text}',cx,cy);x.restore();}}}}],options:{{responsive:false,cutout:'70%',plugins:{{legend:{{display:false}},tooltip:{{enabled:false}}}}}}}});
var gc=document.getElementById('accuracy-chart');
if(gc)new Chart(gc,{{type:'doughnut',data:{{datasets:[{{data:[{total_correct},{total_incorrect}],backgroundColor:['#59a14f','#e0e0e0'],borderWidth:0}}]}},plugins:[{{id:'ct2',afterDraw:function(c){{var x=c.ctx;x.save();x.fillStyle=tc;x.font='bold 1.3rem sans-serif';x.textAlign='center';x.textBaseline='middle';var cx=(c.chartArea.left+c.chartArea.right)/2;var cy=(c.chartArea.top+c.chartArea.bottom)/2;x.fillText('{accuracy_center_text}',cx,cy);x.restore();}}}}],options:{{responsive:false,cutout:'70%',plugins:{{legend:{{display:false}},tooltip:{{enabled:false}}}}}}}});
var r=document.getElementById('radar-chart');
if(r)new Chart(r,{{type:'radar',data:{{labels:{radar_labels_json},datasets:[{{data:{radar_data_json},backgroundColor:'rgba(78,121,167,0.2)',borderColor:'#4e79a7',pointBackgroundColor:'#4e79a7',borderWidth:2}}]}},options:{{responsive:true,plugins:{{legend:{{display:false}}}},scales:{{r:{{min:0,max:100,ticks:{{stepSize:20}}}}}}}}}});
var d=document.getElementById('daily-chart');
if(d)new Chart(d,{{type:'line',data:{{labels:{daily_labels_json},datasets:[{{data:{daily_data_json},borderColor:'#4e79a7',backgroundColor:'rgba(78,121,167,0.1)',fill:true,tension:0.3,pointRadius:4,pointHoverRadius:6}}]}},options:{{responsive:true,plugins:{{legend:{{display:false}}}},scales:{{y:{{min:0,max:100,title:{{display:true,text:{y_label}}}}},x:{{title:{{display:true,text:{x_label}}}}}}}}}}});
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
