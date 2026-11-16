mod token;

use topcoat::{
    Result,
    context::Cx,
    router::{href, page},
    view::{View, view},
};

#[page]
pub async fn page(cx: &Cx) -> Result<impl View> {
    Ok(view! {
        <main class="auth-form card">
            <h1>"Verify your account"</h1>
            <p>
                "Follow the verification link in your registration email, then log in."
            </p>
            topcoat::runtime::link(href: href!(crate::app::login::page), "Go to login")
        </main>
    })
}
