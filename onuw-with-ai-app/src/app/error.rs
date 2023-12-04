use leptos::*;
use leptos_router::*;

#[component]
pub(super) fn PageNotFound() -> impl IntoView {
    view! {
        <h1>"Page not found"</h1>
        <A href="/">"Go to home page"</A>
    }
}
