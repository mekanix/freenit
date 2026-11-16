pub mod project_id;

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
    kanban::{PERMISSIONS, Project},
};

#[page]
pub async fn page(cx: &Cx) -> Result<impl View> {
    auth::require_user(cx).await?;
    let projects = Project::list(cx).await?;
    Ok(view! {
        <main class="container">
            <div class="management-header">
                <h2>"Projects"</h2>
                <details class="management-create">
                    <summary class="button primary">"Create"</summary>
                    <form
                        class="form-grid management-create-form"
                        method="post"
                        action=(href!(create))
                    >
                        <label>
                            "Name"
                            <input type="text" name="name" autofocus="" required="">
                        </label>
                        <label>
                            "Description"
                            <textarea name="description" rows="3"></textarea>
                        </label>
                        <div class="actions">
                            <button class="button primary" type="submit">
                                "Create"
                            </button>
                        </div>
                    </form>
                </details>
            </div>
            if projects.is_empty() {
                <p>"No projects yet."</p>
            } else {
                <div class="management-table projects-table">
                    <div class="heading">"ID"</div>
                    <div class="heading">"Name"</div>
                    <div class="heading">"Description"</div>
                    <div class="heading">"Actions"</div>
                    for project in projects {
                        <div class="data">(project.id)</div>
                        <div class="data">
                            topcoat::runtime::link(
                                href: href!(project_id::page, project_id: project.id),
                                (&project.name)
                            )
                        </div>
                        <div class="data">
                            (project.description.as_deref().unwrap_or_default())
                        </div>
                        <div class="data">
                            <form
                                method="post"
                                action=(href!(project_id::delete, project_id: project.id))
                            >
                                <button class="button error" type="submit">
                                    "Delete"
                                </button>
                            </form>
                        </div>
                    }
                </div>
            }
        </main>
    })
}

#[derive(Deserialize)]
pub struct NamedForm {
    pub name: String,
    #[serde(default)]
    pub description: String,
}

#[route(POST)]
pub async fn create(cx: &Cx, Form(input): Form<NamedForm>) -> Result<SeeOther> {
    let user = auth::require_user(cx).await?;
    let project = Project::add(cx, &user, &input.name, &input.description).await?;
    Ok(see_other(
        href!(project_id::page, project_id: project.id).resolve(cx),
    ))
}

fn checkbox<'de, D>(deserializer: D) -> std::result::Result<bool, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;
    Ok(matches!(value.as_str(), "true" | "on" | "1"))
}

#[derive(Default, Deserialize)]
pub struct PermissionForm {
    #[serde(default, deserialize_with = "checkbox")]
    create_project: bool,
    #[serde(default, deserialize_with = "checkbox")]
    delete_project: bool,
    #[serde(default, deserialize_with = "checkbox")]
    rename_project: bool,
    #[serde(default, deserialize_with = "checkbox")]
    create_board: bool,
    #[serde(default, deserialize_with = "checkbox")]
    delete_board: bool,
    #[serde(default, deserialize_with = "checkbox")]
    rename_board: bool,
    #[serde(default, deserialize_with = "checkbox")]
    create_column: bool,
    #[serde(default, deserialize_with = "checkbox")]
    delete_column: bool,
    #[serde(default, deserialize_with = "checkbox")]
    rename_column: bool,
}

impl PermissionForm {
    pub fn into_vec(self) -> Vec<String> {
        let mut permissions = Vec::new();
        for (permission, enabled) in [
            ("create_project", self.create_project),
            ("delete_project", self.delete_project),
            ("rename_project", self.rename_project),
            ("create_board", self.create_board),
            ("delete_board", self.delete_board),
            ("rename_board", self.rename_board),
            ("create_column", self.create_column),
            ("delete_column", self.delete_column),
            ("rename_column", self.rename_column),
        ] {
            if enabled {
                permissions.push(permission.to_owned());
            }
        }
        permissions
    }
}

#[topcoat::view::component]
pub async fn permission_checkboxes(cx: &Cx, selected: Vec<String>) -> Result<impl View> {
    Ok(view! {
        <fieldset class="permission-list">
            <legend>"Permissions"</legend>
            for permission in PERMISSIONS {
                <label class="checkbox">
                    <input
                        type="checkbox"
                        name=(permission)
                        value="on"
                        checked=(selected.iter().any(|selected| selected == permission))
                    >
                    " "
                    (permission)
                </label>
            }
        </fieldset>
    })
}

pub fn optional_number<'de, D, T>(deserializer: D) -> std::result::Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: std::str::FromStr,
    T::Err: std::fmt::Display,
{
    let value = String::deserialize(deserializer)?;
    if value.trim().is_empty() {
        Ok(None)
    } else {
        value.parse().map(Some).map_err(serde::de::Error::custom)
    }
}

pub fn optional_string<'de, D>(deserializer: D) -> std::result::Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;
    Ok((!value.trim().is_empty()).then_some(value))
}
