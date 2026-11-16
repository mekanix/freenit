use topcoat::{Result, context::Cx, router::error::bad_request};

use crate::{
    db::db,
    features::user::{User, UserId},
};

pub const PERMISSIONS: [&str; 9] = [
    "create_project",
    "delete_project",
    "rename_project",
    "create_board",
    "delete_board",
    "rename_board",
    "create_column",
    "delete_column",
    "rename_column",
];

#[derive(Debug, toasty::Model)]
pub struct Project {
    #[key]
    #[auto]
    pub id: u64,
    #[unique]
    pub name: String,
    pub description: Option<String>,
    pub created_by_id: Option<String>,
}

#[derive(Debug, toasty::Model)]
#[unique(project_id, name)]
pub struct Board {
    #[key]
    #[auto]
    pub id: u64,
    #[index]
    pub project_id: u64,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, toasty::Model)]
#[unique(board_id, name)]
pub struct BoardColumn {
    #[key]
    #[auto]
    pub id: u64,
    #[index]
    pub board_id: u64,
    pub name: String,
    pub position: i64,
}

#[derive(Debug, toasty::Model)]
pub struct Task {
    #[key]
    #[auto]
    pub id: u64,
    #[index]
    pub column_id: u64,
    pub title: String,
    pub description: Option<String>,
    pub position: i64,
    pub assignee_id: Option<String>,
    #[index]
    pub parent_id: Option<u64>,
}

pub struct TaskChanges {
    pub title: String,
    pub description: String,
    pub column_id: u64,
    pub position: i64,
    pub assignee_id: Option<String>,
    pub parent_id: Option<u64>,
}

#[derive(Debug, toasty::Model)]
#[unique(project_id, name)]
pub struct ProjectGroup {
    #[key]
    #[auto]
    pub id: u64,
    #[index]
    pub project_id: u64,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, toasty::Model)]
#[unique(group_id, permission)]
pub struct ProjectGroupPermission {
    #[key]
    #[auto]
    pub id: u64,
    #[index]
    pub group_id: u64,
    pub permission: String,
}

#[derive(Debug, toasty::Model)]
#[unique(group_id, user_id)]
pub struct ProjectMember {
    #[key]
    #[auto]
    pub id: u64,
    #[index]
    pub group_id: u64,
    #[index]
    pub user_id: String,
}

impl Project {
    pub async fn list(cx: &Cx) -> Result<Vec<Self>> {
        Ok(Self::all()
            .order_by(Self::fields().id().asc())
            .exec(&mut db(cx))
            .await?)
    }

    pub async fn get(cx: &Cx, id: u64) -> Result<Self> {
        Ok(Self::get_by_id(&mut db(cx), &id).await?)
    }

    pub async fn add(cx: &Cx, user: &User, name: &str, description: &str) -> Result<Self> {
        let name = required(name, "project name")?;
        if Self::filter_by_name(name)
            .first()
            .exec(&mut db(cx))
            .await?
            .is_some()
        {
            return Err(bad_request("project name already exists").into());
        }
        Ok(toasty::create!(Project {
            name: name.to_owned(),
            description: optional(description),
            created_by_id: Some(user.id.to_string()),
        })
        .exec(&mut db(cx))
        .await?)
    }

    pub async fn edit(cx: &Cx, id: u64, name: &str, description: &str) -> Result<()> {
        let name = required(name, "project name")?;
        if Self::filter_by_name(name)
            .first()
            .exec(&mut db(cx))
            .await?
            .is_some_and(|project| project.id != id)
        {
            return Err(bad_request("project name already exists").into());
        }
        let mut project = Self::get(cx, id).await?;
        project
            .update()
            .name(name.to_owned())
            .description(optional(description))
            .exec(&mut db(cx))
            .await?;
        Ok(())
    }

    pub async fn remove(cx: &Cx, id: u64) -> Result<()> {
        for group in ProjectGroup::for_project(cx, id).await? {
            ProjectGroup::remove(cx, group.id).await?;
        }
        for board in Board::for_project(cx, id).await? {
            Board::remove(cx, board.id).await?;
        }
        Self::filter_by_id(id).delete().exec(&mut db(cx)).await?;
        Ok(())
    }
}

impl Board {
    pub async fn for_project(cx: &Cx, project_id: u64) -> Result<Vec<Self>> {
        Ok(Self::filter_by_project_id(project_id)
            .order_by(Self::fields().id().asc())
            .exec(&mut db(cx))
            .await?)
    }

    pub async fn get(cx: &Cx, id: u64) -> Result<Self> {
        Ok(Self::get_by_id(&mut db(cx), &id).await?)
    }

    pub async fn add(cx: &Cx, project_id: u64, name: &str, description: &str) -> Result<Self> {
        Project::get(cx, project_id).await?;
        let name = required(name, "board name")?;
        Self::check_name(cx, project_id, name, None).await?;
        Ok(toasty::create!(Board {
            project_id,
            name: name.to_owned(),
            description: optional(description),
        })
        .exec(&mut db(cx))
        .await?)
    }

    pub async fn edit(cx: &Cx, id: u64, name: &str, description: &str) -> Result<()> {
        let board = Self::get(cx, id).await?;
        let name = required(name, "board name")?;
        Self::check_name(cx, board.project_id, name, Some(id)).await?;
        let mut board = board;
        board
            .update()
            .name(name.to_owned())
            .description(optional(description))
            .exec(&mut db(cx))
            .await?;
        Ok(())
    }

    pub async fn remove(cx: &Cx, id: u64) -> Result<()> {
        for column in BoardColumn::for_board(cx, id).await? {
            BoardColumn::remove(cx, column.id).await?;
        }
        Self::filter_by_id(id).delete().exec(&mut db(cx)).await?;
        Ok(())
    }

    async fn check_name(cx: &Cx, project_id: u64, name: &str, exclude: Option<u64>) -> Result<()> {
        let fields = Self::fields();
        if Self::filter(
            fields
                .project_id()
                .eq(project_id)
                .and(fields.name().eq(name)),
        )
        .first()
        .exec(&mut db(cx))
        .await?
        .is_some_and(|board| Some(board.id) != exclude)
        {
            return Err(bad_request("board name already exists in this project").into());
        }
        Ok(())
    }
}

impl BoardColumn {
    pub async fn for_board(cx: &Cx, board_id: u64) -> Result<Vec<Self>> {
        Ok(Self::filter_by_board_id(board_id)
            .order_by(Self::fields().position().asc())
            .exec(&mut db(cx))
            .await?)
    }

    pub async fn get(cx: &Cx, id: u64) -> Result<Self> {
        Ok(Self::get_by_id(&mut db(cx), &id).await?)
    }

    pub async fn add(cx: &Cx, board_id: u64, name: &str) -> Result<Self> {
        Board::get(cx, board_id).await?;
        let name = required(name, "column name")?;
        Self::check_name(cx, board_id, name, None).await?;
        let position = Self::for_board(cx, board_id)
            .await?
            .iter()
            .map(|column| column.position)
            .max()
            .map_or(0, |position| position + 1);
        Ok(toasty::create!(BoardColumn {
            board_id,
            name: name.to_owned(),
            position
        })
        .exec(&mut db(cx))
        .await?)
    }

    pub async fn edit(cx: &Cx, id: u64, name: &str, position: i64) -> Result<()> {
        let column = Self::get(cx, id).await?;
        let name = required(name, "column name")?;
        Self::check_name(cx, column.board_id, name, Some(id)).await?;
        let mut column = column;
        column
            .update()
            .name(name.to_owned())
            .position(position)
            .exec(&mut db(cx))
            .await?;
        Ok(())
    }

    pub async fn remove(cx: &Cx, id: u64) -> Result<()> {
        for task in Task::for_column(cx, id).await? {
            Task::remove(cx, task.id).await?;
        }
        Self::filter_by_id(id).delete().exec(&mut db(cx)).await?;
        Ok(())
    }

    async fn check_name(cx: &Cx, board_id: u64, name: &str, exclude: Option<u64>) -> Result<()> {
        let fields = Self::fields();
        if Self::filter(fields.board_id().eq(board_id).and(fields.name().eq(name)))
            .first()
            .exec(&mut db(cx))
            .await?
            .is_some_and(|column| Some(column.id) != exclude)
        {
            return Err(bad_request("column name already exists in this board").into());
        }
        Ok(())
    }
}

impl Task {
    pub async fn for_column(cx: &Cx, column_id: u64) -> Result<Vec<Self>> {
        Ok(Self::filter_by_column_id(column_id)
            .order_by(Self::fields().position().asc())
            .exec(&mut db(cx))
            .await?)
    }

    pub async fn for_board(cx: &Cx, board_id: u64) -> Result<Vec<Self>> {
        let mut tasks = Vec::new();
        for column in BoardColumn::for_board(cx, board_id).await? {
            tasks.extend(Self::for_column(cx, column.id).await?);
        }
        Ok(tasks)
    }

    pub async fn get(cx: &Cx, id: u64) -> Result<Self> {
        Ok(Self::get_by_id(&mut db(cx), &id).await?)
    }

    pub async fn add(
        cx: &Cx,
        column_id: u64,
        title: &str,
        description: &str,
        assignee_id: Option<String>,
        parent_id: Option<u64>,
    ) -> Result<Self> {
        let column = BoardColumn::get(cx, column_id).await?;
        let title = required(title, "task title")?;
        if let Some(assignee_id) = assignee_id.as_deref() {
            User::get(cx, &parse_user_id(assignee_id)?).await?;
        }
        if let Some(parent_id) = parent_id {
            let parent = Self::get(cx, parent_id).await?;
            Self::check_board(cx, parent.column_id, column.board_id).await?;
        }
        let position = Self::for_column(cx, column_id)
            .await?
            .iter()
            .map(|task| task.position)
            .max()
            .map_or(0, |position| position + 1);
        Ok(toasty::create!(Task {
            column_id,
            title: title.to_owned(),
            description: optional(description),
            position,
            assignee_id,
            parent_id,
        })
        .exec(&mut db(cx))
        .await?)
    }

    pub async fn edit(cx: &Cx, id: u64, changes: TaskChanges) -> Result<()> {
        let task = Self::get(cx, id).await?;
        let current_column = BoardColumn::get(cx, task.column_id).await?;
        let destination = BoardColumn::get(cx, changes.column_id).await?;
        if destination.board_id != current_column.board_id {
            return Err(bad_request("cannot move a task to another board").into());
        }
        let title = required(&changes.title, "task title")?;
        if let Some(assignee_id) = changes.assignee_id.as_deref() {
            User::get(cx, &parse_user_id(assignee_id)?).await?;
        }
        Self::check_parent(cx, id, changes.parent_id, destination.board_id).await?;
        let mut task = task;
        task.update()
            .title(title.to_owned())
            .description(optional(&changes.description))
            .column_id(changes.column_id)
            .position(changes.position)
            .assignee_id(changes.assignee_id)
            .parent_id(changes.parent_id)
            .exec(&mut db(cx))
            .await?;
        Ok(())
    }

    pub async fn move_to(cx: &Cx, id: u64, column_id: u64) -> Result<()> {
        let task = Self::get(cx, id).await?;
        Self::edit(
            cx,
            id,
            TaskChanges {
                title: task.title,
                description: task.description.unwrap_or_default(),
                column_id,
                position: task.position,
                assignee_id: task.assignee_id,
                parent_id: task.parent_id,
            },
        )
        .await
    }

    pub async fn remove(cx: &Cx, id: u64) -> Result<()> {
        let children = Self::filter_by_parent_id(id).exec(&mut db(cx)).await?;
        for mut child in children {
            child.update().parent_id(None).exec(&mut db(cx)).await?;
        }
        Self::filter_by_id(id).delete().exec(&mut db(cx)).await?;
        Ok(())
    }

    async fn check_parent(cx: &Cx, id: u64, parent_id: Option<u64>, board_id: u64) -> Result<()> {
        let Some(mut parent_id) = parent_id else {
            return Ok(());
        };
        if parent_id == id {
            return Err(bad_request("task cannot be its own parent").into());
        }
        loop {
            let parent = Self::get(cx, parent_id).await?;
            Self::check_board(cx, parent.column_id, board_id).await?;
            if parent.id == id {
                return Err(bad_request("task parent cannot be a descendant").into());
            }
            let Some(next) = parent.parent_id else {
                break;
            };
            parent_id = next;
        }
        Ok(())
    }

    async fn check_board(cx: &Cx, column_id: u64, board_id: u64) -> Result<()> {
        if BoardColumn::get(cx, column_id).await?.board_id != board_id {
            return Err(bad_request("parent task must belong to the same board").into());
        }
        Ok(())
    }
}

impl ProjectGroup {
    pub async fn for_project(cx: &Cx, project_id: u64) -> Result<Vec<Self>> {
        Ok(Self::filter_by_project_id(project_id)
            .order_by(Self::fields().id().asc())
            .exec(&mut db(cx))
            .await?)
    }

    pub async fn get(cx: &Cx, id: u64) -> Result<Self> {
        Ok(Self::get_by_id(&mut db(cx), &id).await?)
    }

    pub async fn add(
        cx: &Cx,
        project_id: u64,
        name: &str,
        description: &str,
        permissions: Vec<String>,
    ) -> Result<Self> {
        Project::get(cx, project_id).await?;
        let name = required(name, "group name")?;
        Self::check_name(cx, project_id, name, None).await?;
        let group = toasty::create!(ProjectGroup {
            project_id,
            name: name.to_owned(),
            description: optional(description),
        })
        .exec(&mut db(cx))
        .await?;
        Self::set_permissions(cx, group.id, permissions).await?;
        Ok(group)
    }

    pub async fn edit(
        cx: &Cx,
        id: u64,
        name: &str,
        description: &str,
        permissions: Vec<String>,
    ) -> Result<()> {
        let group = Self::get(cx, id).await?;
        let name = required(name, "group name")?;
        Self::check_name(cx, group.project_id, name, Some(id)).await?;
        let mut group = group;
        group
            .update()
            .name(name.to_owned())
            .description(optional(description))
            .exec(&mut db(cx))
            .await?;
        Self::set_permissions(cx, id, permissions).await
    }

    pub async fn remove(cx: &Cx, id: u64) -> Result<()> {
        ProjectGroupPermission::filter_by_group_id(id)
            .delete()
            .exec(&mut db(cx))
            .await?;
        ProjectMember::filter_by_group_id(id)
            .delete()
            .exec(&mut db(cx))
            .await?;
        Self::filter_by_id(id).delete().exec(&mut db(cx)).await?;
        Ok(())
    }

    pub async fn permissions(cx: &Cx, id: u64) -> Result<Vec<String>> {
        Ok(ProjectGroupPermission::filter_by_group_id(id)
            .order_by(ProjectGroupPermission::fields().permission().asc())
            .exec(&mut db(cx))
            .await?
            .into_iter()
            .map(|permission| permission.permission)
            .collect())
    }

    async fn set_permissions(cx: &Cx, id: u64, permissions: Vec<String>) -> Result<()> {
        ProjectGroupPermission::filter_by_group_id(id)
            .delete()
            .exec(&mut db(cx))
            .await?;
        let mut permissions = permissions;
        permissions.sort();
        permissions.dedup();
        for permission in permissions {
            if !PERMISSIONS.contains(&permission.as_str()) {
                return Err(bad_request("unknown project permission").into());
            }
            toasty::create!(ProjectGroupPermission {
                group_id: id,
                permission
            })
            .exec(&mut db(cx))
            .await?;
        }
        Ok(())
    }

    async fn check_name(cx: &Cx, project_id: u64, name: &str, exclude: Option<u64>) -> Result<()> {
        let fields = Self::fields();
        if Self::filter(
            fields
                .project_id()
                .eq(project_id)
                .and(fields.name().eq(name)),
        )
        .first()
        .exec(&mut db(cx))
        .await?
        .is_some_and(|group| Some(group.id) != exclude)
        {
            return Err(bad_request("group name already exists in this project").into());
        }
        Ok(())
    }
}

impl ProjectMember {
    pub async fn member_ids(cx: &Cx, group_id: u64) -> Result<Vec<String>> {
        Ok(Self::filter_by_group_id(group_id)
            .exec(&mut db(cx))
            .await?
            .into_iter()
            .map(|member| member.user_id)
            .collect())
    }

    pub async fn assign(cx: &Cx, group_id: u64, user_id: String) -> Result<()> {
        ProjectGroup::get(cx, group_id).await?;
        User::get(cx, &parse_user_id(&user_id)?).await?;
        if Self::member_ids(cx, group_id).await?.contains(&user_id) {
            return Ok(());
        }
        toasty::create!(ProjectMember { group_id, user_id })
            .exec(&mut db(cx))
            .await?;
        Ok(())
    }

    pub async fn unassign(cx: &Cx, group_id: u64, user_id: String) -> Result<()> {
        let fields = Self::fields();
        Self::filter(
            fields
                .group_id()
                .eq(group_id)
                .and(fields.user_id().eq(user_id)),
        )
        .delete()
        .exec(&mut db(cx))
        .await?;
        Ok(())
    }
}

fn required<'a>(value: &'a str, label: &str) -> Result<&'a str> {
    let value = value.trim();
    if value.is_empty() {
        Err(bad_request(format!("{label} is required")).into())
    } else {
        Ok(value)
    }
}

fn parse_user_id(id: &str) -> Result<UserId> {
    id.parse::<UserId>()
        .map_err(|error| bad_request(format!("invalid user ID: {error}")).into())
}

fn optional(value: &str) -> Option<String> {
    let value = value.trim();
    (!value.is_empty()).then(|| value.to_owned())
}
