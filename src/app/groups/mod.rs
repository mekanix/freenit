pub mod group_id;

use serde::Deserialize;
use topcoat::{
    Result,
    context::Cx,
    router::{
        content::Form,
        error::{SeeOther, see_other},
        href, page, route,
    },
    view::{View, view},
};

use crate::{
    app::pagination::{Pagination, pagination},
    features::{auth, group::Group, user::UserId},
};

#[page]
pub async fn page(cx: &Cx) -> Result<impl View> {
    let admin = auth::require_admin(cx).await?;
    let groups = Group::list(cx).await?;
    let pagination = Pagination::new(cx, groups.len())?;
    let groups = pagination.slice(groups);
    let base = href!(page).resolve(cx);
    let domain = admin
        .email
        .split_once('@')
        .map_or_else(|| "".to_owned(), |(_, domain)| domain.to_owned());

    Ok(view! {
        <main class="container">
            <div class="management-header">
                <h2>"Groups"</h2>
                <button
                    class="button primary"
                    type="button"
                    popovertarget="create-group"
                >
                    "Create"
                </button>
            </div>
            if groups.is_empty() {
                <p>"No groups yet."</p>
            } else {
                <div class="management-table groups-table">
                    <div class="heading">"Name"</div>
                    for group in groups {
                        <div class="group-row">
                            topcoat::runtime::link(
                                href: href!(group_id::page, group_id: &group.id),
                                (&group.name)
                            )
                            <form
                                method="post"
                                action=(href!(group_id::delete, group_id: &group.id))
                            >
                                <button class="button error" type="submit">
                                    "Destroy"
                                </button>
                            </form>
                        </div>
                    }
                </div>
            }
        </main>
        pagination(base: &base, page: pagination.page, pages: pagination.pages)
        <div
            id="create-group"
            class="management-modal"
            popover="auto"
            role="dialog"
            aria-modal="true"
            aria-labelledby="create-group-title"
        >
            <h2 id="create-group-title">"Create"</h2>
            <form class="form-grid" method="post" action=(href!(create))>
                <input type="hidden" name="user_id" value=(admin.id.to_string())>
                <input type="hidden" name="domain" value=(domain)>
                <label>
                    "Name"
                    <input type="text" name="name" required="">
                </label>
                <div class="actions">
                    <button class="button primary" type="submit">"Create"</button>
                    <button
                        class="button error"
                        type="button"
                        popovertarget="create-group"
                        popovertargetaction="hide"
                    >
                        "Close"
                    </button>
                </div>
            </form>
        </div>
    })
}

#[derive(Deserialize)]
pub struct NewGroup {
    name: String,
    #[serde(default)]
    domain: String,
    user_id: UserId,
}

#[route(POST)]
pub async fn create(cx: &Cx, Form(input): Form<NewGroup>) -> Result<SeeOther> {
    auth::require_admin(cx).await?;
    Group::add(cx, &input.name, &input.domain, &input.user_id).await?;
    Ok(see_other(href!(page).resolve(cx)))
}
