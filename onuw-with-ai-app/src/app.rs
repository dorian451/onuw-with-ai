pub mod components;
pub mod pages;
pub mod util;

use self::pages::error::PageNotFound;
use self::pages::home::HomePage;
use leptos::*;
use leptos_meta::*;
use leptos_router::*;

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
        <Body class="items-center bg-white dark:bg-black text-black  dark:text-white"/>

        <Stylesheet id="leptos" href="/pkg/onuw-with-ai.css"/>
        // sets the document title
        <Title text="One Night Ultimate Werewolf"/>

        // content for this welcome page
        <Router fallback=|| PageNotFound().into_view()>
            <main>
                <Routes>
                    <Route path="" view=HomePage/>
                </Routes>
            </main>
        </Router>
    }
}

