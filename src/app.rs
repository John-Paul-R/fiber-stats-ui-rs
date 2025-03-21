use leptos::error::ErrorBoundary;
use leptos::logging::log;
use leptos::prelude::{
    signal, ClassAttribute, CollectView, ElementChild, Get, GlobalAttributes,
    OnAttribute, Update, *,
};
use leptos::*;
use leptos_meta::{Stylesheet, *};
use leptos_router::components::{Route, Router, Routes};
use leptos_router::hooks::use_params;
use leptos_router::*;

use crate::stats_page::{StatsPage, StatsPageParams};

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    logging::log!("where do I run?");

    view! {
        <Style media="screen" id="palette_light_dark" />
        <Style>
            """
                body#page_container {
                    width: 100vw;
                    height: 100vh;
                }
                main#content_main {
                    overflow: auto;
                }
                """
        </Style>
        <Script src="https://static.jpcode.dev/js/multi-palette.min.js" />

        // injects a stylesheet into the document <head>
        <Stylesheet id="leptos" href="/pkg/leptos_start.css"/>
        <Stylesheet href="https://www.fibermc.com/css/core_style.css?v=1.0.1"/>
        // <Stylesheet href="https://www.fibermc.com/css/search.css?v=1.0.1"/>
        <Stylesheet href="https://fonts.googleapis.com/icon?family=Material+Icons"/>


        <Title text="Welcome to Leptos"/>

        <div id="page_container">

            <header>
                <div id="navbar">
                    <div id="navbar_front" class="flex row">
                        // <!-- <button id="menu_button" class="h_button icon_only"><i class=material-icons>menu</i></button> -->
                        <a href="https://www.fibermc.com"><h1 id="title" class="header_element">"FiberMC"</h1></a>
                        <h2 id="subtitle" class="header_element">"Minecraft Fabric Mod List"</h2>
                    </div>
                    <div class="end flex row">
                        <a href="https://ko-fi.com/johnpaul" class="logo button" target="_blank" rel="noopener noreferrer"><img src="https://static.jpcode.dev/img/icon/ko-fi.svg" alt="ko-fi" class="invert icon_after"/></a>
                        <a href="https://patreon.com/jpcode" class="logo button" target="_blank" rel="noopener noreferrer"><img src="https://static.jpcode.dev/img/icon/patreon.svg" alt="Patreon" class="invert icon_after"/></a>
                        <a href="https://github.com/John-Paul-R/fibermc" class="logo button" target="_blank" rel="noopener noreferrer"><img src="https://static.jpcode.dev/img/icon/github.svg" alt="GitHub" class="invert icon_after"/></a>
                        <a href="https://discord.jpcode.dev" class="logo button" target="_blank" rel="noopener noreferrer"><img src="https://static.jpcode.dev/img/icon/discord.svg" alt="Discord" class="invert icon_after"/></a>
                        <a id="about" href="./about" class="button"><span class="text">"About"</span></a>
                        <button class="swap_palette"><span class="text">"Theme"</span><i class="material-icons">"style"</i></button>
                    </div>
                </div>
            </header>

            <Router>
                <main id="content_main">
                    <div id="content_body">
                        <ErrorBoundary
                            // the fallback receives a signal containing current errors
                            fallback=|errors| view! {
                                <div class="error">
                                    <p>"An unexpected error occurred! Errors: "</p>
                                    // we can render a list of errors as strings, if we'd like
                                    <ul>
                                        {move || errors
                                            .get()
                                            .iter()
                                            .map(|(_, e)| view! {<li>{e.to_string()}</li>})
                                            .collect_view()
                                        }
                                    </ul>
                                </div>
                            }
                        >
                            <Routes fallback=|| "Not found">
                                <Route path=path!("") view=|| view! {<HomePage/>}/>
                                <Route path=path!("stats/:mod_id") view=move || {
                                    let params = use_params::<StatsPageParams>();
                                    view! { <StatsPage params=params/> }
                                }/>
                            </Routes>
                        </ErrorBoundary>
                    </div>
                </main>
            </Router>
        </div>
    }
}

/// Renders the home page of your application.
#[component]
fn HomePage() -> impl IntoView {
    // Creates a reactive value to update the button
    let (count, set_count) = signal(0);
    let on_click = move |_| {
        set_count.update(|count| *count += 1);
        log!("Clicked {} times", count.get());
    };

    view! {
        <div>
            <h1>"Welcome to Leptos!"</h1>
        <input type="text" on:keydown=|_| log!("key") on:change=|_| log!("change")/>
        <button on:click=on_click>Click Me: {move || count.get()}</button>
        </div>
        <a href="./stats">"View mod stats"</a>
    }
}
