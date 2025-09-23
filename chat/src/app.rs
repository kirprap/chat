use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

#[derive(Serialize, Deserialize)]
struct GreetArgs<'a> {
    text: &'a str,
}

#[function_component(App)]
pub fn app() -> Html {
    html! {
        <div class="chat-app">
            <aside class="chat-sidebar">
                <div class="chat-logo">{"💬"}</div>
                <nav class="chat-channels">
                    <div class="chat-channel selected">{"# general"}</div>
                    <div class="chat-channel">{"# random"}</div>
                </nav>
            </aside>
            <main class="chat-main">
                <header class="chat-header">
                    <span class="chat-header-title">{"# general"}</span>
                </header>
                <section class="chat-messages">
                    <div class="chat-message">
                        <span class="chat-avatar">{"🧑"}</span>
                        <div class="chat-message-content">
                            <span class="chat-username">{"User1"}</span>
                            <span class="chat-text">{"Hello! How can I help you?"}</span>
                        </div>
                    </div>
                    <div class="chat-message">
                        <span class="chat-avatar">{"🤖"}</span>
                        <div class="chat-message-content">
                            <span class="chat-username">{"Bot"}</span>
                            <span class="chat-text">{"Hi! I have a question about your product."}</span>
                        </div>
                    </div>
                </section>
                <form class="chat-input-area">
                    <input class="chat-input" type="text" placeholder="Message #general" />
                    <button class="chat-send" type="submit">{"Send"}</button>
                </form>
            </main>
        </div>
    }
}
