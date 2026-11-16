pub mod group_id;

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
    app::projects::{PermissionForm, permission_checkboxes},
    features::{
        auth,
        kanban::{Project, ProjectGroup},
    },
};

use super::project_id;

#[derive(Deserialize)]
pub struct GroupForm {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(flatten)]
    pub permissions: PermissionForm,
}

#[page]
pub async fn page(cx: &Cx) -> Result<impl View> {
    auth::require_user(cx).await?;
    let project_id = *project_id(cx)?;
    let project = Project::get(cx, project_id).await?;
    let groups = ProjectGroup::for_project(cx, project_id).await?;
    let mut rows = Vec::with_capacity(groups.len());
    for group in groups {
        let permissions = ProjectGroup::permissions(cx, group.id).await?;
        rows.push((group, permissions));
    }
    Ok(view! {
        <main class="container">
            <div class="management-header">
                <div>
                    <h2>(&project.name)</h2>
                    <p>(project.description.as_deref().unwrap_or_default())</p>
                </div>
                <details class="management-create">
                    <summary class="button primary">"Create Group"</summary>
                    <form
                        class="form-grid management-create-form"
                        method="post"
                        action=(href!(create, project_id: project_id))
                    >
                        <label>
                            "Name"
                            <input type="text" name="name" required="">
                        </label>
                        <label>
                            "Description"
                            <textarea name="description" rows="3"></textarea>
                        </label>
                        permission_checkboxes(selected: Vec::new())
                        <div class="actions">
                            <button class="button primary" type="submit">
                                "Create"
                            </button>
                        </div>
                    </form>
                </details>
            </div>
            if rows.is_empty() {
                <p>"No project groups yet."</p>
            } else {
                <div class="management-table projects-table">
                    <div class="heading">"ID"</div>
                    <div class="heading">"Name"</div>
                    <div class="heading">"Description"</div>
                    <div class="heading">"Actions"</div>
                    for (group, permissions) in rows {
                        <div class="data">(group.id)</div>
                        <div class="data">
                            topcoat::runtime::link(
                                href: href!(
                                    group_id::page,
                                    project_id: project_id,
                                    group_id::group_id: group.id,
                                ),
                                (&group.name)
                            )
                            if !permissions.is_empty() {
                                <div class="kanban-permissions">
                                    (permissions.join(", "))
                                </div>
                            }
                        </div>
                        <div class="data">
                            (group.description.as_deref().unwrap_or_default())
                        </div>
                        <div class="data actions">
                            topcoat::runtime::link(
                                href: href!(
                                    group_id::members::page,
                                    project_id: project_id,
                                    group_id::group_id: group.id,
                                ),
                                attrs: topcoat::view::attributes! { class="button outline" },
                                "Members"
                            )
                            <form
                                method="post"
                                action=(href!(
                                    group_id::delete,
                                    project_id: project_id,
                                    group_id::group_id: group.id,
                                ))
                            >
                                <button class="button error" type="submit">
                                    "Delete"
                                </button>
                            </form>
                        </div>
                    }
                </div>
            }
            <p>
                topcoat::runtime::link(
                    href: href!(super::page, project_id: project_id),
                    "Back to project"
                )
            </p>
        </main>
    })
}

#[route(POST)]
pub async fn create(cx: &Cx, Form(input): Form<GroupForm>) -> Result<SeeOther> {
    auth::require_user(cx).await?;
    let project_id = *project_id(cx)?;
    let group = ProjectGroup::add(
        cx,
        project_id,
        &input.name,
        &input.description,
        input.permissions.into_vec(),
    )
    .await?;
    Ok(see_other(
        href!(group_id::page, project_id: project_id, group_id::group_id: group.id).resolve(cx),
    ))
}
