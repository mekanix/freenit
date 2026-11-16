use dziber::{
    models::account::{AuthMode, FilesAccount},
    personal_data::files::FileServerClient,
};
use topcoat::{Result, context::Cx};

use crate::{config::Config, features::stalwart::protocol};

fn client(cx: &Cx, email: &str) -> FileServerClient {
    let config = topcoat::context::context::<Config>(cx);
    FileServerClient::new(&FilesAccount {
        id: email.to_owned(),
        server_url: config.stalwart.url.clone(),
        username: email.to_owned(),
        password: config.stalwart.password.clone(),
        auth_mode: AuthMode::StalwartImpersonation {
            admin_user: config.stalwart.admin.trim_start_matches('%').to_owned(),
            admin_pass: config.stalwart.password.clone(),
        },
    })
}

pub async fn list(cx: &Cx, email: &str, path: &str) -> Result<Vec<dziber::personal_data::dav::DavResponse>> {
    protocol(client(cx, email).list(path).await)
}

pub async fn download(cx: &Cx, email: &str, path: &str) -> std::result::Result<(u16, String, Vec<u8>), String> {
    client(cx, email).download(path).await
}

pub async fn upload(cx: &Cx, email: &str, path: &str, content_type: &str, bytes: Vec<u8>) -> Result<u16> {
    protocol(client(cx, email).upload(path, content_type, bytes).await)
}

pub async fn create_folder(cx: &Cx, email: &str, parent: &str, name: &str) -> Result<u16> {
    protocol(client(cx, email).create_folder(parent, name).await)
}

pub async fn rename_folder(cx: &Cx, email: &str, path: &str, name: &str) -> Result<u16> {
    protocol(client(cx, email).rename_folder(path, name).await)
}

pub async fn delete_folder(cx: &Cx, email: &str, path: &str) -> Result<u16> {
    protocol(client(cx, email).delete_folder(path).await)
}
