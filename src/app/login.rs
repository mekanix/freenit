use serde::Deserialize;
use topcoat::{
    Result,
    context::Cx,
    htmx::{HxRedirect, hx_request},
    router::{
        content::Form,
        error::see_other,
        href, page, route,
        response::{IntoResponse, Response},
    },
    view::{View, view},
};

use crate::features::auth;

#[page]
pub async fn page(cx: &Cx) -> Result<impl View> {
    let logged_in = auth::current_user(cx).await?.is_some();
    Ok(view! {
        <main class="auth-form card">
            <h1>"Login"</h1>
            if logged_in {
                <p>"You are already signed in."</p>
            }
            <form class="form-grid" method="post" action=(href!(login))>
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
                <label class="floating-field">
                    <input
                        type="password"
                        name="password"
                        autocomplete="current-password"
                        placeholder=" "
                        required=""
                    >
                    <span>"Password"</span>
                </label>
                <div class="actions">
                    <button class="button primary" type="submit">"Login"</button>
                </div>
            </form>
            <p>
                topcoat::runtime::link(
                    href: href!(crate::app::forgot_password::page),
                    "Forgot password?"
                )
            </p>
            <p>
                topcoat::runtime::link(
                    href: href!(crate::app::register::page),
                    "Create an account"
                )
            </p>
        </main>
    })
}

#[derive(Deserialize)]
pub struct Login {
    email: String,
    password: String,
}

#[route(POST)]
pub async fn login(cx: &Cx, Form(input): Form<Login>) -> Result<Response> {
    auth::login(cx, &input.email, input.password).await?;
    if hx_request(cx) {
        (HxRedirect::from("/"), ()).into_response(cx)
    } else {
        see_other("/").into_response(cx)
    }
}
