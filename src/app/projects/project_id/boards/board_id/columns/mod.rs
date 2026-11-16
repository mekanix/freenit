pub mod column_id;

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

use crate::features::{auth, kanban::BoardColumn};

use super::{board_id, project_id};

#[derive(Deserialize)]
pub struct ColumnForm {
    pub name: String,
    #[serde(default)]
    pub position: i64,
}

#[route(POST)]
pub async fn create(cx: &Cx, Form(input): Form<ColumnForm>) -> Result<SeeOther> {
    auth::require_user(cx).await?;
    let project_id = *project_id(cx)?;
    let board_id = *board_id(cx)?;
    BoardColumn::add(cx, board_id, &input.name).await?;
    Ok(see_other(
        href!(super::page, project_id: project_id, board_id: board_id).resolve(cx),
    ))
}
