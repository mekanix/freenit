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

use crate::features::{
    auth,
    kanban::{ProjectGroup, ProjectMember},
    user::User,
};

use super::{group_id, project_id};

#[page]
pub async fn page(cx: &Cx) -> Result<impl View> {
    auth::require_user(cx).await?;
    let project_id = *project_id(cx)?;
    let group_id = *group_id(cx)?;
    let group = ProjectGroup::get(cx, group_id).await?;
    let users = User::list(cx).await?;
    let member_ids = ProjectMember::member_ids(cx, group_id).await?;
    Ok(view! {
        <main class="container">
            <div class="management-header">
                <div>
                    <h2>(&group.name)</h2>
                    <p>(group.description.as_deref().unwrap_or_default())</p>
                </div>
                topcoat::runtime::link(
                    href: href!(super::page, project_id: project_id, group_id: group_id),
                    attrs: topcoat::view::attributes! { class="button outline" },
                    "Back to Group"
                )
            </div>
            <h3>"Members"</h3>
            <div class="management-table group-members-table">
                <div class="heading">"Name"</div>
                <div class="heading">"Email"</div>
                <div class="heading">"Member"</div>
                for user in users {
                    let user_id = user.id.to_string();
                    let member = member_ids.contains(&user_id);
                    <div class="data">
                        (user.fullname.as_deref().unwrap_or(&user.email))
                    </div>
                    <div class="data">(&user.email)</div>
                    <div class="data">
                        <form
                            method="post"
                            action=(if member {
                                href!(unassign, project_id: project_id, group_id: group_id).resolve(
                                    cx,
                                )
                            } else {
                                href!(assign, project_id: project_id, group_id: group_id).resolve(
                                    cx,
                                )
                            })
                        >
                            <input
                                type="hidden"
                                name="user_id"
                                value=(user.id.to_string())
                            >
                            <button class="button outline" type="submit">
                                if member {
                                    "Remove"
                                } else {
                                    "Add"
                                }
                            </button>
                        </form>
                    </div>
                }
            </div>
        </main>
    })
}

#[derive(Deserialize)]
struct Membership {
    user_id: String,
}

#[route(POST "./assign")]
async fn assign(cx: &Cx, Form(input): Form<Membership>) -> Result<SeeOther> {
    auth::require_user(cx).await?;
    let project_id = *project_id(cx)?;
    let group_id = *group_id(cx)?;
    ProjectMember::assign(cx, group_id, input.user_id).await?;
    Ok(see_other(
        href!(page, project_id: project_id, group_id: group_id).resolve(cx),
    ))
}

#[route(POST "./unassign")]
async fn unassign(cx: &Cx, Form(input): Form<Membership>) -> Result<SeeOther> {
    auth::require_user(cx).await?;
    let project_id = *project_id(cx)?;
    let group_id = *group_id(cx)?;
    ProjectMember::unassign(cx, group_id, input.user_id).await?;
    Ok(see_other(
        href!(page, project_id: project_id, group_id: group_id).resolve(cx),
    ))
}
