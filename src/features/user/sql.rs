use argon2::{Argon2, PasswordHash, PasswordVerifier, password_hash::PasswordHasher};
use topcoat::{Result, context::Cx, router::error::internal_server_error};

use crate::{
    db::db,
    features::{group::UserGroup, role::UserRole},
};

#[derive(Debug, toasty::Model)]
pub struct User {
    #[key]
    #[auto]
    pub id: u64,

    #[unique]
    pub email: String,

    /// Password hash in Argon2 PHC format.
    password: String,

    pub fullname: Option<String>,

    pub active: bool,

    pub admin: bool,

    pub omemo_bundle: Option<String>,
}

impl User {
    pub async fn list(cx: &Cx) -> Result<Vec<User>> {
        let users = User::all()
            .order_by(User::fields().id().asc())
            .exec(&mut db(cx))
            .await?;
        Ok(users)
    }

    pub async fn get(cx: &Cx, id: u64) -> Result<User> {
        Ok(User::get_by_id(&mut db(cx), &id).await?)
    }

    pub async fn find_by_email(cx: &Cx, email: &str) -> Result<Option<User>> {
        Ok(User::filter_by_email(email.trim().to_lowercase())
            .first()
            .exec(&mut db(cx))
            .await?)
    }

    pub async fn authenticate(cx: &Cx, email: &str, password: String) -> Result<Option<User>> {
        let Some(user) = Self::find_by_email(cx, email).await? else {
            return Ok(None);
        };
        let hash = user.password.clone();
        let valid = tokio::task::spawn_blocking(move || {
            PasswordHash::new(&hash).is_ok_and(|hash| {
                Argon2::default()
                    .verify_password(password.as_bytes(), &hash)
                    .is_ok()
            })
        })
        .await
        .map_err(internal_server_error)?;
        Ok(valid.then_some(user))
    }

    pub async fn from_token(cx: &Cx, id: &str) -> Result<User> {
        let id = id
            .parse()
            .map_err(|_| topcoat::router::error::unauthorized())?;
        Self::get(cx, id).await
    }

    pub fn token_id(&self) -> String {
        self.id.to_string()
    }

    pub async fn add(
        cx: &Cx,
        email: &str,
        password: String,
        fullname: &str,
        active: bool,
        admin: bool,
    ) -> Result<User> {
        let email = email.trim().to_lowercase();
        if email.is_empty() || password.is_empty() {
            return Err(
                topcoat::router::error::bad_request("email and password are required").into(),
            );
        }
        if Self::find_by_email(cx, &email).await?.is_some() {
            return Err(topcoat::router::error::bad_request("user already exists").into());
        }

        let password = tokio::task::spawn_blocking(move || hash_password(&password))
            .await
            .map_err(internal_server_error)?;
        let fullname = (!fullname.trim().is_empty()).then(|| fullname.trim().to_owned());

        toasty::create!(User {
            email: email.clone(),
            password,
            fullname,
            active,
            admin,
            omemo_bundle: None,
        })
        .exec(&mut db(cx))
        .await?;
        Ok(Self::find_by_email(cx, &email)
            .await?
            .expect("the newly created user can be loaded"))
    }

    #[cfg(feature = "jabber")]
    pub fn omemo_key(&self) -> String {
        self.id.to_string()
    }

    #[cfg(feature = "jabber")]
    pub async fn set_omemo_bundle(cx: &Cx, id: u64, bundle: Option<String>) -> Result<()> {
        let mut user = User::get_by_id(&mut db(cx), &id).await?;
        user.update().omemo_bundle(bundle).exec(&mut db(cx)).await?;
        Ok(())
    }

    pub async fn activate(cx: &Cx, id: u64) -> Result<()> {
        let mut user = Self::get(cx, id).await?;
        user.update().active(true).exec(&mut db(cx)).await?;
        Ok(())
    }

    pub async fn set_password(cx: &Cx, id: u64, password: String) -> Result<()> {
        if password.is_empty() {
            return Err(topcoat::router::error::bad_request("password is required").into());
        }
        let password = tokio::task::spawn_blocking(move || hash_password(&password))
            .await
            .map_err(internal_server_error)?;
        let mut user = Self::get(cx, id).await?;
        user.update().password(password).exec(&mut db(cx)).await?;
        Ok(())
    }

    pub async fn toggle_active(cx: &Cx, id: u64) -> Result<()> {
        let mut user = User::get_by_id(&mut db(cx), &id).await?;
        let active = !user.active;
        user.update().active(active).exec(&mut db(cx)).await?;
        Ok(())
    }

    pub async fn set_admin(cx: &Cx, id: u64, admin: bool) -> Result<()> {
        let mut user = User::get_by_id(&mut db(cx), &id).await?;
        user.update().admin(admin).exec(&mut db(cx)).await?;
        Ok(())
    }

    pub async fn toggle_admin(cx: &Cx, id: u64) -> Result<()> {
        let user = User::get_by_id(&mut db(cx), &id).await?;
        Self::set_admin(cx, id, !user.admin).await
    }

    pub async fn remove(cx: &Cx, id: u64) -> Result<()> {
        let mut db = db(cx);
        UserRole::filter_by_user_id(id)
            .delete()
            .exec(&mut db)
            .await?;
        UserGroup::filter_by_user_id(id)
            .delete()
            .exec(&mut db)
            .await?;
        User::filter_by_id(id).delete().exec(&mut db).await?;
        Ok(())
    }
}

fn hash_password(password: &str) -> String {
    Argon2::default()
        .hash_password(password.as_bytes())
        .expect("hashing a password with the default parameters succeeds")
        .to_string()
}


impl From<User> for super::User {
    fn from(user: User) -> Self {
        Self {
            id: super::UserId::Sql(user.id),
            email: user.email,
            fullname: user.fullname,
            active: user.active,
            admin: user.admin,
            omemo_bundle: user.omemo_bundle,
            uid_number: None,
        }
    }
}
