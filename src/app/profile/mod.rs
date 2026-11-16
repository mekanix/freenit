mod password;

use crate::features::auth;
use topcoat::{
    Result,
    context::Cx,
    router::{href, page},
    view::{View, view},
};

#[page]
pub async fn page(cx: &Cx) -> Result<impl View> {
    let user = auth::require_user(cx).await?;
    Ok(view! {
        <main class="container profile-page">
            <div>
                "ID: "
                (user.id.to_string())
            </div>
            <div>
                "Email: "
                (user.email)
            </div>
            if let Some(fullname) = user.fullname {
                <div>
                    "Full name: "
                    (fullname)
                </div>
            }
            <div>
                "Active: "
                if user.active {
                    "yes"
                } else {
                    "no"
                }
            </div>
            <div>
                "Admin: "
                if user.admin {
                    "yes"
                } else {
                    "no"
                }
            </div>
            topcoat::runtime::link(
                href: href!(password::page),
                attrs: topcoat::view::attributes! { class="button primary" },
                "Change Password"
            )
        </main>
    })
}
