pub mod tasks;

use topcoat::{
    Result,
    context::Cx,
    router::{
        content::Form,
        error::{SeeOther, see_other},
        href, module_param, route,
    },
};

use crate::features::{auth, kanban::BoardColumn};

use super::{ColumnForm, board_id, project_id};

module_param!(pub column_id: u64 else bad_request);

#[route(POST "./edit")]
pub async fn edit(cx: &Cx, Form(input): Form<ColumnForm>) -> Result<SeeOther> {
    auth::require_user(cx).await?;
    let project_id = *project_id(cx)?;
    let board_id = *board_id(cx)?;
    BoardColumn::edit(cx, *column_id(cx)?, &input.name, input.position).await?;
    Ok(see_other(
        href!(super::super::page, project_id: project_id, board_id: board_id).resolve(cx),
    ))
}

#[route(POST "./delete")]
pub async fn delete(cx: &Cx) -> Result<SeeOther> {
    auth::require_user(cx).await?;
    let project_id = *project_id(cx)?;
    let board_id = *board_id(cx)?;
    BoardColumn::remove(cx, *column_id(cx)?).await?;
    Ok(see_other(
        href!(super::super::page, project_id: project_id, board_id: board_id).resolve(cx),
    ))
}
