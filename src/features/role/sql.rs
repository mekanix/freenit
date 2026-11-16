use topcoat::{Result, context::Cx};

use crate::{db::db, features::user::sql::User};

pub type RoleId = u64;

#[derive(Debug, toasty::Model)]
pub struct Role {
    #[key]
    #[auto]
    pub id: RoleId,

    #[unique]
    pub name: String,
}

#[derive(Debug, toasty::Model)]
#[unique(user_id, role_id)]
pub struct UserRole {
    #[key]
    #[auto]
    pub id: u64,

    #[index]
    pub user_id: u64,

    #[belongs_to]
    pub user: toasty::Deferred<User>,

    #[index]
    pub role_id: RoleId,

    #[belongs_to]
    pub role: toasty::Deferred<Role>,
}

impl Role {
    pub async fn list(cx: &Cx) -> Result<Vec<Role>> {
        let roles = Role::all()
            .order_by(Role::fields().id().asc())
            .exec(&mut db(cx))
            .await?;
        Ok(roles)
    }

    pub async fn get(cx: &Cx, id: &RoleId) -> Result<Role> {
        Ok(Role::get_by_id(&mut db(cx), id).await?)
    }

    pub async fn add(cx: &Cx, name: &str, user_id: u64) -> Result<()> {
        let name = name.trim();
        if name.is_empty()
            || Role::filter_by_name(name)
                .first()
                .exec(&mut db(cx))
                .await?
                .is_some()
        {
            return Ok(());
        }

        let role = toasty::create!(Role { name }).exec(&mut db(cx)).await?;
        toasty::create!(UserRole {
            user_id,
            role_id: role.id,
        })
        .exec(&mut db(cx))
        .await?;
        Ok(())
    }

    pub async fn remove(cx: &Cx, id: &RoleId) -> Result<()> {
        let mut db = db(cx);
        UserRole::filter_by_role_id(*id)
            .delete()
            .exec(&mut db)
            .await?;
        Role::filter_by_id(*id).delete().exec(&mut db).await?;
        Ok(())
    }

    pub async fn for_user(cx: &Cx, user_id: u64) -> Result<Vec<Role>> {
        let links = UserRole::filter_by_user_id(user_id)
            .order_by(UserRole::fields().id().asc())
            .exec(&mut db(cx))
            .await?;
        let mut roles = Vec::with_capacity(links.len());
        for link in links {
            roles.push(Role::get_by_id(&mut db(cx), &link.role_id).await?);
        }
        Ok(roles)
    }

    pub async fn users(cx: &Cx, role_id: &RoleId) -> Result<Vec<User>> {
        let links = UserRole::filter_by_role_id(*role_id)
            .order_by(UserRole::fields().id().asc())
            .exec(&mut db(cx))
            .await?;
        let mut users = Vec::with_capacity(links.len());
        for link in links {
            users.push(User::get_by_id(&mut db(cx), &link.user_id).await?);
        }
        Ok(users)
    }

    pub async fn assign(cx: &Cx, role_id: &RoleId, user_id: u64) -> Result<()> {
        let mut db = db(cx);
        User::get_by_id(&mut db, &user_id).await?;
        Role::get_by_id(&mut db, role_id).await?;
        if Role::for_user(cx, user_id)
            .await?
            .iter()
            .any(|role| role.id == *role_id)
        {
            return Ok(());
        }

        toasty::create!(UserRole {
            user_id,
            role_id: *role_id,
        })
        .exec(&mut db)
        .await?;
        Ok(())
    }

    pub async fn unassign(cx: &Cx, role_id: &RoleId, user_id: u64) -> Result<()> {
        let fields = UserRole::fields();
        UserRole::filter(
            fields
                .user_id()
                .eq(user_id)
                .and(fields.role_id().eq(*role_id)),
        )
        .delete()
        .exec(&mut db(cx))
        .await?;
        Ok(())
    }
}


impl From<Role> for super::Role {
    fn from(role: Role) -> Self {
        Self {
            id: super::RoleId::Sql(role.id),
            name: role.name,
        }
    }
}
