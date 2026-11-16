use serde::Deserialize;
use topcoat::{
    Result,
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
    features::{auth, jabber, user::{User, UserId}},
};

#[page]
pub async fn page(cx: &Cx) -> Result<impl View> {
    auth::require_admin(cx).await?;
    let config = &topcoat::context::context::<Config>(cx).jabber;
    let users = User::list(cx).await?;
    let mut rows = Vec::with_capacity(users.len());
    for user in users {
        let bundle = jabber::decrypt_bundle(cx, &user)?
            .map(|bundle| serde_json::to_string_pretty(&bundle).unwrap_or_default())
            .unwrap_or_default();
        rows.push((user, bundle));
    }

    Ok(view! {
        <main class="container">
            <h1>"Jabber"</h1>
            <section>
                <h2>"XMPP WebSocket"</h2>
                if config.ws_url.is_empty() {
                    <p>"Not configured."</p>
                } else {
                    <p><code>(&config.ws_url)</code></p>
                }
                <p>
                    topcoat::runtime::link(
                        href: href!(crate::app::chat::page),
                        "Open chat"
                    )
                </p>
            </section>
            <section>
                <h2>"OMEMO bundles"</h2>
                if rows.is_empty() {
                    <p>"No users yet."</p>
                } else {
                    for (user, bundle) in rows {
                        <article class="card">
                            <header><h3>(&user.email)</h3></header>
                            <form class="form-grid" method="post" action=(href!(save))>
                                <input
                                    type="hidden"
                                    name="user_id"
                                    value=(user.id.to_string())
                                >
                                <label>
                                    "OMEMO JSON"
                                    <textarea
                                        name="bundle"
                                        rows="12"
                                        placeholder="Leave empty to remove the bundle"
                                    >
                                        (bundle)
                                    </textarea>
                                </label>
                                <button class="button primary" type="submit">
                                    "Save bundle"
                                </button>
                            </form>
                        </article>
                    }
                }
            </section>
        </main>
    })
}

#[derive(Deserialize)]
pub struct OmemoBundle {
    user_id: UserId,
    bundle: String,
}

#[route(POST "./omemo")]
pub async fn save(cx: &Cx, Form(input): Form<OmemoBundle>) -> Result<SeeOther> {
    auth::require_admin(cx).await?;
    let user = User::get(cx, &input.user_id).await?;
    let encrypted = jabber::encrypt_bundle(cx, &user, &input.bundle)?;
    User::set_omemo_bundle(cx, &input.user_id, encrypted).await?;
    Ok(see_other(href!(page).resolve(cx)))
}
