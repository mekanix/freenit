pub mod user_id;

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
    app::pagination::{Pagination, pagination},
    features::{auth, user::User},
};

#[page]
pub async fn page(cx: &Cx) -> Result<impl View> {
    auth::require_admin(cx).await?;
    let users = User::list(cx).await?;
    let pagination = Pagination::new(cx, users.len())?;
    let users = pagination.slice(users);
    let base = href!(page).resolve(cx);

    Ok(view! {
        <main class="container">
            <h2>"Users"</h2>
            if users.is_empty() {
                <p>"No users yet."</p>
            } else {
                <div class="management-table users-table">
                    <div class="heading">"ID"</div>
                    <div class="heading">"Email"</div>
                    <div class="heading">"Active"</div>
                    <div class="heading">"Admin"</div>
                    for user in users {
                        <div class="data">(user.id.to_string())</div>
                        <div class="data">
                            topcoat::runtime::link(
                                href: href!(user_id::page, user_id: &user.id),
                                (&user.email)
                            )
                        </div>
                        <div class="data">
                            <form
                                method="post"
                                action=(href!(user_id::toggle_active, user_id: &user.id))
                            >
                                <button
                                    class="management-checkbox"
                                    type="submit"
                                    aria-checked=(if user.active { "true" } else { "false" })
                                    aria-label="Toggle active"
                                ></button>
                            </form>
                        </div>
                        <div class="data">
                            <form
                                method="post"
                                action=(href!(user_id::toggle_admin, user_id: &user.id))
                            >
                                <button
                                    class="management-checkbox"
                                    type="submit"
                                    aria-checked=(if user.admin { "true" } else { "false" })
                                    aria-label="Toggle administrator"
                                ></button>
                            </form>
                        </div>
                    }
                </div>
            }
        </main>
        pagination(base: &base, page: pagination.page, pages: pagination.pages)
    })
}

#[derive(Deserialize)]
pub struct NewUser {
    email: String,
    password: String,
    #[serde(default)]
    fullname: String,
    #[serde(default)]
    active: bool,
    #[serde(default)]
    admin: bool,
}

#[route(POST)]
pub async fn create(cx: &Cx, Form(new_user): Form<NewUser>) -> Result<SeeOther> {
    auth::require_admin(cx).await?;
    let _user = User::add(
        cx,
        &new_user.email,
        new_user.password,
        &new_user.fullname,
        new_user.active,
        new_user.admin,
    )
    .await?;
    #[cfg(feature = "mail")]
    crate::features::mail::send_verification(cx, &_user).await?;
    Ok(see_other(href!(page).resolve(cx)))
}
