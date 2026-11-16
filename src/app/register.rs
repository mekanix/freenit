use serde::Deserialize;
use topcoat::{
    Result,
    context::{Cx, try_context},
    router::{
        Body, Method, StatusCode,
        content::Form,
        error::{SeeOther, rewrite, see_other},
        href, page, route,
    },
    view::{View, view},
};

use crate::{chota::components::toast::Notification, features::auth};

struct RegistrationFailed {
    email: String,
}

#[page]
pub async fn page(cx: &Cx) -> Result<impl View> {
    let failed = try_context::<RegistrationFailed>(cx);
    let email = failed.map_or("", |failed| failed.email.as_str());

    Ok(view! {
        if failed.is_some() {
            (StatusCode::BAD_REQUEST)
        }
        <main class="auth-form card">
            <h1>"Register"</h1>
            <form class="form-grid" method="post" action=(href!(register))>
                <label class="floating-field">
                    <input
                        type="email"
                        name="email"
                        autocomplete="email"
                        value=(email)
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
                        autocomplete="new-password"
                        placeholder=" "
                        minlength="8"
                        required=""
                    >
                    <span>"Password"</span>
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
                    <button class="button primary" type="submit">"Register"</button>
                </div>
            </form>
            <p>
                topcoat::runtime::link(
                    href: href!(crate::app::login::page),
                    "Already have an account?"
                )
            </p>
        </main>
    })
}

#[derive(Deserialize)]
pub struct Registration {
    email: String,
    password: String,
    repeat_password: String,
}

#[route(POST)]
pub async fn register(cx: &Cx, Form(input): Form<Registration>) -> Result<SeeOther> {
    if input.password != input.repeat_password {
        return Err(rewrite(href!(page).resolve(cx), Body::empty())
            .method(Method::GET)
            .with(RegistrationFailed { email: input.email })
            .with(Notification::error("passwords do not match"))
            .into());
    }
    if let Err(error) = auth::register(cx, &input.email, input.password).await {
        return Err(rewrite(href!(page).resolve(cx), Body::empty())
            .method(Method::GET)
            .with(RegistrationFailed { email: input.email })
            .with(Notification::error(error.to_string()))
            .into());
    }
    if cfg!(feature = "mail") {
        Ok(see_other(href!(crate::app::verify::page).resolve(cx)))
    } else {
        Ok(see_other(href!(crate::app::login::page).resolve(cx)))
    }
}
