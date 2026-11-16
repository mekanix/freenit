use crate::{
    app::{
        pagination::{Pagination, pagination},
        users::user_id::user_id,
    },
    features::{
        auth,
        group::{Group, GroupId},
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

module_param!(pub group_id: GroupId else bad_request);

#[page]
pub async fn page(cx: &Cx) -> Result<impl View> {
    auth::require_admin(cx).await?;
    let group = Group::get(cx, group_id(cx)?).await?;
    let users = User::list(cx).await?;
    let members = Group::users(cx, &group.id).await?;
    let member_ids: Vec<_> = members.iter().map(|user| user.id.clone()).collect();
    let pagination = Pagination::new(cx, users.len())?;
    let users = pagination.slice(users);
    let base = href!(page, group_id: &group.id).resolve(cx);

    Ok(view! {
        <main class="container">
            <h2>
                "Group: "
                (&group.name)
            </h2>
            <h3>"Users"</h3>
            <div class="management-table membership-table">
                <div class="heading">"Name"</div>
                <div class="heading">"Member"</div>
                for user in users {
                    topcoat::runtime::link(
                        href: href!(super::super::users::user_id::page, user_id: &user.id),
                        attrs: topcoat::view::attributes! { class="data" },
                        (user.email)
                    )
                    <div class="data">
                        <form
                            method="post"
                            action=(if member_ids.contains(&user.id) {
                                href!(unassign, group_id: &group.id).resolve(cx)
                            } else {
                                href!(assign, group_id: &group.id).resolve(cx)
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
                                aria-label="Toggle group membership"
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
pub struct Membership {
    user_id: UserId,
}

#[route(POST "./assign")]
pub async fn assign(cx: &Cx, Form(input): Form<Membership>) -> Result<SeeOther> {
    auth::require_admin(cx).await?;
    Group::assign(cx, group_id(cx)?, &input.user_id).await?;
    Ok(see_other(href!(page, group_id: group_id(cx)?).resolve(cx)))
}

#[route(POST "./unassign")]
pub async fn unassign(cx: &Cx, Form(input): Form<Membership>) -> Result<SeeOther> {
    auth::require_admin(cx).await?;
    Group::unassign(cx, group_id(cx)?, &input.user_id).await?;
    Ok(see_other(href!(page, group_id: group_id(cx)?).resolve(cx)))
}

#[route(POST "./delete")]
pub async fn delete(cx: &Cx) -> Result<SeeOther> {
    auth::require_admin(cx).await?;
    Group::remove(cx, group_id(cx)?).await?;
    Ok(see_other(href!(super::page).resolve(cx)))
}
