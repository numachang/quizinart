use maud::{html, Markup};

/// htmx navigation link with href fallback + hx-get for in-page swap.
pub fn nav_link(href: &str, body: Markup) -> Markup {
    html! {
        a href=(href)
          hx-get=(href)
          hx-target="main"
          hx-push-url="true"
          hx-swap="innerHTML" {
            (body)
        }
    }
}
