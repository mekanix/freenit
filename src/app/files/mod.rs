use percent_encoding::{NON_ALPHANUMERIC, percent_decode_str, utf8_percent_encode};
use topcoat::{Result, context::Cx, router::{Body, BodyLimit, content::{Form, multipart::Multipart}, error::{bad_request, not_found}, href, page, query_params, response::Response, route}, view::{View, view}};

use crate::features::{auth, files};

#[query_params(error = bad_request)]
struct FilesQuery { path: Option<String> }

#[page]
pub async fn page(cx: &Cx) -> Result<impl View> {
    let user = auth::require_user(cx).await?;
    let query = query_params::<FilesQuery>(cx)?;
    let path = query.path.as_deref().unwrap_or("");
    let entries = files::list(cx, &user.email, path).await?;
    let entries: Vec<_> = entries.into_iter().map(|entry| {
        let name = percent_decode_str(entry.href.trim_end_matches('/').rsplit('/').next().unwrap_or(&entry.href)).decode_utf8_lossy().into_owned();
        let directory = entry.resource_type().iter().any(|kind| kind.eq_ignore_ascii_case("collection"));
        let child_path = if path.is_empty() { name.clone() } else { format!("{path}/{name}") };
        let size = entry.prop("getcontentlength").unwrap_or("").to_owned();
        (name, directory, child_path, size)
    }).collect();
    let parent = path.rsplit_once('/').map(|(parent, _)| parent).unwrap_or("");
    Ok(view! {
        <section class="container">
            <h1>"Files"</h1>
            <p>
                "/"
                (path)
            </p>
            if !path.is_empty() {
                <p><a href=(files_url(cx, parent))>"Up one level"</a></p>
            }
            <form method="post" action=(href!(upload)) enctype="multipart/form-data">
                <input type="hidden" name="path" value=(path)>
                <input type="file" name="file" required="">
                <button class="button primary" type="submit">"Upload"</button>
            </form>
            <form method="post" action=(href!(create_folder))>
                <input type="hidden" name="parent" value=(path)>
                <input
                    type="text"
                    name="name"
                    placeholder="New folder name"
                    required=""
                >
                <button class="button" type="submit">"Create folder"</button>
            </form>
            <table>
                <thead>
                    <tr>
                        <th>"Name"</th>
                        <th>"Size"</th>
                    </tr>
                </thead>
                <tbody>
                    for (name, is_dir, child_path, size) in &entries {
                        <tr>
                            <td>
                                if *is_dir {
                                    <a href=(files_url(cx, child_path))>
                                        (name)
                                        "/"
                                    </a>
                                } else {
                                    <a href=(download_url(cx, child_path))>(name)</a>
                                }
                            </td>
                            <td>(size)</td>
                            <td>
                                if *is_dir {
                                    <details>
                                        <summary>"Manage"</summary>
                                        <form method="post" action=(href!(rename_folder))>
                                            <input type="hidden" name="path" value=(child_path)>
                                            <input type="text" name="name" value=(name) required="">
                                            <button class="button" type="submit">"Rename"</button>
                                        </form>
                                        <form method="post" action=(href!(delete_folder))>
                                            <input type="hidden" name="path" value=(child_path)>
                                            <button
                                                class="button error"
                                                type="submit"
                                                title="Deletes this folder and its contents"
                                            >
                                                "Delete folder"
                                            </button>
                                        </form>
                                    </details>
                                }
                            </td>
                        </tr>
                    }
                </tbody>
            </table>
        </section>
    })
}

#[route(POST "./upload", with(BodyLimit::max(26 * 1024 * 1024)))]
async fn upload(cx: &Cx, mut multipart: Multipart) -> Result<Response> {
    let user = auth::require_user(cx).await?;
    let mut path = String::new();
    let mut file: Option<(String, String, Vec<u8>)> = None;
    while let Some(field) = multipart.next_field().await? {
        match field.name().unwrap_or_default() {
            "path" => path = field.text().await?,
            "file" => {
                let name = field.file_name().unwrap_or("upload").to_owned();
                let content_type = field.content_type().unwrap_or("application/octet-stream").to_owned();
                file = Some((name, content_type, field.bytes().await?.to_vec()));
            }
            _ => {}
        }
    }
    let (name, content_type, bytes) = file.ok_or_else(|| bad_request("choose a file to upload"))?;
    let full_path = if path.is_empty() { name } else { format!("{path}/{name}") };
    let status = files::upload(cx, &user.email, &full_path, &content_type, bytes).await?;
    if status >= 400 { return Err(bad_request("Stalwart rejected the upload").into()); }
    Ok(Response::builder().status(303).header("Location", files_url(cx, &path)).body(Body::empty())?)
}

#[derive(serde::Deserialize)]
struct CreateFolderForm {
    parent: String,
    name: String,
}

#[route(POST "./folder/create")]
async fn create_folder(cx: &Cx, Form(input): Form<CreateFolderForm>) -> Result<Response> {
    let user = auth::require_user(cx).await?;
    let status = files::create_folder(cx, &user.email, &input.parent, &input.name).await?;
    ensure_success(status)?;
    redirect(cx, &input.parent)
}

#[derive(serde::Deserialize)]
struct RenameFolderForm {
    path: String,
    name: String,
}

#[route(POST "./folder/rename")]
async fn rename_folder(cx: &Cx, Form(input): Form<RenameFolderForm>) -> Result<Response> {
    let user = auth::require_user(cx).await?;
    let parent = input.path.rsplit_once('/').map_or("", |(parent, _)| parent);
    let status = files::rename_folder(cx, &user.email, &input.path, &input.name).await?;
    ensure_success(status)?;
    redirect(cx, parent)
}

#[derive(serde::Deserialize)]
struct DeleteFolderForm {
    path: String,
}

#[route(POST "./folder/delete")]
async fn delete_folder(cx: &Cx, Form(input): Form<DeleteFolderForm>) -> Result<Response> {
    let user = auth::require_user(cx).await?;
    let parent = input.path.rsplit_once('/').map_or("", |(parent, _)| parent);
    let status = files::delete_folder(cx, &user.email, &input.path).await?;
    ensure_success(status)?;
    redirect(cx, parent)
}

#[query_params(error = bad_request)]
struct DownloadQuery { path: String }

#[route(GET "./download")]
async fn download(cx: &Cx) -> Result<Response> {
    let user = auth::require_user(cx).await?;
    let query = query_params::<DownloadQuery>(cx)?;
    let (status, content_type, bytes) = files::download(cx, &user.email, &query.path).await.map_err(|error| {
        tracing::error!("Stalwart file download failed: {error}");
        not_found()
    })?;
    if status == 404 { return Err(not_found().into()); }
    if status >= 400 { return Err(bad_request("Stalwart rejected the download").into()); }
    Ok(Response::builder().header("Content-Type", content_type).body(Body::from(bytes))?)
}

fn files_url(cx: &Cx, path: &str) -> String {
    format!("{}?path={}", href!(page).resolve(cx), encode(path))
}
fn redirect(cx: &Cx, path: &str) -> Result<Response> {
    Ok(Response::builder()
        .status(303)
        .header("Location", files_url(cx, path))
        .body(Body::empty())?)
}
fn ensure_success(status: u16) -> Result<()> {
    if !(200..300).contains(&status) {
        return Err(bad_request(format!("Stalwart rejected the folder operation ({status})")).into());
    }
    Ok(())
}
fn download_url(cx: &Cx, path: &str) -> String {
    format!("{}?path={}", href!(download).resolve(cx), encode(path))
}
fn encode(value: &str) -> String { utf8_percent_encode(value, NON_ALPHANUMERIC).to_string() }
