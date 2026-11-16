pub mod task_id;

use serde::Deserialize;
use topcoat::{
    Result,
    context::Cx,
    router::{
        content::Form,
        error::{SeeOther, see_other},
        href, route,
    },
};

use crate::{
    app::projects::{optional_number, optional_string},
    features::{auth, kanban::Task},
};

use super::{board_id, column_id, project_id};

#[derive(Deserialize)]
pub struct NewTask {
    title: String,
    #[serde(default)]
    description: String,
    #[serde(default, deserialize_with = "optional_string")]
    assignee_id: Option<String>,
    #[serde(default, deserialize_with = "optional_number")]
    parent_id: Option<u64>,
}

#[route(POST)]
pub async fn create(cx: &Cx, Form(input): Form<NewTask>) -> Result<SeeOther> {
    auth::require_user(cx).await?;
    let project_id = *project_id(cx)?;
    let board_id = *board_id(cx)?;
    let column_id = *column_id(cx)?;
    Task::add(
        cx,
        column_id,
        &input.title,
        &input.description,
        input.assignee_id,
        input.parent_id,
    )
    .await?;
    Ok(see_other(
        href!(super::super::super::page, project_id: project_id, board_id: board_id).resolve(cx),
    ))
}
