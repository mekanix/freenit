pub mod role_id;

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
    features::{auth, role::Role, user::UserId},
};

#[page]
pub async fn page(cx: &Cx) -> Result<impl View> {
    let admin = auth::require_admin(cx).await?;
    let roles = Role::list(cx).await?;
    let pagination = Pagination::new(cx, roles.len())?;
    let roles = pagination.slice(roles);
    let base = href!(page).resolve(cx);

    Ok(view! {
        <main class="container">
            <div class="management-header">
                <h2>"Roles"</h2>
                <button
                    class="button primary"
                    type="button"
                    popovertarget="create-role"
                >
                    "Create"
                </button>
            </div>
            if roles.is_empty() {
                <p>"No roles yet."</p>
            } else {
                <div class="management-table roles-table">
                    <div class="heading">"ID"</div>
                    <div class="heading">"Name"</div>
                    for role in roles {
                        <div class="data">
                            topcoat::runtime::link(
                                href: href!(role_id::page, role_id: &role.id),
                                (role.id.to_string())
                            )
                        </div>
                        <div class="data">
                            topcoat::runtime::link(
                                href: href!(role_id::page, role_id: &role.id),
                                (&role.name)
                            )
                        </div>
                    }
                </div>
            }
        </main>
        pagination(base: &base, page: pagination.page, pages: pagination.pages)
        <div
            id="create-role"
            class="management-modal"
            popover="auto"
            role="dialog"
            aria-modal="true"
            aria-labelledby="create-role-title"
        >
            <h2 id="create-role-title">"Create"</h2>
            <form class="form-grid" method="post" action=(href!(create))>
                <input type="hidden" name="user_id" value=(admin.id.to_string())>
                <label>
                    "Name"
                    <input type="text" name="name" required="">
                </label>
                <div class="actions">
                    <button class="button primary" type="submit">"Create"</button>
                    <button
                        class="button error"
                        type="button"
                        popovertarget="create-role"
                        popovertargetaction="hide"
                    >
                        "Close"
                    </button>
                </div>
            </form>
        </div>
    })
}

#[derive(Deserialize)]
pub struct NewRole {
    name: String,
    user_id: UserId,
}

#[route(POST)]
pub async fn create(cx: &Cx, Form(new_role): Form<NewRole>) -> Result<SeeOther> {
    auth::require_admin(cx).await?;
    Role::add(cx, &new_role.name, &new_role.user_id).await?;
    Ok(see_other(href!(page).resolve(cx)))
}
