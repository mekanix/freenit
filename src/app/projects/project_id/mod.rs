pub mod boards;
pub mod groups;

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
    app::projects::NamedForm,
    features::{
        auth,
        kanban::{Board, Project},
    },
};

module_param!(pub project_id: u64 else bad_request);

#[page]
pub async fn page(cx: &Cx) -> Result<impl View> {
    auth::require_user(cx).await?;
    let id = *project_id(cx)?;
    let project = Project::get(cx, id).await?;
    let boards = Board::for_project(cx, id).await?;
    Ok(view! {
        <main class="container">
            <div class="management-header">
                <div>
                    <h2>(&project.name)</h2>
                    <p>(project.description.as_deref().unwrap_or_default())</p>
                </div>
                <div class="actions">
                    topcoat::runtime::link(
                        href: href!(groups::page, project_id: id),
                        attrs: topcoat::view::attributes! { class="button outline" },
                        "Groups"
                    )
                    <details class="management-create">
                        <summary class="button primary">"Create Board"</summary>
                        <form
                            class="form-grid management-create-form"
                            method="post"
                            action=(href!(boards::create, project_id: id))
                        >
                            <label>
                                "Name"
                                <input type="text" name="name" required="">
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
            </div>
            <details class="management-create">
                <summary class="button secondary">"Edit Project"</summary>
                <form
                    class="form-grid management-create-form"
                    method="post"
                    action=(href!(edit, project_id: id))
                >
                    <label>
                        "Name"
                        <input
                            type="text"
                            name="name"
                            value=(&project.name)
                            required=""
                        >
                    </label>
                    <label>
                        "Description"
                        <textarea name="description" rows="3">
                            (project.description.as_deref().unwrap_or_default())
                        </textarea>
                    </label>
                    <div class="actions">
                        <button class="button primary" type="submit">"Save"</button>
                    </div>
                </form>
            </details>
            if boards.is_empty() {
                <p>"No boards yet."</p>
            } else {
                <div class="management-table projects-table">
                    <div class="heading">"ID"</div>
                    <div class="heading">"Name"</div>
                    <div class="heading">"Description"</div>
                    <div class="heading">"Actions"</div>
                    for board in boards {
                        <div class="data">(board.id)</div>
                        <div class="data">
                            topcoat::runtime::link(
                                href: href!(
                                    boards::board_id::page,
                                    project_id: id,
                                    boards::board_id::board_id: board.id,
                                ),
                                (&board.name)
                            )
                        </div>
                        <div class="data">
                            (board.description.as_deref().unwrap_or_default())
                        </div>
                        <div class="data">
                            <form
                                method="post"
                                action=(href!(
                                    boards::board_id::delete,
                                    project_id: id,
                                    boards::board_id::board_id: board.id,
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
            <p>topcoat::runtime::link(href: href!(super::page), "Back to projects")</p>
        </main>
    })
}

#[route(POST "./edit")]
pub async fn edit(cx: &Cx, Form(input): Form<NamedForm>) -> Result<SeeOther> {
    auth::require_user(cx).await?;
    let id = *project_id(cx)?;
    Project::edit(cx, id, &input.name, &input.description).await?;
    Ok(see_other(href!(page, project_id: id).resolve(cx)))
}

#[route(POST "./delete")]
pub async fn delete(cx: &Cx) -> Result<SeeOther> {
    auth::require_user(cx).await?;
    Project::remove(cx, *project_id(cx)?).await?;
    Ok(see_other(href!(super::page).resolve(cx)))
}
