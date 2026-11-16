use crate::features::auth;
use topcoat::{
    Result,
    context::Cx,
    router::{href, module_param, page},
    view::{View, view},
};

module_param!(pub token: String else bad_request);

#[page]
pub async fn page(cx: &Cx) -> Result<impl View> {
    auth::verify(cx, token(cx)?).await?;
    Ok(view! {
        <main class="auth-form card">
            <h1>"Account verified"</h1>
            <p>"Your account is active. You can now log in."</p>
            topcoat::runtime::link(
                href: href!(crate::app::login::page),
                attrs: topcoat::view::attributes! { class="button primary" },
                "Login"
            )
        </main>
    })
}
