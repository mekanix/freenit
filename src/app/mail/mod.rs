pub mod compose;
mod dark_mode;

use dziber::personal_data::webmail::{Email, EmailAddress};
use percent_encoding::{NON_ALPHANUMERIC, utf8_percent_encode};
use serde::Deserialize;
use std::collections::HashMap;
use topcoat::{
    Result,
    asset::{Asset, asset},
    context::Cx,
    router::{
        Body,
        content::Form,
        error::{SeeOther, bad_request, not_found, see_other},
        href, page, query_params,
        response::Response,
        route,
    },
    view::{View, view},
};

use crate::features::{auth, theme, webmail};

pub(super) const MAIL_CSS: Asset = asset!("./style.css");

#[query_params(error = bad_request)]
pub struct MailQuery {
    mailbox: Option<String>,
    email: Option<String>,
    q: Option<String>,
}

#[query_params(error = bad_request)]
struct AttachmentQuery {
    blob: String,
    name: String,
}

#[page]
pub async fn page(cx: &Cx) -> Result<impl View> {
    let user = auth::require_user(cx).await?;
    let query = query_params::<MailQuery>(cx)?;
    let client = webmail::webmail(cx, &user.email).await?;
    let mailboxes = webmail::mailboxes(&client).await?;
    let mailbox_tree = webmail::mailbox_tree(&mailboxes);
    let mailbox_id = query
        .mailbox
        .clone()
        .or_else(|| {
            mailboxes
                .iter()
                .find(|mailbox| mailbox.role.as_deref() == Some("inbox"))
                .map(|mailbox| mailbox.id.clone())
        })
        .or_else(|| mailboxes.first().map(|mailbox| mailbox.id.clone()));
    let selected_mailbox = mailbox_id
        .as_deref()
        .and_then(|id| mailboxes.iter().find(|mailbox| mailbox.id == id))
        .map(|mailbox| mailbox.name.clone());
    let mut emails = match mailbox_id.as_deref() {
        Some(id) => webmail::protocol(client.emails(id).await)?,
        None => Vec::new(),
    };
    if let Some(search) = query.q.as_deref().map(str::trim).filter(|q| !q.is_empty()) {
        let search = search.to_lowercase();
        emails.retain(|email| {
            email
                .subject
                .as_deref()
                .unwrap_or_default()
                .to_lowercase()
                .contains(&search)
                || email
                    .preview
                    .as_deref()
                    .unwrap_or_default()
                    .to_lowercase()
                    .contains(&search)
                || email.from.as_ref().is_some_and(|addresses| {
                    addresses.iter().any(|address| {
                        address.email.to_lowercase().contains(&search)
                            || address
                                .name
                                .as_deref()
                                .unwrap_or_default()
                                .to_lowercase()
                                .contains(&search)
                    })
                })
        });
    }
    let mut selected_email = match query.email.as_deref() {
        Some(id) => Some(webmail::protocol(client.email(id).await)?.ok_or_else(not_found)?),
        None => None,
    };
    if let Some(email) = selected_email.as_mut()
        && !email.keywords.contains_key("$seen")
    {
        webmail::protocol(client.mark_read(&email.id).await)?;
        email.keywords.insert("$seen".to_owned(), true);
        if let Some(list_email) = emails.iter_mut().find(|item| item.id == email.id) {
            list_email.keywords.insert("$seen".to_owned(), true);
        }
    }
    let email_count = emails.len();
    let threads = group_email_threads(emails);
    let base = href!(page).resolve(cx);
    let compose_url = href!(compose::page).resolve(cx);
    let trash_id = mailboxes
        .iter()
        .find(|mailbox| mailbox.role.as_deref() == Some("trash"))
        .map(|mailbox| mailbox.id.clone());
    let archive_id = mailboxes
        .iter()
        .find(|mailbox| mailbox.role.as_deref() == Some("archive"))
        .map(|mailbox| mailbox.id.clone());

    Ok(view! {
        <main class="mail-layout">
            <aside class="mail-sidebar">
                topcoat::runtime::link(
                    href: compose_url,
                    attrs: topcoat::view::attributes! { class="button primary" },
                    "Compose"
                )
                <nav>
                    mailbox_items(
                        tree: &mailbox_tree,
                        base: &base,
                        selected: mailbox_id.as_deref(),
                        depth: 0
                    )
                </nav>
                topcoat::runtime::link(
                    href: href!(crate::app::sieve::page),
                    "Sieve filters"
                )
            </aside>
            <section class="mail-list">
                <header class="actions">
                    <h2>(selected_mailbox.as_deref().unwrap_or("Mail"))</h2>
                    <span>(email_count)</span>
                </header>
                <form class="mail-search" method="get" action=(base.clone())>
                    if let Some(mailbox_id) = &mailbox_id {
                        <input type="hidden" name="mailbox" value=(mailbox_id)>
                    }
                    <input
                        type="search"
                        name="q"
                        value=(query.q.clone().unwrap_or_default())
                        placeholder="Search messages"
                        aria-label="Search messages"
                    >
                </form>
                if email_count == 0 {
                    <p class="empty-state">"No messages"</p>
                } else {
                    #[key(thread.id)]
                    for thread in &threads {
                        <div class="mail-thread">
                            if thread.messages.len() > 1 {
                                <input
                                    class="mail-thread-expanded"
                                    type="checkbox"
                                    id=(format!("mail-thread-{}", encode(&thread.id)))
                                    aria-label=(format!(
                                        "Toggle thread with {} messages",
                                        thread.messages.len(),
                                    ))
                                    checked=(thread
                                        .messages
                                        .iter()
                                        .any(
                                            |email| {
                                                query.email.as_deref() == Some(email.id.as_str())
                                            },
                                        ))
                                >
                            }
                            <div class="mail-thread-root">
                                if thread.messages.len() > 1 {
                                    <label
                                        class="mail-thread-toggle"
                                        for=(format!("mail-thread-{}", encode(&thread.id)))
                                    >
                                        <span>(thread.messages.len())</span>
                                    </label>
                                }
                                email_row(
                                    email: &thread.messages[0],
                                    base: &base,
                                    mailbox_id: mailbox_id.as_deref(),
                                    selected_email: query.email.as_deref(),
                                    search: query.q.as_deref()
                                )
                            </div>
                            if thread.messages.len() > 1 {
                                <div class="mail-thread-replies">
                                    #[key(email.id)]
                                    for email in thread.messages.iter().skip(1) {
                                        email_row(
                                            email: email,
                                            base: &base,
                                            mailbox_id: mailbox_id.as_deref(),
                                            selected_email: query.email.as_deref(),
                                            search: query.q.as_deref()
                                        )
                                    }
                                </div>
                            }
                        </div>
                    }
                }
            </section>
            <section class="mail-viewer">
                if let Some(email) = selected_email {
                    message_view(
                        email: &email,
                        mailbox_id: mailbox_id.as_deref(),
                        archive_id: archive_id.as_deref(),
                        has_trash: trash_id.is_some()
                    )
                } else {
                    <div class="empty-state">"Select an email to read"</div>
                }
            </section>
        </main>
    })
}

struct MailThread {
    id: String,
    messages: Vec<Email>,
}

fn group_email_threads(emails: Vec<Email>) -> Vec<MailThread> {
    let mut threads: Vec<MailThread> = Vec::new();
    let mut indexes: HashMap<String, usize> = HashMap::new();

    for email in emails {
        let thread_id = email
            .thread_id
            .as_deref()
            .filter(|thread_id| !thread_id.is_empty())
            .map(|thread_id| format!("thread:{thread_id}"))
            .unwrap_or_else(|| format!("email:{}", email.id));
        if let Some(index) = indexes.get(&thread_id).copied() {
            threads[index].messages.push(email);
        } else {
            indexes.insert(thread_id.clone(), threads.len());
            threads.push(MailThread {
                id: thread_id,
                messages: vec![email],
            });
        }
    }

    threads
}

#[topcoat::view::component]
async fn email_row(
    cx: &Cx,
    email: &Email,
    base: &str,
    mailbox_id: Option<&str>,
    selected_email: Option<&str>,
    search: Option<&str>,
) -> Result<impl View> {
    Ok(view! {
        <a
            class=(if selected_email == Some(email.id.as_str()) {
                "mail-item selected"
            } else {
                "mail-item"
            })
            (topcoat::runtime::link_attrs(
                cx,
                email_url(base, mailbox_id, &email.id, search),
                topcoat::runtime::prefetch_mode(cx),
            ))
        >
            <div class="mail-item-top">
                <strong>(email.sender())</strong>
                <time>(&email.received_at)</time>
            </div>
            <span class="mail-subject">
                (email.subject.as_deref().unwrap_or("(no subject)"))
            </span>
            <span class="mail-preview">
                (email.preview.as_deref().unwrap_or_default())
            </span>
            if email.has_attachment.unwrap_or(false) {
                <span>"Attachment"</span>
            }
            if !email.keywords.contains_key("$seen") {
                <span class="tag bg-primary text-white">"Unread"</span>
            }
        </a>
    })
}

#[topcoat::view::component]
async fn message_view(
    cx: &Cx,
    email: &Email,
    mailbox_id: Option<&str>,
    archive_id: Option<&str>,
    has_trash: bool,
) -> Result<impl View> {
    let reply = compose_action(cx, "reply", email, mailbox_id);
    let reply_all = compose_action(cx, "reply_all", email, mailbox_id);
    let forward = compose_action(cx, "forward", email, mailbox_id);
    let current_theme = theme::current(cx);
    let html_class = if current_theme == theme::Theme::Auto {
        "mail-html mail-html-auto-light"
    } else {
        "mail-html"
    };
    let html = email.body(true).map(|body| match current_theme {
        theme::Theme::Light => (email_document(body, theme::Theme::Light), None),
        theme::Theme::Dark => (
            email_document(&dark_mode::adapt(body), theme::Theme::Dark),
            None,
        ),
        theme::Theme::Auto => (
            email_document(body, theme::Theme::Auto),
            Some(email_document(
                &dark_mode::adapt(body),
                theme::Theme::Dark,
            )),
        ),
    });
    Ok(view! {
        <header class="mail-message-header">
            <h1>(email.subject.as_deref().unwrap_or("(no subject)"))</h1>
            <div class="actions">
                topcoat::runtime::link(
                    href: reply,
                    attrs: topcoat::view::attributes! { class="button outline" },
                    "Reply"
                )
                topcoat::runtime::link(
                    href: reply_all,
                    attrs: topcoat::view::attributes! { class="button outline" },
                    "Reply all"
                )
                topcoat::runtime::link(
                    href: forward,
                    attrs: topcoat::view::attributes! { class="button outline" },
                    "Forward"
                )
                if !email.keywords.contains_key("$seen") {
                    <form method="post" action=(href!(mark_read))>
                        email_fields(email: email, mailbox_id: mailbox_id)
                        <button class="button outline" type="submit">
                            "Mark read"
                        </button>
                    </form>
                }
                if archive_id.is_some() {
                    <form method="post" action=(href!(move_email))>
                        email_fields(email: email, mailbox_id: mailbox_id)
                        <input
                            type="hidden"
                            name="destination"
                            value=(archive_id.unwrap_or_default())
                        >
                        <button class="button outline" type="submit">"Archive"</button>
                    </form>
                }
                <form method="post" action=(href!(delete_email))>
                    email_fields(email: email, mailbox_id: mailbox_id)
                    <input type="hidden" name="trash" value=(has_trash.to_string())>
                    <button class="button error" type="submit">"Delete"</button>
                </form>
            </div>
            <dl class="mail-meta">
                <dt>"From"</dt>
                <dd>(format_addresses(email.from.as_deref()))</dd>
                <dt>"To"</dt>
                <dd>(format_addresses(email.to.as_deref()))</dd>
                if email.cc.as_ref().is_some_and(|addresses| !addresses.is_empty()) {
                    <dt>"Cc"</dt>
                    <dd>(format_addresses(email.cc.as_deref()))</dd>
                }
                <dt>"Date"</dt>
                <dd>(&email.received_at)</dd>
            </dl>
            if email
                .attachments
                .as_ref()
                .is_some_and(|attachments| !attachments.is_empty()) {
                <div class="attachments">
                    <strong>"Attachments"</strong>
                    for part in email.attachments.iter().flatten() {
                        if let Some(blob_id) = &part.blob_id {
                            <a
                                href=(attachment_url(
                                    cx,
                                    blob_id,
                                    part.name.as_deref().unwrap_or("attachment"),
                                ))
                            >
                                (part.name.as_deref().unwrap_or("attachment"))
                                if let Some(size) = part.size {
                                    " ("
                                    (size / 1024)
                                    " KB)"
                                }
                            </a>
                        }
                    }
                </div>
            }
        </header>
        if let Some(html) = html {
            <iframe class=(html_class) title="Email content" srcdoc=(html.0) sandbox=""></iframe>
            if let Some(dark_html) = html.1 {
                <iframe
                    class="mail-html mail-html-auto-dark"
                    title="Email content"
                    srcdoc=(dark_html)
                    sandbox=""
                ></iframe>
            }
        } else if let Some(body) = email.body(false) {
            <pre class="mail-text">(body)</pre>
        } else {
            <p class="empty-state">"No message body"</p>
        }
    })
}

#[topcoat::view::component]
async fn mailbox_items(
    cx: &Cx,
    tree: &[webmail::MailboxNode],
    base: &str,
    selected: Option<&str>,
    depth: usize,
) -> Result<impl View> {
    let items: Vec<_> = tree
        .iter()
        .map(|node| {
            (
                node,
                selected.is_some_and(|selected| mailbox_tree_contains(&node.children, selected)),
            )
        })
        .collect();

    Ok(view! {
        #[key(node.mailbox.id)]
        for (node, expanded) in items {
            <div class="mailbox-row">
                if !node.children.is_empty() {
                    <input
                        class="mailbox-expanded"
                        type="checkbox"
                        id=(format!("mailbox-expanded-{}", encode(&node.mailbox.id)))
                        aria-label=(format!("Toggle {} subfolders", node.mailbox.name))
                        checked=(expanded)
                    >
                }
                <div
                    class="mailbox-row-main"
                    style=(format!("padding-left: {}px", depth * 16))
                >
                    if !node.children.is_empty() {
                        <label
                            class="mailbox-toggle"
                            for=(format!("mailbox-expanded-{}", encode(&node.mailbox.id)))
                        >
                            <span></span>
                        </label>
                    } else {
                        <span class="mailbox-toggle-spacer"></span>
                    }
                    <a
                        class=(if Some(node.mailbox.id.as_str()) == selected {
                            "selected"
                        } else {
                            ""
                        })
                        (topcoat::runtime::link_attrs(
                            cx,
                            mailbox_url(base, &node.mailbox.id),
                            topcoat::runtime::prefetch_mode(cx),
                        ))
                    >
                        <span class="mailbox-icon">
                            (mailbox_icon(node.mailbox.role.as_deref()))
                        </span>
                        <span class="mailbox-name">(&node.mailbox.name)</span>
                        if node.mailbox.unread_emails > 0 {
                            <span class="tag bg-primary text-white">
                                (node.mailbox.unread_emails)
                            </span>
                        }
                    </a>
                </div>
                if !node.children.is_empty() {
                    <div class="mailbox-children">
                        mailbox_items(
                            tree: &node.children,
                            base: base,
                            selected: selected,
                            depth: depth + 1
                        )
                    </div>
                }
            </div>
        }
    })
}

fn mailbox_tree_contains(tree: &[webmail::MailboxNode], mailbox_id: &str) -> bool {
    tree.iter().any(|node| {
        node.mailbox.id == mailbox_id || mailbox_tree_contains(&node.children, mailbox_id)
    })
}

fn mailbox_icon(role: Option<&str>) -> &'static str {
    match role {
        Some("inbox") => "📥",
        Some("sent") => "📤",
        Some("drafts") => "📝",
        Some("junk") => "🚫",
        Some("trash") => "🗑",
        Some("archive") => "📦",
        _ => "📁",
    }
}

fn email_document(body: &str, theme: theme::Theme) -> String {
    let theme_css = match theme {
        theme::Theme::Light => {
            ":root{color-scheme:light;--mail-text:#333333;--mail-background:#ffffff;--mail-link:#2f63f0;}"
        }
        theme::Theme::Dark => {
            ":root{color-scheme:dark;--mail-text:#e0e6ed;--mail-background:#1b2433;--mail-link:#7aa2ff;}"
        }
        theme::Theme::Auto => {
            ":root{color-scheme:light dark;--mail-text:#333333;--mail-background:#ffffff;--mail-link:#2f63f0;}@media(prefers-color-scheme:dark){:root{color-scheme:dark;--mail-text:#e0e6ed;--mail-background:#1b2433;--mail-link:#7aa2ff;}}"
        }
    };
    format!(
        "<!DOCTYPE html><html><head><meta charset=\"utf-8\"><style>{theme_css}body{{margin:0;padding:1rem;color:var(--mail-text);background-color:var(--mail-background);font-family:sans-serif;line-height:1.5}}a{{color:var(--mail-link)}}img{{max-width:100%;height:auto}}</style></head><body>{body}</body></html>"
    )
}

#[topcoat::view::component]
async fn email_fields(cx: &Cx, email: &Email, mailbox_id: Option<&str>) -> Result<impl View> {
    Ok(view! {
        <input type="hidden" name="email_id" value=(&email.id)>
        <input type="hidden" name="mailbox_id" value=(mailbox_id.unwrap_or_default())>
    })
}

#[derive(Deserialize)]
struct EmailForm {
    email_id: String,
    mailbox_id: String,
    destination: Option<String>,
    #[serde(default)]
    trash: bool,
}

#[route(POST "./mark-read")]
async fn mark_read(cx: &Cx, Form(input): Form<EmailForm>) -> Result<SeeOther> {
    let user = auth::require_user(cx).await?;
    let client = webmail::webmail(cx, &user.email).await?;
    webmail::protocol(client.mark_read(&input.email_id).await)?;
    Ok(see_other(email_url(
        &href!(page).resolve(cx),
        Some(&input.mailbox_id),
        &input.email_id,
        None,
    )))
}

#[route(POST "./move")]
async fn move_email(cx: &Cx, Form(input): Form<EmailForm>) -> Result<SeeOther> {
    let user = auth::require_user(cx).await?;
    let destination = input
        .destination
        .as_deref()
        .filter(|id| !id.is_empty())
        .ok_or_else(|| bad_request("destination is required"))?;
    let client = webmail::webmail(cx, &user.email).await?;
    webmail::protocol(client.move_email(&input.email_id, destination).await)?;
    Ok(see_other(email_url(
        &href!(page).resolve(cx),
        Some(&input.mailbox_id),
        &input.email_id,
        None,
    )))
}

#[route(POST "./delete")]
async fn delete_email(cx: &Cx, Form(input): Form<EmailForm>) -> Result<SeeOther> {
    let user = auth::require_user(cx).await?;
    let client = webmail::webmail(cx, &user.email).await?;
    let trash = if input.trash {
        webmail::mailboxes(&client).await?
            .iter()
            .find(|mailbox| mailbox.role.as_deref() == Some("trash"))
            .map(|mailbox| mailbox.id.clone())
    } else {
        None
    };
    webmail::protocol(client.delete_email(&input.email_id, trash.as_deref()).await)?;
    Ok(see_other(mailbox_url(
        &href!(page).resolve(cx),
        &input.mailbox_id,
    )))
}

#[route(GET "./attachment")]
async fn attachment(cx: &Cx) -> Result<Response> {
    let user = auth::require_user(cx).await?;
    let query = query_params::<AttachmentQuery>(cx)?;
    let client = webmail::webmail(cx, &user.email).await?;
    let download = webmail::protocol(client.download(&query.blob, &query.name).await)?;
    let mut builder = Response::builder().status(download.status);
    for (name, value) in download.headers {
        builder = builder.header(name, value);
    }
    Ok(builder.body(Body::from(download.body))?)
}

pub fn mailbox_url(base: &str, mailbox_id: &str) -> String {
    format!("{base}?mailbox={}", encode(mailbox_id))
}

pub fn email_url(
    base: &str,
    mailbox_id: Option<&str>,
    email_id: &str,
    search: Option<&str>,
) -> String {
    let mut url = mailbox_url(base, mailbox_id.unwrap_or_default());
    url.push_str(&format!("&email={}", encode(email_id)));
    if let Some(search) = search.filter(|search| !search.is_empty()) {
        url.push_str(&format!("&q={}", encode(search)));
    }
    url
}

fn compose_action(cx: &Cx, action: &str, email: &Email, mailbox_id: Option<&str>) -> String {
    format!(
        "{}?{}={}&mailbox={}",
        href!(compose::page).resolve(cx),
        action,
        encode(&email.id),
        encode(mailbox_id.unwrap_or_default())
    )
}

fn attachment_url(cx: &Cx, blob_id: &str, name: &str) -> String {
    format!(
        "{}?blob={}&name={}",
        href!(attachment).resolve(cx),
        encode(blob_id),
        encode(name)
    )
}

pub fn format_addresses(addresses: Option<&[EmailAddress]>) -> String {
    addresses
        .unwrap_or_default()
        .iter()
        .map(|address| match &address.name {
            Some(name) if !name.is_empty() => format!("{name} <{}>", address.email),
            _ => address.email.clone(),
        })
        .collect::<Vec<_>>()
        .join(", ")
}

pub fn encode(value: &str) -> String {
    utf8_percent_encode(value, NON_ALPHANUMERIC).to_string()
}
