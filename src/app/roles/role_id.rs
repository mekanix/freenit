use crate::{
    app::{
        pagination::{Pagination, pagination},
        users::user_id::user_id,
    },
    features::{
        auth,
        role::{Role, RoleId},
        user::{User, UserId},
    },
};
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

module_param!(pub role_id: RoleId else bad_request);

#[page]
pub async fn page(cx: &Cx) -> Result<impl View> {
    auth::require_admin(cx).await?;
    let role = Role::get(cx, role_id(cx)?).await?;
    let users = User::list(cx).await?;
    let members = Role::users(cx, &role.id).await?;
    let member_ids: Vec<_> = members.iter().map(|user| user.id.clone()).collect();
    let pagination = Pagination::new(cx, users.len())?;
    let users = pagination.slice(users);
    let base = href!(page, role_id: &role.id).resolve(cx);

    Ok(view! {
        <main class="container">
            <h2>
                "Role: "
                (&role.name)
            </h2>
            <h3>"Users"</h3>
            <div class="management-table role-members-table">
                <div class="heading">"ID"</div>
                <div class="heading">"Email"</div>
                <div class="heading">"Active"</div>
                <div class="heading">"Admin"</div>
                <div class="heading">"Member"</div>
                for user in users {
                    <div class="data">(user.id.to_string())</div>
                    topcoat::runtime::link(
                        href: href!(super::super::users::user_id::page, user_id: &user.id),
                        attrs: topcoat::view::attributes! { class="data" },
                        (user.email)
                    )
                    <div class="data">
                        <input type="checkbox" checked=(user.active) disabled="">
                    </div>
                    <div class="data">
                        <input type="checkbox" checked=(user.admin) disabled="">
                    </div>
                    <div class="data">
                        <form
                            method="post"
                            action=(if member_ids.contains(&user.id) {
                                href!(unassign, role_id: &role.id).resolve(cx)
                            } else {
                                href!(assign, role_id: &role.id).resolve(cx)
                            })
                        >
                            <input
                                type="hidden"
                                name="user_id"
                                value=(user.id.to_string())
                            >
                            <button
                                class="management-checkbox"
                                type="submit"
                                aria-checked=(if member_ids.contains(&user.id) {
                                    "true"
                                } else {
                                    "false"
                                })
                                aria-label="Toggle role membership"
                            ></button>
                        </form>
                    </div>
                }
            </div>
        </main>
        pagination(base: &base, page: pagination.page, pages: pagination.pages)
    })
}

#[derive(Deserialize)]
pub struct RoleAssignment {
    user_id: UserId,
}

#[route(POST "./assign")]
pub async fn assign(cx: &Cx, Form(assignment): Form<RoleAssignment>) -> Result<SeeOther> {
    auth::require_admin(cx).await?;
    Role::assign(cx, role_id(cx)?, &assignment.user_id).await?;
    Ok(see_other(href!(page, role_id: role_id(cx)?).resolve(cx)))
}

#[route(POST "./unassign")]
pub async fn unassign(cx: &Cx, Form(assignment): Form<RoleAssignment>) -> Result<SeeOther> {
    auth::require_admin(cx).await?;
    Role::unassign(cx, role_id(cx)?, &assignment.user_id).await?;
    Ok(see_other(href!(page, role_id: role_id(cx)?).resolve(cx)))
}

#[route(POST "./delete")]
pub async fn delete(cx: &Cx) -> Result<SeeOther> {
    auth::require_admin(cx).await?;
    Role::remove(cx, role_id(cx)?).await?;
    Ok(see_other(href!(super::page).resolve(cx)))
}
