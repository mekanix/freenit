use crate::{db::db, features::user::sql::User};
use topcoat::{Result, context::Cx};

pub type GroupId = u64;

#[derive(Debug, toasty::Model)]
pub struct Group {
    #[key]
    #[auto]
    pub id: GroupId,
    #[unique]
    pub name: String,
}

#[derive(Debug, toasty::Model)]
#[unique(user_id, group_id)]
pub struct UserGroup {
    #[key]
    #[auto]
    pub id: u64,
    #[index]
    pub user_id: u64,
    #[belongs_to]
    pub user: toasty::Deferred<User>,
    #[index]
    pub group_id: GroupId,
    #[belongs_to]
    pub group: toasty::Deferred<Group>,
}

impl Group {
    pub async fn list(cx: &Cx) -> Result<Vec<Group>> {
        Ok(Group::all()
            .order_by(Group::fields().name().asc())
            .exec(&mut db(cx))
            .await?)
    }

    pub async fn get(cx: &Cx, id: &GroupId) -> Result<Group> {
        Ok(Group::get_by_id(&mut db(cx), id).await?)
    }

    pub async fn add(cx: &Cx, name: &str, _domain: &str, user_id: u64) -> Result<()> {
        let name = name.trim();
        if name.is_empty() {
            return Err(topcoat::router::error::bad_request("group name is required").into());
        }
        User::get(cx, user_id).await?;
        if Group::filter_by_name(name)
            .first()
            .exec(&mut db(cx))
            .await?
            .is_some()
        {
            return Err(topcoat::router::error::bad_request("group already exists").into());
        }
        let group = toasty::create!(Group {
            name: name.to_owned()
        })
        .exec(&mut db(cx))
        .await?;
        Self::assign(cx, &group.id, user_id).await
    }

    pub async fn remove(cx: &Cx, id: &GroupId) -> Result<()> {
        let mut db = db(cx);
        UserGroup::filter_by_group_id(*id)
            .delete()
            .exec(&mut db)
            .await?;
        Group::filter_by_id(*id).delete().exec(&mut db).await?;
        Ok(())
    }

    pub async fn users(cx: &Cx, group_id: &GroupId) -> Result<Vec<User>> {
        let links = UserGroup::filter_by_group_id(*group_id)
            .order_by(UserGroup::fields().id().asc())
            .exec(&mut db(cx))
            .await?;
        let mut users = Vec::with_capacity(links.len());
        for link in links {
            users.push(User::get(cx, link.user_id).await?);
        }
        Ok(users)
    }

    pub async fn assign(cx: &Cx, group_id: &GroupId, user_id: u64) -> Result<()> {
        Self::get(cx, group_id).await?;
        User::get(cx, user_id).await?;
        if Self::users(cx, group_id)
            .await?
            .iter()
            .any(|user| user.id == user_id)
        {
            return Ok(());
        }
        toasty::create!(UserGroup {
            user_id,
            group_id: *group_id
        })
        .exec(&mut db(cx))
        .await?;
        Ok(())
    }

    pub async fn unassign(cx: &Cx, group_id: &GroupId, user_id: u64) -> Result<()> {
        let fields = UserGroup::fields();
        UserGroup::filter(
            fields
                .user_id()
                .eq(user_id)
                .and(fields.group_id().eq(*group_id)),
        )
        .delete()
        .exec(&mut db(cx))
        .await?;
        Ok(())
    }
}


impl From<Group> for super::Group {
    fn from(group: Group) -> Self {
        Self {
            id: super::GroupId::Sql(group.id),
            name: group.name,
        }
    }
}
