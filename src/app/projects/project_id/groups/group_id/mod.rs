pub mod members;

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

use crate::{
    app::projects::permission_checkboxes,
    features::{auth, kanban::ProjectGroup},
};

use super::{GroupForm, project_id};

module_param!(pub group_id: u64 else bad_request);

#[page]
pub async fn page(cx: &Cx) -> Result<impl View> {
    auth::require_user(cx).await?;
    let project_id = *project_id(cx)?;
    let group_id = *group_id(cx)?;
    let group = ProjectGroup::get(cx, group_id).await?;
    let permissions = ProjectGroup::permissions(cx, group_id).await?;
    Ok(view! {
        <main class="container">
            <div class="management-header">
                <div>
                    <h2>(&group.name)</h2>
                    <p>(group.description.as_deref().unwrap_or_default())</p>
                </div>
                topcoat::runtime::link(
                    href: href!(members::page, project_id: project_id, group_id: group_id),
                    attrs: topcoat::view::attributes! { class="button primary" },
                    "Members"
                )
            </div>
            <form
                class="form-grid"
                method="post"
                action=(href!(edit, project_id: project_id, group_id: group_id))
            >
                <label>
                    "Name"
                    <input type="text" name="name" value=(&group.name) required="">
                </label>
                <label>
                    "Description"
                    <textarea name="description" rows="3">
                        (group.description.as_deref().unwrap_or_default())
                    </textarea>
                </label>
                permission_checkboxes(selected: permissions)
                <div class="actions">
                    <button class="button primary" type="submit">"Save Group"</button>
                </div>
            </form>
            <form
                method="post"
                action=(href!(delete, project_id: project_id, group_id: group_id))
            >
                <button class="button error" type="submit">"Delete Group"</button>
            </form>
            <p>
                topcoat::runtime::link(
                    href: href!(super::page, project_id: project_id),
                    "Back to groups"
                )
            </p>
        </main>
    })
}

#[route(POST "./edit")]
pub async fn edit(cx: &Cx, Form(input): Form<GroupForm>) -> Result<SeeOther> {
    auth::require_user(cx).await?;
    let project_id = *project_id(cx)?;
    let group_id = *group_id(cx)?;
    ProjectGroup::edit(
        cx,
        group_id,
        &input.name,
        &input.description,
        input.permissions.into_vec(),
    )
    .await?;
    Ok(see_other(
        href!(page, project_id: project_id, group_id: group_id).resolve(cx),
    ))
}

#[route(POST "./delete")]
pub async fn delete(cx: &Cx) -> Result<SeeOther> {
    auth::require_user(cx).await?;
    let project_id = *project_id(cx)?;
    ProjectGroup::remove(cx, *group_id(cx)?).await?;
    Ok(see_other(
        href!(super::page, project_id: project_id).resolve(cx),
    ))
}
