use gloo_timers::callback::Timeout;
use leptos::{
    html,
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
use uuid::Uuid;
use wasm_bindgen::JsValue;
use web_sys::{ScrollBehavior, ScrollToOptions};

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
    let (loading, set_loading) = signal(false);
    let (messages, set_messages) = signal(Vec::new());
    let (id, set_id, remove_id) = use_local_storage::<String, FromToStringCodec>("id");
    let (unfolded, set_unfolded) = signal(false);
    let (full, set_full) = signal(false);
    let input = RwSignal::new("".to_string());

    let messages_container: NodeRef<html::Div> = NodeRef::new();

    if id.get_untracked().is_empty() {
        set_id.set(Uuid::new_v4().to_string());
    }
    let url = format!("/api/chat?id={}", id.get_untracked()).leak();
    let UseWebSocketReturn {
        ready_state,
        message,
        send,
        ..
    } = use_websocket::<String, Vec<Message>, JsonSerdeCodec>(url);

    Effect::new(move || {
        if let Some(msgs) = message.get() {
            set_messages.set(msgs);
            set_loading.set(false);
            let container = messages_container
                .get()
                .expect("no messages container found");
            request_animation_frame(move || {
                let options = ScrollToOptions::new();
                options.set_top(container.scroll_height() as f64);
                options.set_behavior(ScrollBehavior::Smooth);
                container.scroll_to_with_scroll_to_options(&options)
            });
        }
    });

    let send_message = move || {
        if loading.get_untracked() {
            return;
        }
        let input_val = input.get();
        if input_val.is_empty() {
            return;
        }

        send(&input_val);

        let mut msg_lock = set_messages.write();
        msg_lock.push(Message {
            role: "user".to_string(),
            content: input_val,
        });
        set_loading.set(true);
        let container = messages_container
            .get()
            .expect("no messages container found");
        request_animation_frame(move || {
            let options = ScrollToOptions::new();
            options.set_top(container.scroll_height() as f64);
            options.set_behavior(ScrollBehavior::Smooth);
            container.scroll_to_with_scroll_to_options(&options)
        });

        input.set("".to_string());
    };
    let send_message = StoredValue::new(send_message);

    view! {
        <div
            id="bg"
            class:full=move || full.get()
        >
        </div>
        <div
            id="icon"
            class:hidden=move || unfolded.get()
            on:click=move |_| {
                set_full.set(true);
                window().parent().unwrap().unwrap().post_message(&JsValue::from_str("open"), "*").unwrap();
                set_unfolded.set(true);
                Timeout::new(250, move || {
                    set_full.set(false);
                }).forget();
            }
        >
            <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-message-circle-icon lucide-message-circle"><path d="M2.992 16.342a2 2 0 0 1 .094 1.167l-1.065 3.29a1 1 0 0 0 1.236 1.168l3.413-.998a2 2 0 0 1 1.099.092 10 10 0 1 0-4.777-4.719"/></svg>
        </div>

        <div
            id="chat"
            class:hidden=move || !unfolded.get()
        >
            <div id="banner">
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
                <p>BYOAT Chatbot</p>
                <button
                    id="clear"
                    on:click=move |_| {
                        remove_id();
                        set_full.set(true);
                        Timeout::new(250, move || {
                            window().parent().unwrap().unwrap().post_message(&JsValue::from_str("close"), "*").unwrap();
                            set_unfolded.set(false);
                            set_full.set(false);
                            window().location().reload().unwrap();
                        }).forget();
                    }
                >
                    <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-trash2-icon lucide-trash-2"><path d="M10 11v6"/><path d="M14 11v6"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6"/><path d="M3 6h18"/><path d="M8 6V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/></svg>
                </button>
                <button
                    id="close"
                    on:click=move |_| {
                        set_full.set(true);
                        Timeout::new(250, move || {
                            window().parent().unwrap().unwrap().post_message(&JsValue::from_str("close"), "*").unwrap();
                            set_unfolded.set(false);
                            set_full.set(false);
                        }).forget();
                    }
                >
                    <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-x-icon lucide-x"><path d="M18 6 6 18"/><path d="m6 6 12 12"/></svg>
                </button>
            </div>
            <div
                node_ref=messages_container
                id="messages"
            >
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
                                {msg.content.clone()}
                            </div>
                        }
                    }
                />
                <Show
                    when=move || loading.get()
                >
                    <div class="message bot">
                        <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-ellipsis-icon lucide-ellipsis"><circle cx="12" cy="12" r="1"/><circle cx="19" cy="12" r="1"/><circle cx="5" cy="12" r="1"/></svg>
                    </div>
                </Show>
            </div>
            <div id="inputs">
                <textarea
                    bind:value=input
                    on:keypress=move |kb_event| {
                            if kb_event.key() == "Enter".to_string() {
                               send_message.get_value()();
                               kb_event.prevent_default();
                            }
                        }
                    placeholder="Message..."
                    id="text"
                ></textarea>
                <Show
                    when=move || !input.get().is_empty() && !loading.get()
                >
                    <button
                        on:click=move |_| send_message.get_value()()
                        id="send"
                    >
                        <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-send-icon lucide-send"><path d="M14.536 21.686a.5.5 0 0 0 .937-.024l6.5-19a.496.496 0 0 0-.635-.635l-19 6.5a.5.5 0 0 0-.024.937l7.93 3.18a2 2 0 0 1 1.112 1.11z"/><path d="m21.854 2.147-10.94 10.939"/></svg>
                    </button>
                </Show>
            </div>
        </div>
    }
}
