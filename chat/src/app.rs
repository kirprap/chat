use yew::prelude::*;
use gloo_net::websocket::{futures::WebSocket, Message};
use futures::{StreamExt, SinkExt};
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlInputElement;
use gloo_dialogs::prompt;
use serde_json;

use crate::types::ChatMessage;

#[function_component(App)]
pub fn app() -> Html {
    let messages = use_state(Vec::<ChatMessage>::new);
    let input_text = use_state(String::new);
    let ws_tx = use_mut_ref(|| None);
    let username = use_state(|| String::new());

    {
        let username = username.clone();
        let messages = messages.clone();
        let ws_tx = ws_tx.clone();
        use_effect_with((), move |_| {
            // Prompt for username
            if username.is_empty() {
                let name = prompt("Enter your username:", Some(""))
                    .unwrap_or_else(|| "Anonymous".to_string());
                {
                    username.set(name.clone());
                    
                    // Connect to WebSocket with username
                    let ws_conn = WebSocket::open(&format!("ws://127.0.0.1:8080/ws/?username={}", name))
                        .unwrap();
                    let (write, mut read) = ws_conn.split();
                    *ws_tx.borrow_mut() = Some(write);

                    spawn_local(async move {
                        while let Some(msg) = read.next().await {
                            if let Ok(Message::Text(data)) = msg {
                                if let Ok(chat_msg) = serde_json::from_str::<ChatMessage>(&data) {
                                    let mut new_messages = (*messages).clone();
                                    new_messages.push(chat_msg);
                                    messages.set(new_messages);
                                }
                            }
                        }
                    });
                }
            }
            || {}
        });
    }

    let on_input = {
        let input_text = input_text.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            input_text.set(input.value());
        })
    };

    let on_submit = {
        let input_text = input_text.clone();
        let ws_tx = ws_tx.clone();
        let username = username.clone();
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            let input = (*input_text).clone();
            if !input.is_empty() {
                input_text.set(String::new());
                let ws_tx = ws_tx.clone();
                let chat_msg = ChatMessage {
                    username: (*username).clone(),
                    content: input,
                    message_type: "message".to_string(),
                };
                
                if let Ok(msg) = serde_json::to_string(&chat_msg) {
                    spawn_local(async move {
                        if let Some(ws) = &mut *ws_tx.borrow_mut() {
                            ws.send(Message::Text(msg)).await.unwrap();
                        }
                    });
                }
            }
        })
    };

    html! {
        <div class="chat-app">
            <aside class="chat-sidebar">
                <div class="chat-logo">{"💬"}</div>
                <nav class="chat-channels">
                    <div class="chat-channel selected">{"# general"}</div>
                </nav>
            </aside>
            <main class="chat-main">
                <header class="chat-header">
                    <span class="chat-header-title">{"# general"}</span>
                </header>
                <section class="chat-messages">
                    { for (*messages).iter().map(|msg| {
                        html! {
                            <div class={classes!("chat-message", msg.message_type.clone())}>
                                <div class="chat-avatar">{&msg.username.chars().next().unwrap_or('?')}</div>
                                <div class="chat-message-content">
                                    <span class="chat-username">{&msg.username}</span>
                                    <span class="chat-text">
                                        {
                                            if msg.message_type == "join" || msg.message_type == "leave" {
                                                format!("* {}", &msg.content)
                                            } else {
                                                msg.content.clone()
                                            }
                                        }
                                    </span>
                                </div>
                            </div>
                        }
                    })}
                </section>
                <form class="chat-input-area" onsubmit={on_submit}>
                    <input 
                        class="chat-input"
                        type="text"
                        value={(*input_text).clone()}
                        oninput={on_input}
                        placeholder="Type a message..."
                        disabled={(*username).is_empty()}
                    />
                    <button
                        class="chat-send"
                        type="submit"
                        disabled={(*username).is_empty()}
                    >
                        {"Send"}
                    </button>
                </form>
            </main>
        </div>
    }
}
