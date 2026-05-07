use leptos::{
    leptos_dom::logging::console_log,
    logging::debug_log,
    prelude::*,
    server::codee::string::{FromToStringCodec, JsonSerdeCodec},
};
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet};
use leptos_router::{
    components::{Route, Router, Routes},
    StaticSegment,
};
use leptos_use::{
    core::ConnectionReadyState, storage::use_local_storage, use_websocket, UseWebSocketReturn,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub struct Message {
    pub role: String,
    pub content: String,
}

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <AutoReload options=options.clone() />
                <HydrationScripts options/>
                <MetaTags/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/chatbot.css"/>

        // content for this welcome page
        <Router>
            <main>
                <Routes fallback=|| "Page not found.".into_view()>
                    <Route path=StaticSegment("") view=HomePage/>
                </Routes>
            </main>
        </Router>
    }
}

/// Renders the home page of your application.
#[component]
fn HomePage() -> impl IntoView {
    let (messages, set_messages) = signal(Vec::new());
    let (id, set_id, remove_id) = use_local_storage::<String, FromToStringCodec>("id");
    let input = RwSignal::new("".to_string());

    let url = format!("/api/chat?id={}", id.get()).leak();
    let UseWebSocketReturn {
        ready_state,
        message,
        send,
        ..
    } = use_websocket::<String, Vec<Message>, JsonSerdeCodec>(url);

    Effect::new(move || {
        if let Some(msgs) = message.get() {
            set_messages.set(msgs);
        }
    });

    let send_message = move |_| {
        let input = input.get();

        send(&input);
        let mut msg_lock = set_messages.write();
        msg_lock.push(Message {
            role: "user".to_string(),
            content: input,
        });
        msg_lock.push(Message {
            role: "loading".to_string(),
            content: String::new(),
        });
    };

    view! {
        <div id="icon"><svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-message-circle-icon lucide-message-circle"><path d="M2.992 16.342a2 2 0 0 1 .094 1.167l-1.065 3.29a1 1 0 0 0 1.236 1.168l3.413-.998a2 2 0 0 1 1.099.092 10 10 0 1 0-4.777-4.719"/></svg></div>
        <div id="chat">
            <div id="banner">
                <p>BYOAT Chatbot</p>
                <div
                    style=move || {
                        format!("background-color: {};",
                            if ready_state.get() == ConnectionReadyState::Open {
                                "green"
                            } else {
                                "red"
                            }
                        )
                    }
                    id="status"
                ></div>
            </div>
            <div id="messages">
                <For
                    each=move || messages.get().into_iter().enumerate()
                    key=|msg| msg.0
                    children=move |(_id, msg)| {
                       view! {
                        <div class={
                                format!("message {}",
                                    if msg.role == "user" {
                                        "user"
                                    } else {
                                        "bot"
                                    }
                                )
                            }>
                                {
                                    if msg.role == "loading" {
                                        view! {
                                            <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-ellipsis-icon lucide-ellipsis"><circle cx="12" cy="12" r="1"/><circle cx="19" cy="12" r="1"/><circle cx="5" cy="12" r="1"/></svg>
                                        }.into_any()
                                    } else {
                                        view! {
                                            {msg.content.clone()}
                                         }.into_any()
                                    }
                                }
                            </div>
                        }
                    }
                />
            </div>
            <div id="inputs">
                <textarea
                    bind:value=input
                    id="text"
                ></textarea>
                <button
                    on:click=send_message
                    id="send"
                ><svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-send-icon lucide-send"><path d="M14.536 21.686a.5.5 0 0 0 .937-.024l6.5-19a.496.496 0 0 0-.635-.635l-19 6.5a.5.5 0 0 0-.024.937l7.93 3.18a2 2 0 0 1 1.112 1.11z"/><path d="m21.854 2.147-10.94 10.939"/></svg></button>
            </div>
        </div>
    }
}
