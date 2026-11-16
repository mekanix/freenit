use serde::Deserialize;
use topcoat::{
    Result,
    asset::asset,
    context::Cx,
    router::{
        content::Form,
        error::{SeeOther, see_other},
        href, page, route,
    },
    view::{View, view},
};

use crate::{
    config::Config,
    features::{auth, jabber, user::User},
};

#[page]
pub async fn page(cx: &Cx) -> Result<impl View> {
    let user = auth::require_user(cx).await?;
    let token = auth::issue_token(cx, &user, auth::TokenPurpose::Access)?;
    let config = &topcoat::context::context::<Config>(cx).jabber;
    let ws_url = if config.ws_url.is_empty() {
        format!(
            "wss://{}:5443/ws",
            user.email.split_once('@').map_or("", |(_, domain)| domain)
        )
    } else {
        config.ws_url.clone()
    };
    let bundle = jabber::decrypt_bundle(cx, &user)?
        .unwrap_or(serde_json::Value::Null)
        .to_string();
    Ok(view! {
        <main
            id="chat"
            class="chat-shell"
            data-ws-url=(ws_url)
            data-token=(token)
            data-email=(&user.email)
            data-omemo=(bundle)
            data-save-url=(href!(save_omemo).resolve(cx))
        >
            <header class="chat-toolbar">
                <span id="chat-status" class="status disconnected">"Disconnected"</span>
                <input
                    id="chat-ws-url"
                    type="text"
                    value=(if config.ws_url.is_empty() {
                        format!(
                            "wss://{}:5443/ws",
                            user.email.split_once('@').map_or("", |(_, domain)| domain),
                        )
                    } else {
                        config.ws_url.clone()
                    })
                    aria-label="XMPP WebSocket URL"
                >
                <button
                    id="chat-connect"
                    data-action="connect"
                    class="button outline"
                    type="button"
                >
                    "Connect"
                </button>
            </header>
            <p id="chat-error" class="error-bar" hidden=""></p>
            <section id="chat-call" class="call-card" hidden="">
                <strong id="chat-call-peer"></strong>
                <p id="chat-call-status"></p>
                <div class="actions">
                    <button
                        id="chat-call-accept"
                        data-action="call-accept"
                        class="button primary"
                        type="button"
                    >
                        "Accept"
                    </button>
                    <button
                        id="chat-call-reject"
                        data-action="call-reject"
                        class="button error"
                        type="button"
                    >
                        "Reject"
                    </button>
                    <button
                        id="chat-call-end"
                        data-action="call-end"
                        class="button error"
                        type="button"
                    >
                        "End call"
                    </button>
                </div>
            </section>
            <audio id="chat-local-audio" autoplay="" muted="" playsinline="" hidden=""></audio>
            <audio id="chat-remote-audio" autoplay="" playsinline="" hidden=""></audio>
            <div class="chat-layout">
                <aside class="chat-sidebar">
                    <div class="actions">
                        <h2>"Contacts"</h2>
                        <span id="chat-roster-count"></span>
                    </div>
                    <input
                        id="chat-search"
                        type="search"
                        placeholder="Search contacts"
                        aria-label="Search contacts"
                    >
                    <div id="chat-roster" class="chat-roster"></div>
                    <form id="chat-join" class="form-grid">
                        <h2>"Join room"</h2>
                        <input
                            name="jid"
                            type="text"
                            placeholder="room@conference.example.com"
                            required=""
                        >
                        <input
                            name="nick"
                            type="text"
                            placeholder="Nickname"
                            required=""
                        >
                        <button class="button outline" type="submit">"Join"</button>
                    </form>
                    <div id="chat-rooms" class="chat-roster"></div>
                </aside>
                <section class="chat-panel">
                    <header class="actions">
                        <div>
                            <h2 id="chat-title">"Select a contact or room"</h2>
                            <p id="chat-subtitle"></p>
                        </div>
                        <button
                            id="chat-call-start"
                            data-action="call-start"
                            class="button outline"
                            type="button"
                            hidden=""
                        >
                            "Audio call"
                        </button>
                        <button
                            id="chat-omemo"
                            data-action="omemo"
                            class="button outline"
                            type="button"
                            hidden=""
                        >
                            "OMEMO"
                        </button>
                    </header>
                    <div id="chat-messages" class="chat-messages">
                        <p class="empty-state">
                            "Select a contact or room to start chatting"
                        </p>
                    </div>
                    <form id="chat-send" class="chat-composer">
                        <textarea
                            id="chat-message"
                            rows="2"
                            placeholder="Type a message"
                            disabled=""
                        ></textarea>
                        <button class="button primary" type="submit" disabled="">
                            "Send"
                        </button>
                    </form>
                </section>
            </div>
            <form
                id="chat-omemo-form"
                method="post"
                action=(href!(save_omemo))
                hidden=""
            >
                <input type="hidden" name="bundle" value="">
            </form>
            <script
                type="module"
                src=(asset!("assets/chat.js"))
                data-module=(asset!("assets/dziber.js"))
                data-wasm=(asset!("assets/dziber_bg.wasm"))
            ></script>
        </main>
    })
}

#[derive(Deserialize)]
struct OmemoForm {
    bundle: String,
}

#[route(POST "./omemo")]
async fn save_omemo(cx: &Cx, Form(input): Form<OmemoForm>) -> Result<SeeOther> {
    let user = auth::require_user(cx).await?;
    let encrypted = jabber::encrypt_bundle(cx, &user, &input.bundle)?;
    User::set_omemo_bundle(cx, &user.id, encrypted).await?;
    Ok(see_other(href!(page).resolve(cx)))
}
