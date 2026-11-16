use serde::Deserialize;
use topcoat::{
    Result,
    context::Cx,
    router::{
        content::Form,
        error::{SeeOther, bad_request, see_other},
        href, module_param, page, route,
    },
    view::{View, view},
};

use crate::{
    app::projects::{optional_number, optional_string},
    features::{
        auth,
        kanban::{Board, BoardColumn, Task},
        user::User,
    },
};

use super::{board_id, column_id, project_id};

module_param!(pub task_id: u64 else bad_request);

#[page]
pub async fn page(cx: &Cx) -> Result<impl View> {
    auth::require_user(cx).await?;
    let project_id = *project_id(cx)?;
    let board_id = *board_id(cx)?;
    let task_id = *task_id(cx)?;
    let task = Task::get(cx, task_id).await?;
    if task.column_id != *column_id(cx)? {
        return Err(bad_request("task does not belong to this column").into());
    }
    let board = Board::get(cx, board_id).await?;
    if board.project_id != project_id {
        return Err(bad_request("board does not belong to this project").into());
    }
    let columns = BoardColumn::for_board(cx, board_id).await?;
    let tasks = Task::for_board(cx, board_id).await?;
    let users = User::list(cx).await?;

    Ok(view! {
        <main class="container kanban-task-page">
            <p>
                topcoat::runtime::link(
                    href: href!(
                        super::super::super::super::page,
                        project_id: project_id,
                        board_id: board_id,
                    ),
                    attrs: topcoat::view::attributes! { class="button outline" },
                    "Back to board"
                )
            </p>
            <div class="kanban-task-detail">
                <h1>(&task.title)</h1>
                <section>
                    <span class="kanban-label">"Description"</span>
                    <p class="kanban-description">
                        (task.description.as_deref().unwrap_or("No description"))
                    </p>
                </section>
                <section>
                    <span class="kanban-label">"Assignee"</span>
                    if let Some(assignee) = users
                        .iter()
                        .find(|user| Some(user.id.to_string()) == task.assignee_id) {
                        <span>(&assignee.email)</span>
                    } else {
                        <span class="empty-state">"Unassigned"</span>
                    }
                </section>
                <section>
                    <span class="kanban-label">"Parent task"</span>
                    if let Some(parent) = tasks
                        .iter()
                        .find(|candidate| Some(candidate.id) == task.parent_id) {
                        topcoat::runtime::link(
                            href: task_url(cx, parent),
                            (&parent.title)
                        )
                    } else {
                        <span class="empty-state">"No parent task"</span>
                    }
                </section>
                <section>
                    <span class="kanban-label">"Child tasks"</span>
                    if tasks.iter().any(|child| child.parent_id == Some(task.id)) {
                        <ul>
                            for child in tasks
                                .iter()
                                .filter(|child| child.parent_id == Some(task.id)) {
                                <li>
                                    topcoat::runtime::link(
                                        href: task_url(cx, child),
                                        (&child.title)
                                    )
                                </li>
                            }
                        </ul>
                    } else {
                        <span class="empty-state">"No child tasks"</span>
                    }
                </section>
            </div>
            <section class="kanban-task-editor">
                <h2>"Edit Task"</h2>
                <form
                    class="form-grid"
                    method="post"
                    action=(href!(
                        edit,
                        project_id: project_id,
                        board_id: board_id,
                        column_id: task.column_id,
                        task_id: task_id,
                    ))
                >
                    <label>
                        "Title"
                        <input type="text" name="title" value=(&task.title) required="">
                    </label>
                    <label>
                        "Description"
                        <textarea name="description" rows="6">
                            (task.description.as_deref().unwrap_or_default())
                        </textarea>
                    </label>
                    <label>
                        "Column"
                        <select name="column_id">
                            for column in &columns {
                                <option
                                    value=(column.id)
                                    selected=(column.id == task.column_id)
                                >
                                    (&column.name)
                                </option>
                            }
                        </select>
                    </label>
                    <label>
                        "Position"
                        <input
                            type="number"
                            name="position"
                            value=(task.position)
                            required=""
                        >
                    </label>
                    <label>
                        "Assignee"
                        <select name="assignee_id">
                            <option value="">"Unassigned"</option>
                            for user in &users {
                                let user_id = user.id.to_string();
                                let selected = task.assignee_id.as_deref()
                                    == Some(user_id.as_str());
                                <option value=(user_id) selected=(selected)>
                                    (&user.email)
                                </option>
                            }
                        </select>
                    </label>
                    <label>
                        "Parent task"
                        <select name="parent_id">
                            <option value="">"No parent task"</option>
                            for candidate in tasks
                                .iter()
                                .filter(|candidate| candidate.id != task.id) {
                                <option
                                    value=(candidate.id)
                                    selected=(task.parent_id == Some(candidate.id))
                                >
                                    (&candidate.title)
                                </option>
                            }
                        </select>
                    </label>
                    <div class="actions">
                        <button class="button primary" type="submit">"Save"</button>
                    </div>
                </form>
                <form
                    method="post"
                    action=(href!(
                        delete,
                        project_id: project_id,
                        board_id: board_id,
                        column_id: task.column_id,
                        task_id: task_id,
                    ))
                >
                    <button class="button error" type="submit">"Delete Task"</button>
                </form>
            </section>
        </main>
    })
}

#[derive(Deserialize)]
struct TaskEdit {
    title: String,
    #[serde(default)]
    description: String,
    column_id: u64,
    position: i64,
    #[serde(default, deserialize_with = "optional_string")]
    assignee_id: Option<String>,
    #[serde(default, deserialize_with = "optional_number")]
    parent_id: Option<u64>,
}

#[route(POST "./edit")]
async fn edit(cx: &Cx, Form(input): Form<TaskEdit>) -> Result<SeeOther> {
    auth::require_user(cx).await?;
    let project_id = *project_id(cx)?;
    let board_id = *board_id(cx)?;
    let task_id = *task_id(cx)?;
    Task::edit(
        cx,
        task_id,
        crate::features::kanban::TaskChanges {
            title: input.title,
            description: input.description,
            column_id: input.column_id,
            position: input.position,
            assignee_id: input.assignee_id,
            parent_id: input.parent_id,
        },
    )
    .await?;
    Ok(see_other(
        href!(
            page,
            project_id: project_id,
            board_id: board_id,
            column_id: input.column_id,
            task_id: task_id,
        )
        .resolve(cx),
    ))
}

#[route(POST "./delete")]
async fn delete(cx: &Cx) -> Result<SeeOther> {
    auth::require_user(cx).await?;
    let project_id = *project_id(cx)?;
    let board_id = *board_id(cx)?;
    Task::remove(cx, *task_id(cx)?).await?;
    Ok(see_other(
        href!(
            super::super::super::super::page,
            project_id: project_id,
            board_id: board_id,
        )
        .resolve(cx),
    ))
}

fn task_url(cx: &Cx, task: &Task) -> String {
    href!(
        page,
        project_id: *project_id(cx).unwrap_or(&0),
        board_id: *board_id(cx).unwrap_or(&0),
        column_id: task.column_id,
        task_id: task.id,
    )
    .resolve(cx)
}
