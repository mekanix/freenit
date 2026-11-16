use crate::features::auth;
use serde::Deserialize;
use topcoat::{
    Result,
    context::Cx,
    router::{
        content::Form,
        error::{SeeOther, see_other},
        href, module_param, page, route,
    },
    view::{View, view},
};

module_param!(pub token: String else bad_request);

#[page]
pub async fn page(cx: &Cx) -> Result<impl View> {
    let token = token(cx)?.clone();
    Ok(view! {
        <main class="auth-form card">
            <h1>"Reset password"</h1>
            <form class="form-grid" method="post" action=(href!(reset, token: token))>
                <label class="floating-field">
                    <input
                        type="password"
                        name="password"
                        autocomplete="new-password"
                        placeholder=" "
                        minlength="8"
                        required=""
                    >
                    <span>"New password"</span>
                </label>
                <label class="floating-field">
                    <input
                        type="password"
                        name="repeat_password"
                        autocomplete="new-password"
                        placeholder=" "
                        minlength="8"
                        required=""
                    >
                    <span>"Repeat password"</span>
                </label>
                <div class="actions">
                    <button class="button primary" type="submit">
                        "Reset password"
                    </button>
                </div>
            </form>
        </main>
    })
}

#[derive(Deserialize)]
pub struct NewPassword {
    password: String,
    repeat_password: String,
}

#[route(POST)]
pub async fn reset(cx: &Cx, Form(input): Form<NewPassword>) -> Result<SeeOther> {
    if input.password != input.repeat_password {
        return Err(topcoat::router::error::bad_request("passwords do not match").into());
    }
    auth::reset_password(cx, token(cx)?, input.password).await?;
    auth::clear_session(cx);
    Ok(see_other(href!(crate::app::login::page).resolve(cx)))
}
