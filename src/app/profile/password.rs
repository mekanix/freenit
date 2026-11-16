use crate::features::{auth, user::User};
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

#[page]
pub async fn page(cx: &Cx) -> Result<impl View> {
    auth::require_user(cx).await?;
    Ok(view! {
        <main class="auth-form card">
            <h1>"Change password"</h1>
            <form class="form-grid" method="post" action=(href!(change))>
                <label class="floating-field">
                    <input
                        type="password"
                        name="current_password"
                        autocomplete="current-password"
                        placeholder=" "
                        required=""
                    >
                    <span>"Current password"</span>
                </label>
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
                        "Change password"
                    </button>
                    topcoat::runtime::link(
                        href: href!(super::page),
                        attrs: topcoat::view::attributes! { class="button secondary" },
                        "Cancel"
                    )
                </div>
            </form>
        </main>
    })
}

#[derive(Deserialize)]
pub struct PasswordChange {
    current_password: String,
    password: String,
    repeat_password: String,
}

#[route(POST)]
pub async fn change(cx: &Cx, Form(input): Form<PasswordChange>) -> Result<SeeOther> {
    let user = auth::require_user(cx).await?;
    if input.password != input.repeat_password {
        return Err(topcoat::router::error::bad_request("passwords do not match").into());
    }
    User::authenticate(cx, &user.email, input.current_password)
        .await?
        .ok_or_else(topcoat::router::error::unauthorized)?;
    User::set_password(cx, &user.id, input.password).await?;
    Ok(see_other(href!(super::page).resolve(cx)))
}
