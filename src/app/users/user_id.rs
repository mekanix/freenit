use crate::{
    app::{
        groups::group_id::group_id,
        pagination::{Pagination, pagination},
        roles::role_id::role_id,
    },
    features::{auth, group::Group, role::Role, user::{User, UserId}},
};
use topcoat::{
    Result,
    context::Cx,
    router::{
        error::{SeeOther, see_other},
        href, module_param, page, route,
    },
    view::{View, view},
};

module_param!(pub user_id: UserId else bad_request);

#[page]
pub async fn page(cx: &Cx) -> Result<impl View> {
    auth::require_admin(cx).await?;
    let id = user_id(cx)?;
    let user = User::get(cx, id).await?;
    let roles = Role::list(cx).await?;
    let assigned_roles = Role::for_user(cx, id).await?;
    let role_pages = Pagination::new(cx, roles.len())?;
    let roles = role_pages.slice(roles);
    let groups = Group::list(cx).await?;
    let mut group_rows = Vec::with_capacity(groups.len());
    for group in groups {
        let member = Group::users(cx, &group.id)
            .await?
            .iter()
            .any(|member| &member.id == id);
        group_rows.push((group, member));
    }
    let group_pages = Pagination::new(cx, group_rows.len())?;
    let group_rows = group_pages.slice(group_rows);
    let base = href!(page, user_id: id).resolve(cx);

    Ok(view! {
        <main class="container">
            <h2>
                "User: "
                (&user.email)
            </h2>
            <h3>"Roles"</h3>
            <div class="management-table membership-table">
                <div class="heading">"Name"</div>
                <div class="heading">"Member"</div>
                for role in roles {
                    let assigned = assigned_roles
                        .iter()
                        .any(|assigned| assigned.id == role.id);
                    topcoat::runtime::link(
                        href: href!(super::super::roles::role_id::page, role_id: &role.id),
                        attrs: topcoat::view::attributes! { class="data" },
                        (&role.name)
                    )
                    <div class="data">
                        <form
                            method="post"
                            action=(if assigned {
                                href!(super::super::roles::role_id::unassign, role_id: &role.id).resolve(
                                    cx,
                                )
                            } else {
                                href!(super::super::roles::role_id::assign, role_id: &role.id).resolve(
                                    cx,
                                )
                            })
                        >
                            <input type="hidden" name="user_id" value=(id.to_string())>
                            <button
                                class="management-checkbox"
                                type="submit"
                                aria-checked=(if assigned { "true" } else { "false" })
                                aria-label="Toggle role membership"
                            ></button>
                        </form>
                    </div>
                }
            </div>
        </main>
        pagination(base: &base, page: role_pages.page, pages: role_pages.pages)
        <main class="container">
            <h3>"Groups"</h3>
            <div class="management-table membership-table">
                <div class="heading">"Name"</div>
                <div class="heading">"Member"</div>
                for (group, member) in group_rows {
                    topcoat::runtime::link(
                        href: href!(super::super::groups::group_id::page, group_id: &group.id),
                        attrs: topcoat::view::attributes! { class="data" },
                        (&group.name)
                    )
                    <div class="data">
                        <form
                            method="post"
                            action=(if member {
                                href!(
                                    super::super::groups::group_id::unassign,
                                    group_id: &group.id,
                                ).resolve(
                                    cx,
                                )
                            } else {
                                href!(
                                    super::super::groups::group_id::assign,
                                    group_id: &group.id,
                                ).resolve(
                                    cx,
                                )
                            })
                        >
                            <input type="hidden" name="user_id" value=(id.to_string())>
                            <button
                                class="management-checkbox"
                                type="submit"
                                aria-checked=(if member { "true" } else { "false" })
                                aria-label="Toggle group membership"
                            ></button>
                        </form>
                    </div>
                }
            </div>
        </main>
        pagination(base: &base, page: group_pages.page, pages: group_pages.pages)
    })
}

#[route(POST "./toggle-active")]
pub async fn toggle_active(cx: &Cx) -> Result<SeeOther> {
    auth::require_admin(cx).await?;
    User::toggle_active(cx, user_id(cx)?).await?;
    Ok(see_other(href!(super::page).resolve(cx)))
}

#[route(POST "./toggle-admin")]
pub async fn toggle_admin(cx: &Cx) -> Result<SeeOther> {
    auth::require_admin(cx).await?;
    User::toggle_admin(cx, user_id(cx)?).await?;
    Ok(see_other(href!(super::page).resolve(cx)))
}

#[route(POST "./delete")]
pub async fn delete(cx: &Cx) -> Result<SeeOther> {
    auth::require_admin(cx).await?;
    User::remove(cx, user_id(cx)?).await?;
    Ok(see_other(href!(super::page).resolve(cx)))
}
