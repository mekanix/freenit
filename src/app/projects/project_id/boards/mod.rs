pub mod board_id;

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
    app::projects::NamedForm,
    features::{auth, kanban::Board},
};

use super::project_id;

#[route(POST)]
pub async fn create(cx: &Cx, Form(input): Form<NamedForm>) -> Result<SeeOther> {
    auth::require_user(cx).await?;
    let project_id = *project_id(cx)?;
    let board = Board::add(cx, project_id, &input.name, &input.description).await?;
    Ok(see_other(
        href!(board_id::page, project_id: project_id, board_id: board.id).resolve(cx),
    ))
}
