use std::fmt::Debug;
use leptos::*;
use leptos::leptos_dom::ErrorKey;
use leptos_meta::*;
use leptos_router::*;
use uuid::{uuid, Uuid};
use crate::stats_page::{StatsPage,StatsPageParams};

#[component]
pub fn App(cx: Scope) -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context(cx);

    view! {
        cx,

        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/leptos_start.css"/>

        // sets the document title
        <Title text="Welcome to Leptos"/>

        // content for this welcome page
        <Router>
            <main>
                <ErrorBoundary
                    // the fallback receives a signal containing current errors
                    fallback=|cx, errors| view! { cx,
                        <div class="error">
                            <p>"An unexpected error occurred! Errors: "</p>
                            // we can render a list of errors as strings, if we'd like
                            <ul>
                                {move || errors.get()
                                    .into_iter()
                                    .map(|(_, e)| view! { cx, <li>{e.to_string()}</li>})
                                    .collect_view(cx)
                                }
                            </ul>
                        </div>
                    }
                >
                    <Routes>
                        <Route path="" view=|cx| view! { cx, <HomePage/> }/>
                        <Route path="stats/:mod_id" view=|cx| {
                            let params = use_params::<StatsPageParams>(cx);
                            view! { cx, <StatsPage params=params/> }
                        }/>
                    </Routes>
                </ErrorBoundary>
         </main>
        </Router>
    }
}

/// Renders the home page of your application.
#[component]
fn HomePage(cx: Scope) -> impl IntoView {
    // Creates a reactive value to update the button
    let (count, set_count) = create_signal(cx, 0);
    let on_click = move |_| set_count.update(|count| *count += 1);

    view! { cx,
        <div>
            <h1>"Welcome to Leptos!"</h1>
            <button on:click=on_click>"Click Me: " {count}</button>
        </div>
        <a href="./stats">"View mod stats"</a>
    }
}
