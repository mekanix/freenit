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

use crate::features::auth;

#[page]
pub async fn page(cx: &Cx) -> Result<impl View> {
    Ok(view! {
        <main class="auth-form card">
            <h1>"Forgot password"</h1>
            if cfg!(feature = "mail") {
                <p>"Enter your email address and we will send a recovery link."</p>
                <form class="form-grid" method="post" action=(href!(request))>
                    <label class="floating-field">
                        <input
                            type="email"
                            name="email"
                            autocomplete="email"
                            placeholder=" "
                            autofocus=""
                            required=""
                        >
                        <span>"Email"</span>
                    </label>
                    <div class="actions">
                        <button class="button primary" type="submit">
                            "Send recovery link"
                        </button>
                    </div>
                </form>
            } else {
                <p>
                    "Password recovery requires the mail feature and SMTP configuration."
                </p>
            }
            <p>
                topcoat::runtime::link(
                    href: href!(crate::app::login::page),
                    "Back to login"
                )
            </p>
        </main>
    })
}

#[derive(Deserialize)]
pub struct Recovery {
    email: String,
}

#[route(POST)]
pub async fn request(cx: &Cx, Form(input): Form<Recovery>) -> Result<SeeOther> {
    auth::request_reset(cx, &input.email).await?;
    Ok(see_other(href!(sent).resolve(cx)))
}

#[page("./sent")]
pub async fn sent(cx: &Cx) -> Result<impl View> {
    Ok(view! {
        <main class="auth-form card">
            <h1>"Check your email"</h1>
            <p>"If the account exists, a password recovery link has been sent."</p>
            topcoat::runtime::link(
                href: href!(crate::app::login::page),
                "Back to login"
            )
        </main>
    })
}
