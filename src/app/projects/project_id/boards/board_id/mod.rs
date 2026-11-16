pub mod columns;

use topcoat::{
    Result,
    context::Cx,
    router::{
        content::Form,
        error::{SeeOther, bad_request, see_other},
        href, module_param, page, query_params, route,
    },
    runtime::{Event, Signal, procedure, signal},
    view::{View, view},
};

use crate::{
    app::projects::NamedForm,
    features::{
        auth,
        kanban::{Board, BoardColumn, Task},
        user::User,
    },
};

use super::project_id;

module_param!(pub board_id: u64 else bad_request);

#[query_params(error = bad_request)]
struct BoardQuery {
    task: Option<u64>,
}

#[page]
pub async fn page(cx: &Cx) -> Result<impl View> {
    auth::require_user(cx).await?;
    let project_id = *project_id(cx)?;
    let board_id = *board_id(cx)?;
    let board = Board::get(cx, board_id).await?;
    if board.project_id != project_id {
        return Err(bad_request("board does not belong to this project").into());
    }
    let columns = BoardColumn::for_board(cx, board_id).await?;
    let tasks = Task::for_board(cx, board_id).await?;
    let users = User::list(cx).await?;
    let query = query_params::<BoardQuery>(cx)?;
    let selected_task = match query.task {
        Some(task_id) => {
            let task = Task::get(cx, task_id).await?;
            if BoardColumn::get(cx, task.column_id).await?.board_id != board_id {
                return Err(bad_request("task does not belong to this board").into());
            }
            Some(task)
        }
        None => None,
    };
    let board_url = href!(page, project_id: project_id, board_id: board_id).resolve(cx);
    let dragged_task = signal(cx, || 0u64);
    let drag_over_column = signal(cx, || 0u64);
    let move_version = signal(cx, || 0u64);
    let _move_version = move_version.get();

    Ok(view! {
        <main
            class="container kanban-page"
            @dragend=$(|_e| {
                dragged_task.set(0u64);
                drag_over_column.set(0u64);
            })
        >
            <div class="management-header">
                <div>
                    <h2>(&board.name)</h2>
                    <p>(board.description.as_deref().unwrap_or_default())</p>
                </div>
                <details class="management-create">
                    <summary class="button secondary">"Edit Board"</summary>
                    <form
                        class="form-grid management-create-form"
                        method="post"
                        action=(href!(edit, project_id: project_id, board_id: board_id))
                    >
                        <label>
                            "Name"
                            <input
                                type="text"
                                name="name"
                                value=(&board.name)
                                required=""
                            >
                        </label>
                        <label>
                            "Description"
                            <textarea name="description" rows="3">
                                (board.description.as_deref().unwrap_or_default())
                            </textarea>
                        </label>
                        <div class="actions">
                            <button class="button primary" type="submit">"Save"</button>
                        </div>
                    </form>
                </details>
            </div>
            <p>
                topcoat::runtime::link(
                    href: href!(super::super::page, project_id: project_id),
                    "Back to project"
                )
            </p>
            <div class="kanban-board">
                for column in &columns {
                    let column_id = column.id;
                    let column_tasks: Vec<_> = tasks
                        .iter()
                        .filter(|task| task.column_id == column.id)
                        .collect();
                    <section
                        :class=$(if drag_over_column.get() == column_id {
                            "kanban-column drag-over"
                        } else {
                            "kanban-column"
                        })
                        @dragover=$(|e: Event| {
                            e.prevent_default();
                            if drag_over_column.get() != column_id {
                                drag_over_column.set(column_id);
                            }
                        })
                        @dragleave=$(|_e| {
                            if drag_over_column.get() == column_id {
                                drag_over_column.set(0u64);
                            }
                        })
                        @drop=$(async |e: Event| {
                            e.prevent_default();
                            if dragged_task.get() != 0u64 {
                                move_task(dragged_task.get(), column_id).await;
                                move_version.increment();
                            }
                            dragged_task.set(0u64);
                            drag_over_column.set(0u64);
                        })
                    >
                        <header class="kanban-column-header">
                            <strong>(&column.name)</strong>
                            <form
                                method="post"
                                action=(href!(
                                    columns::column_id::delete,
                                    project_id: project_id,
                                    board_id: board_id,
                                    columns::column_id::column_id: column.id,
                                ))
                            >
                                <button
                                    class="button small clear error"
                                    type="submit"
                                    title="Delete column"
                                >
                                    "×"
                                </button>
                            </form>
                        </header>
                        <details class="kanban-edit">
                            <summary>"Edit column"</summary>
                            <form
                                class="form-grid"
                                method="post"
                                action=(href!(
                                    columns::column_id::edit,
                                    project_id: project_id,
                                    board_id: board_id,
                                    columns::column_id::column_id: column.id,
                                ))
                            >
                                <input
                                    type="text"
                                    name="name"
                                    value=(&column.name)
                                    required=""
                                >
                                <input
                                    type="number"
                                    name="position"
                                    value=(column.position)
                                    aria-label="Column position"
                                    required=""
                                >
                                <button class="button outline" type="submit">
                                    "Save column"
                                </button>
                            </form>
                        </details>
                        <div class="kanban-tasks" role="list" aria-label=(&column.name)>
                            for task in &column_tasks {
                                if task.parent_id.is_none_or(
                                    |parent_id| {
                                        !column_tasks
                                            .iter()
                                            .any(|candidate| candidate.id == parent_id)
                                    },
                                ) {
                                    let children: Vec<_> = column_tasks
                                        .iter()
                                        .filter(|candidate| candidate.parent_id == Some(task.id))
                                        .collect();
                                    task_card(
                                        task: task,
                                        dragged: &dragged_task,
                                        selected: selected_task
                                            .as_ref()
                                            .is_some_and(|selected| selected.id == task.id),
                                        child_count: children.len()
                                    )
                                    if !children.is_empty() {
                                        <div class="kanban-children">
                                            for child in children {
                                                task_card(
                                                    task: child,
                                                    dragged: &dragged_task,
                                                    selected: selected_task
                                                        .as_ref()
                                                        .is_some_and(|selected| selected.id == child.id),
                                                    child_count: 0
                                                )
                                            }
                                        </div>
                                    }
                                }
                            }
                        </div>
                        <details class="kanban-create">
                            <summary class="button small">"+ Task"</summary>
                            <form
                                class="form-grid"
                                method="post"
                                action=(href!(
                                    columns::column_id::tasks::create,
                                    project_id: project_id,
                                    board_id: board_id,
                                    columns::column_id::column_id: column.id,
                                ))
                            >
                                <input
                                    type="text"
                                    name="title"
                                    placeholder="Title"
                                    aria-label="Task title"
                                    required=""
                                >
                                <textarea
                                    name="description"
                                    rows="3"
                                    placeholder="Description"
                                ></textarea>
                                <select name="assignee_id" aria-label="Assignee">
                                    <option value="">"Unassigned"</option>
                                    for user in &users {
                                        <option value=(user.id.to_string())>(&user.email)</option>
                                    }
                                </select>
                                <select name="parent_id" aria-label="Parent task">
                                    <option value="">"No parent task"</option>
                                    for parent in &tasks {
                                        <option value=(parent.id)>(&parent.title)</option>
                                    }
                                </select>
                                <button class="button primary" type="submit">
                                    "Create"
                                </button>
                            </form>
                        </details>
                    </section>
                }
                <details class="management-create kanban-add-column">
                    <summary class="button outline">"+ Column"</summary>
                    <form
                        class="form-grid management-create-form"
                        method="post"
                        action=(href!(
                            columns::create,
                            project_id: project_id,
                            board_id: board_id,
                        ))
                    >
                        <input
                            type="text"
                            name="name"
                            placeholder="Column name"
                            required=""
                        >
                        <button class="button primary" type="submit">"Create"</button>
                    </form>
                </details>
            </div>
            if let Some(selected) = &selected_task {
                <aside class="kanban-task-pane">
                    <a
                        class="button clear kanban-pane-close"
                        href=(&board_url)
                        aria-label="Close task"
                    >
                        "×"
                    </a>
                    <h2>
                        topcoat::runtime::link(
                            href: task_url(cx, selected),
                            (&selected.title)
                        )
                    </h2>
                    <section>
                        <span class="kanban-label">"Description"</span>
                        <p>
                            (selected.description.as_deref().unwrap_or("No description"))
                        </p>
                    </section>
                    <section>
                        <span class="kanban-label">"Parent task"</span>
                        if let Some(parent) = tasks
                            .iter()
                            .find(|task| Some(task.id) == selected.parent_id) {
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
                        <ul>
                            for child in tasks
                                .iter()
                                .filter(|task| task.parent_id == Some(selected.id)) {
                                <li>
                                    topcoat::runtime::link(
                                        href: task_url(cx, child),
                                        (&child.title)
                                    )
                                </li>
                            }
                        </ul>
                    </section>
                </aside>
            }
        </main>
    })
}

#[topcoat::view::component]
async fn task_card(
    cx: &Cx,
    task: &&Task,
    dragged: &Signal<u64>,
    selected: bool,
    child_count: usize,
) -> Result<impl View> {
    let url = format!(
        "{}?task={}",
        href!(
            page,
            project_id: *project_id(cx).unwrap_or(&0),
            board_id: *board_id(cx).unwrap_or(&0),
        )
        .resolve(cx),
        task.id,
    );
    let task_id = task.id;
    Ok(view! {
        <a
            class=(if selected {
                "kanban-task-card selected"
            } else {
                "kanban-task-card"
            })
            href=(url)
            draggable="true"
            @dragstart=$(|_e| dragged.set(task_id))
        >
            <span class="kanban-task-title">
                (&task.title)
                if child_count > 0 {
                    <span class="kanban-task-count">
                        " ("
                        (child_count)
                        ")"
                    </span>
                }
            </span>
            if let Some(description) = &task.description {
                <span class="kanban-task-description">(description)</span>
            }
        </a>
    })
}

fn task_url(cx: &Cx, task: &Task) -> String {
    href!(
        columns::column_id::tasks::task_id::page,
        project_id: *project_id(cx).unwrap_or(&0),
        board_id: *board_id(cx).unwrap_or(&0),
        columns::column_id::column_id: task.column_id,
        columns::column_id::tasks::task_id::task_id: task.id,
    )
    .resolve(cx)
}

#[route(POST "./edit")]
async fn edit(cx: &Cx, Form(input): Form<NamedForm>) -> Result<SeeOther> {
    auth::require_user(cx).await?;
    let project_id = *project_id(cx)?;
    let board_id = *board_id(cx)?;
    Board::edit(cx, board_id, &input.name, &input.description).await?;
    Ok(see_other(
        href!(page, project_id: project_id, board_id: board_id).resolve(cx),
    ))
}

#[route(POST "./delete")]
pub async fn delete(cx: &Cx) -> Result<SeeOther> {
    auth::require_user(cx).await?;
    let project_id = *project_id(cx)?;
    Board::remove(cx, *board_id(cx)?).await?;
    Ok(see_other(
        href!(super::super::page, project_id: project_id).resolve(cx),
    ))
}

#[procedure]
async fn move_task(cx: &Cx, task_id: u64, column_id: u64) -> Result<()> {
    auth::require_user(cx).await?;
    Task::move_to(cx, task_id, column_id).await
}
