#[cfg(feature = "calendar")]
mod calendar;
#[cfg(feature = "contacts")]
mod contacts;
#[cfg(feature = "files")]
mod files;
#[cfg(feature = "jabber")]
mod chat;
mod forgot_password;
mod groups;
#[cfg(feature = "jabber")]
mod jabber;
mod login;
mod logout;
mod pagination;
#[cfg(feature = "webmail")]
mod mail;
mod profile;
#[cfg(feature = "kanban")]
mod projects;
mod register;
mod reset_password;
mod roles;
#[cfg(feature = "webmail")]
mod sieve;
mod users;
mod verify;

use crate::{
    chota,
    features::{
        auth,
        theme::{self, save_theme},
    },
};
use topcoat::{
    Result,
    asset::{Asset, AssetBundle, RouterBuilderAssetExt, asset},
    context::Cx,
    cookie::CookieLayer,
    htmx::hx_request,
    icon::{icon, iconify::iconify_icon},
    router::{
        Body, LayerFn, LayerFuture, Next, Path, RouterBuilder, RouterBuilderDiscoverExt, Slot,
        error::{
            BadRequestError, ContentTooLargeError, ForbiddenError, MethodNotAllowedError,
            NotFoundError, RedirectError, SeeOther, TooManyRequestsError, UnauthorizedError,
        },
        href, layout, module_router, page,
        request::{method, uri},
    },
    runtime::{Event, RuntimeLayer, signal},
    view::{View, view},
};

/// Builds the application's router.
pub fn router() -> RouterBuilder {
    // The pages and layouts of this module and its submodules, with paths derived
    // from the module tree.
    module_router!()
        // Fonts and other discovered integrations have no place in the module tree.
        .discover()
        .layer(CookieLayer::new())
        .layer(LayerFn::new(None::<&Path>, log_request_errors))
        .assets(AssetBundle::load().unwrap())
        .layer(RuntimeLayer::new())
        .with(topcoat::runtime::PrefetchMode::Never)
}

fn log_request_errors<'a>(cx: &'a Cx, body: Body, next: Next<'a>) -> LayerFuture<'a> {
    Box::pin(async move {
        let method = method(cx).clone();
        let uri = uri(cx).clone();
        match next.run(cx, body).await {
            Ok(response) => {
                if response.status().is_server_error() {
                    tracing::error!(%method, %uri, status = %response.status(), "request returned a server error");
                }
                Ok(response)
            }
            Err(error) => {
                if is_client_error(&error) {
                    tracing::debug!(%method, %uri, error = %error, "request rejected");
                } else {
                    tracing::error!(%method, %uri, error = %format_args!("{error:#}"), "request failed");
                }
                Err(error)
            }
        }
    })
}

fn is_client_error(error: &topcoat::Error) -> bool {
    error.is::<BadRequestError>()
        || error.is::<ContentTooLargeError>()
        || error.is::<ForbiddenError>()
        || error.is::<MethodNotAllowedError>()
        || error.is::<NotFoundError>()
        || error.is::<RedirectError>()
        || error.is::<SeeOther>()
        || error.is::<TooManyRequestsError>()
        || error.is::<UnauthorizedError>()
}

#[layout]
async fn root(cx: &Cx, slot: Slot<'_>) -> Result<impl View> {
    let user = auth::current_user(cx).await?;
    #[allow(unused_variables)]
    let path = uri(cx).path();
    #[allow(unused_mut)]
    let mut page_css: Option<Asset> = None;
    #[cfg(feature = "webmail")]
    if path == "/mail" || path.starts_with("/mail/") {
        page_css = Some(mail::MAIL_CSS);
    }
    #[cfg(feature = "webmail")]
    if path == "/sieve" || path.starts_with("/sieve/") {
        page_css = Some(sieve::SIEVE_CSS);
    }
    #[cfg(feature = "calendar")]
    if path == "/calendar" || path.starts_with("/calendar/") {
        page_css = Some(calendar::CALENDAR_CSS);
    }
    #[cfg(feature = "contacts")]
    if path == "/contacts" || path.starts_with("/contacts/") {
        page_css = Some(contacts::CONTACTS_CSS);
    }
    let drawer_open = signal(cx, || false);
    let current_theme = theme::current(cx);
    let theme_mode = signal(cx, || current_theme.as_str().to_owned());
    let _chat_assets = (
        asset!("assets/chat.js"),
        asset!("assets/dziber.js"),
        asset!("assets/dziber_bg.wasm"),
    );
    Ok(view! {
        if hx_request(cx) {
            document_head(page_css: page_css)
            notification_toast()
            (slot)
        } else {
            <!DOCTYPE html>
            <html
                lang="en"
                :data-theme=$(if theme_mode.get() == "dark" {
                    "dark"
                } else if theme_mode.get() == "light" {
                    "light"
                } else {
                    ""
                })
                :data-theme-mode=$(theme_mode.get())
            >
                document_head(page_css: page_css)
                <body
                    hx-boost="true"
                    hx-target="#app-main"
                    hx-swap="innerHTML"
                    hx-ext="head-support"
                    @keydown=$(|e: Event| {
                        if e.key == "Escape" {
                            drawer_open.set(false);
                        }
                    })
                >
                    notification_toast()
                    <aside
                        id="app-drawer"
                        :class=$(if drawer_open.get() {
                            "app-drawer open"
                        } else {
                            "app-drawer"
                        })
                        aria-label="Main navigation"
                    >
                        <div class="app-drawer-inner">
                            <nav class="app-menu" @click=$(|_e| drawer_open.set(false))>
                                if let Some(user) = user {
                                    if user.admin {
                                        topcoat::runtime::link(
                                            href: href!(users::page),
                                            attrs: topcoat::view::attributes! { class="menu-item" },
                                            icon(data: iconify_icon!("lucide:users"))
                                            "Users"
                                        )
                                        topcoat::runtime::link(
                                            href: href!(roles::page),
                                            attrs: topcoat::view::attributes! { class="menu-item" },
                                            icon(data: iconify_icon!("lucide:shield-check"))
                                            "Roles"
                                        )
                                        topcoat::runtime::link(
                                            href: href!(groups::page),
                                            attrs: topcoat::view::attributes! { class="menu-item" },
                                            icon(data: iconify_icon!("lucide:users-round"))
                                            "Groups"
                                        )
                                    }
                                    if cfg!(feature = "kanban") {
                                        topcoat::runtime::link(
                                            href: "/projects",
                                            attrs: topcoat::view::attributes! { class="menu-item" },
                                            icon(data: iconify_icon!("lucide:kanban"))
                                            "Projects"
                                        )
                                    }
                                    topcoat::runtime::link(
                                        href: href!(profile::page),
                                        attrs: topcoat::view::attributes! { class="menu-item" },
                                        icon(data: iconify_icon!("lucide:user-round"))
                                        "Profile"
                                    )
                                    if cfg!(feature = "webmail") {
                                        topcoat::runtime::link(
                                            href: "/mail",
                                            attrs: topcoat::view::attributes! { class="menu-item" },
                                            icon(data: iconify_icon!("lucide:mail"))
                                            "Mail"
                                        )
                                    }
                                    if cfg!(feature = "calendar") {
                                        topcoat::runtime::link(
                                            href: "/calendar",
                                            attrs: topcoat::view::attributes! { class="menu-item" },
                                            icon(data: iconify_icon!("lucide:calendar"))
                                            "Calendar"
                                        )
                                    }
                                    if cfg!(feature = "contacts") {
                                        topcoat::runtime::link(
                                            href: "/contacts",
                                            attrs: topcoat::view::attributes! { class="menu-item" },
                                            icon(data: iconify_icon!("lucide:contact-round"))
                                            "Contacts"
                                        )
                                    }
                                    if cfg!(feature = "files") {
                                        topcoat::runtime::link(
                                            href: href!(files::page),
                                            attrs: topcoat::view::attributes! { class="menu-item" },
                                            icon(data: iconify_icon!("lucide:folder"))
                                            "Files"
                                        )
                                    }
                                    if cfg!(feature = "webmail") {
                                        topcoat::runtime::link(
                                            href: "/sieve",
                                            attrs: topcoat::view::attributes! { class="menu-item" },
                                            icon(data: iconify_icon!("lucide:filter"))
                                            "Filters"
                                        )
                                    }
                                    if cfg!(feature = "jabber") {
                                        topcoat::runtime::link(
                                            href: "/chat",
                                            attrs: topcoat::view::attributes! { class="menu-item" },
                                            icon(data: iconify_icon!("lucide:message-circle"))
                                            "Chat"
                                        )
                                    }
                                    if user.admin && cfg!(feature = "jabber") {
                                        topcoat::runtime::link(
                                            href: "/jabber",
                                            attrs: topcoat::view::attributes! { class="menu-item" },
                                            icon(data: iconify_icon!("lucide:settings"))
                                            "Jabber"
                                        )
                                    }
                                    <form method="post" action=(href!(logout::logout))>
                                        <button class="menu-item menu-button" type="submit">
                                            icon(data: iconify_icon!("lucide:log-out"))
                                            "Logout"
                                        </button>
                                    </form>
                                } else {
                                    topcoat::runtime::link(
                                        href: href!(register::page),
                                        attrs: topcoat::view::attributes! { class="menu-item" },
                                        icon(data: iconify_icon!("lucide:user-plus"))
                                        "Register"
                                    )
                                    topcoat::runtime::link(
                                        href: href!(login::page),
                                        attrs: topcoat::view::attributes! { class="menu-item" },
                                        icon(data: iconify_icon!("lucide:log-in"))
                                        "Login"
                                    )
                                }
                            </nav>
                            <button
                                class="button clear app-drawer-close"
                                type="button"
                                aria-label="Close navigation"
                                @click=$(|_e| drawer_open.set(false))
                            >
                                "×"
                            </button>
                        </div>
                    </aside>
                    <button
                        class="app-drawer-backdrop"
                        type="button"
                        aria-label="Close navigation"
                        @click=$(|_e| drawer_open.set(false))
                    ></button>
                    <section class="app-root">
                        <header class="app-menu-bar">
                            <button
                                class="app-menu-trigger"
                                type="button"
                                aria-label="Open navigation"
                                aria-controls="app-drawer"
                                :aria-expanded=$(if drawer_open.get() {
                                    "true"
                                } else {
                                    "false"
                                })
                                @click=$(|_e| drawer_open.toggle())
                            >
                                icon(data: iconify_icon!("lucide:menu"))
                            </button>
                            topcoat::runtime::link(
                                href: href!(home),
                                attrs: topcoat::view::attributes! { class="app-title" },
                                "Freenit"
                            )
                            <button
                                class="theme-toggle"
                                type="button"
                                :title=$(if theme_mode.get() == "light" {
                                    "Light mode"
                                } else if theme_mode.get() == "dark" {
                                    "Dark mode"
                                } else {
                                    "Auto mode"
                                })
                                aria-label="Change color theme"
                                @click=$(async |_e| {
                                    let next = if theme_mode.get() == "light" {
                                        "dark".to_owned()
                                    } else if theme_mode.get() == "dark" {
                                        "auto".to_owned()
                                    } else {
                                        "light".to_owned()
                                    };
                                    let saved = save_theme(next).await;
                                    theme_mode.set(saved);
                                })
                            >
                                <span data-theme-icon="light">
                                    icon(data: iconify_icon!("lucide:sun"))
                                </span>
                                <span data-theme-icon="dark">
                                    icon(data: iconify_icon!("lucide:moon"))
                                </span>
                                <span data-theme-icon="auto">
                                    icon(data: iconify_icon!("lucide:monitor"))
                                </span>
                            </button>
                        </header>
                        <div id="app-main" class="app-main">(slot)</div>
                    </section>
                </body>
            </html>
        }
    })
}

#[topcoat::view::component]
async fn document_head(cx: &Cx, page_css: Option<Asset>) -> Result<impl View> {
    Ok(view! {
        <head>
            <meta charset="utf-8">
            <meta name="viewport" content="width=device-width, initial-scale=1">
            <meta name="description" content="Freenit services">
            <title>"Freenit"</title>
            <script
                src=(asset!(
                    "https://cdn.jsdelivr.net/npm/htmx.org@2.0.10/dist/htmx.min.js"
                ))
            ></script>
            <script
                src=(asset!("https://cdn.jsdelivr.net/npm/htmx-ext-head-support@2.0.5"))
            ></script>
            <link rel="stylesheet" href=(chota::stylesheet!())>
            <link rel="stylesheet" href=(asset!("styles.css"))>
            if let Some(page_css) = page_css {
                <link rel="stylesheet" href=(page_css) blocking="render">
            }
            topcoat::runtime::script()
            topcoat::dev::script()
        </head>
    })
}

#[topcoat::view::component]
async fn notification_toast(cx: &Cx) -> Result<impl View> {
    use topcoat::runtime::signal;

    use crate::chota::components::toast::{Notification, ToastVariant, toast};

    let notification = topcoat::context::try_context::<Notification>(cx);
    let open = signal(cx, || notification.is_some());
    let (message, variant) = notification.map_or(("", ToastVariant::Error), |notification| {
        (notification.message.as_str(), notification.variant)
    });
    Ok(view! { toast(open: &open, message: message, variant: variant) })
}

#[page]
async fn home(cx: &Cx) -> Result<impl View> {
    let _ = cx;
    Ok(view! { <main>"Home"</main> })
}
