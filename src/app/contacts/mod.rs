use dziber::models::contact_card::{Addressbook, ContactCard};
use dziber::personal_data::contacts::{export_vcards, import_vcards};
use percent_encoding::{NON_ALPHANUMERIC, utf8_percent_encode};
use serde::Deserialize;
use topcoat::{
    Result,
    asset::{Asset, asset},
    context::Cx,
    router::{
        Body, content::{Form, multipart::Multipart},
        error::{SeeOther, bad_request, see_other},
        href, page, query_params, route,
        response::Response,
    },
    view::{View, view},
};

use crate::features::{auth, contacts as feature, stalwart};

pub(super) const CONTACTS_CSS: Asset = asset!("./style.css");

#[query_params(error = bad_request)]
struct ContactsQuery {
    book: Option<String>,
    contact: Option<String>,
    edit: Option<String>,
    new: Option<String>,
    q: Option<String>,
}

#[page]
pub async fn page(cx: &Cx) -> Result<impl View> {
    let user = auth::require_user(cx).await?;
    let query = query_params::<ContactsQuery>(cx)?;
    let data = feature::load(cx, &user.email).await?;
    let selected_book_id = match query.book.as_deref() {
        Some("") => None,
        Some(id) => Some(id.to_owned()),
        None => data.addressbooks.first().map(|book| book.id.clone()),
    };
    let book = feature::selected_addressbook(&data, selected_book_id.as_deref()).cloned();
    let selected = feature::selected_contact(
        &data,
        query.contact.as_deref().or(query.edit.as_deref()),
    )
    .cloned();
    let is_new = query.new.is_some();
    let is_edit = is_new || query.edit.is_some();
    let search = query.q.as_deref().unwrap_or_default();
    let visible_contacts: Vec<_> = data
        .contacts
        .iter()
        .filter(|contact| book.as_ref().is_none_or(|book| contact.addressbook_id == book.id))
        .filter(|contact| {
            search.is_empty()
                || contact.display_name.to_lowercase().contains(&search.to_lowercase())
                || contact.org.to_lowercase().contains(&search.to_lowercase())
                || contact
                    .emails
                    .iter()
                    .any(|email| email.to_lowercase().contains(&search.to_lowercase()))
        })
        .cloned()
        .collect();

    let book_title = book.as_ref().map(|book| book.name.clone()).unwrap_or_else(|| "All Contacts".to_owned());
    let export_url = format!("{}?book={}", href!(export).resolve(cx), encode(selected_book_id.as_deref().unwrap_or_default()));

    Ok(view! {
        <div class="contacts-layout">
            <aside class="list-pane">
                <section class="contact-list">
                    <div class="books-section">
                        <div class="section-header">
                            <span class="section-title">"Address Books"</span>
                            <details class="create-book">
                                <summary class="icon-btn" title="New address book">
                                    "+"
                                </summary>
                                <form method="post" action=(href!(create_addressbook))>
                                    <input
                                        type="text"
                                        name="name"
                                        placeholder="Address book name"
                                        required=""
                                    >
                                    <button class="button primary" type="submit">
                                        "Create"
                                    </button>
                                </form>
                            </details>
                        </div>
                        <nav class="book-list" aria-label="Address books">
                            <a
                                class=(if selected_book_id.is_none() {
                                    "book-item active"
                                } else {
                                    "book-item"
                                })
                                href=(page_url(Some(""), None, search))
                            >
                                <span class="book-name">"All"</span>
                                <span class="book-count">(data.contacts.len())</span>
                            </a>
                            for addressbook in &data.addressbooks {
                                <div
                                    class=(if selected_book_id.as_deref()
                                        == Some(addressbook.id.as_str()) {
                                        "book-item active"
                                    } else {
                                        "book-item"
                                    })
                                >
                                    <a
                                        class="book-select"
                                        href=(page_url(Some(&addressbook.id), None, search))
                                    >
                                        <span class="book-name">(&addressbook.name)</span>
                                        <span class="book-count">
                                            (data
                                                .contacts
                                                .iter()
                                                .filter(|contact| contact.addressbook_id == addressbook.id)
                                                .count())
                                        </span>
                                    </a>
                                    <details class="book-edit">
                                        <summary title="Rename address book">"✎"</summary>
                                        <form method="post" action=(href!(rename_addressbook))>
                                            <input type="hidden" name="id" value=(&addressbook.id)>
                                            <input
                                                type="text"
                                                name="name"
                                                value=(&addressbook.name)
                                                required=""
                                                aria-label="Address book name"
                                            >
                                            <button class="button primary" type="submit">
                                                "Save"
                                            </button>
                                        </form>
                                    </details>
                                    <form
                                        class="book-delete"
                                        method="post"
                                        action=(href!(delete_addressbook))
                                    >
                                        <input type="hidden" name="id" value=(&addressbook.id)>
                                        <button type="submit" title="Delete address book">
                                            "×"
                                        </button>
                                    </form>
                                </div>
                            }
                        </nav>
                    </div>
                    <div class="contacts-section">
                        <header class="list-header">
                            <span class="list-title">
                                (format!("{} ({})", book_title, visible_contacts.len()))
                            </span>
                            <div class="header-btns">
                                if book.is_some() {
                                    <a class="export-btn" href=(export_url)>"Export"</a>
                                } else {
                                    <button class="export-btn" type="button" disabled="">
                                        "Export"
                                    </button>
                                }
                                if book.is_some() {
                                    <form
                                        id="contacts-import-form"
                                        method="post"
                                        action=(href!(import))
                                        enctype="multipart/form-data"
                                    >
                                        <input
                                            type="hidden"
                                            name="book_id"
                                            value=(book
                                                .as_ref()
                                                .map(|book| book.id.as_str())
                                                .unwrap_or_default())
                                        >
                                        <input
                                            id="contacts-import-file"
                                            class="import-file"
                                            type="file"
                                            name="file"
                                            accept=".vcf,text/vcard"
                                            required=""
                                            hidden=""
                                            @change="document.getElementById('contacts-import-form').requestSubmit()"
                                        >
                                        <button
                                            class="import-btn"
                                            type="button"
                                            @click="document.getElementById('contacts-import-file').click()"
                                        >
                                            "Import"
                                        </button>
                                    </form>
                                    <a
                                        class="new-btn"
                                        href=(new_contact_url(
                                            book.as_ref().map(|book| book.id.as_str()),
                                            search,
                                        ))
                                    >
                                        "New"
                                    </a>
                                } else {
                                    <button class="new-btn" type="button" disabled="">
                                        "New"
                                    </button>
                                }
                            </div>
                        </header>
                        <form class="search-bar" method="get" action=(href!(page))>
                            if let Some(id) = selected_book_id.as_ref() {
                                <input type="hidden" name="book" value=(id)>
                            }
                            <input
                                type="search"
                                name="q"
                                value=(search)
                                placeholder="Search contacts…"
                                aria-label="Search contacts"
                            >
                        </form>
                        if visible_contacts.is_empty() {
                            <div class="state-msg">"No contacts found."</div>
                        } else {
                            <nav class="list-items" aria-label="Contacts">
                                for contact in &visible_contacts {
                                    <a
                                        class=(if query.contact.as_deref()
                                                == Some(contact.id.as_str())
                                            || query.edit.as_deref() == Some(contact.id.as_str()) {
                                            "contact-item active"
                                        } else {
                                            "contact-item"
                                        })
                                        href=(page_url(
                                            selected_book_id.as_deref(),
                                            Some(&contact.id),
                                            search,
                                        ))
                                    >
                                        <span class="avatar">
                                            (initials(&contact.display_name))
                                        </span>
                                        <span class="info">
                                            <span class="name">(&contact.display_name)</span>
                                            if let Some(email) = contact.emails.first() {
                                                <span class="email">(email)</span>
                                            } else if !contact.org.is_empty() {
                                                <span class="email">(&contact.org)</span>
                                            }
                                        </span>
                                    </a>
                                }
                            </nav>
                        }
                    </div>
                </section>
            </aside>
            <section class="detail-pane">
                if is_edit {
                    contact_form(
                        contact: selected.clone(),
                        addressbooks: data.addressbooks.clone(),
                        book_id: book
                            .as_ref()
                            .map(|book| book.id.clone())
                            .or_else(
                                || data.addressbooks.first().map(|book| book.id.clone()),
                            ),
                        is_new: is_new
                    )
                } else if let Some(contact) = selected {
                    <div class="viewer">
                        <header class="contact-detail-header">
                            <span class="contact-avatar large">
                                (viewer_initials(&contact.display_name))
                            </span>
                            <div class="contact-title">
                                <h2>(&contact.display_name)</h2>
                                if !contact.org.is_empty() {
                                    <p>(&contact.org)</p>
                                }
                            </div>
                            <div class="contact-actions">
                                <a
                                    class="button outline"
                                    href=(edit_contact_url(
                                        selected_book_id.as_deref(),
                                        &contact.id,
                                        search,
                                    ))
                                >
                                    "Edit"
                                </a>
                                <form method="post" action=(href!(delete_contact))>
                                    <input type="hidden" name="id" value=(&contact.id)>
                                    <input
                                        type="hidden"
                                        name="book_id"
                                        value=(&contact.addressbook_id)
                                    >
                                    <button class="button error" type="submit">
                                        "Delete"
                                    </button>
                                </form>
                            </div>
                        </header>
                        <div class="contact-fields">
                            if !contact.emails.is_empty() {
                                <div class="contact-field">
                                    <strong>"Email"</strong>
                                    <div>
                                        for email in &contact.emails {
                                            <a href=(format!("mailto:{email}"))>(email)</a>
                                        }
                                    </div>
                                </div>
                            }
                            if !contact.phones.is_empty() {
                                <div class="contact-field">
                                    <strong>"Phone"</strong>
                                    <div>
                                        for phone in &contact.phones {
                                            <a href=(format!("tel:{phone}"))>(phone)</a>
                                        }
                                    </div>
                                </div>
                            }
                            if !contact.org.is_empty() {
                                <div class="contact-field">
                                    <strong>"Organization"</strong>
                                    <span>(&contact.org)</span>
                                </div>
                            }
                            if !contact.note.is_empty() {
                                <div class="contact-field">
                                    <strong>"Note"</strong>
                                    <span class="contact-note">(&contact.note)</span>
                                </div>
                            }
                            <div class="contact-field">
                                <strong>"Address book"</strong>
                                <span>
                                    (data
                                        .addressbooks
                                        .iter()
                                        .find(|book| book.id == contact.addressbook_id)
                                        .map(|book| book.name.as_str())
                                        .unwrap_or("Address book"))
                                </span>
                            </div>
                        </div>
                    </div>
                } else {
                    <div class="empty-state">
                        "Select a contact or create a new one."
                    </div>
                }
            </section>
        </div>
    })
}

#[topcoat::view::component]
async fn contact_form(
    cx: &Cx,
    contact: Option<ContactCard>,
    addressbooks: Vec<Addressbook>,
    book_id: Option<String>,
    is_new: bool,
) -> Result<impl View> {
    Ok(view! {
        <form class="contact-editor" method="post" action=(href!(save_contact))>
            <header>
                <h2>(if is_new { "New Contact" } else { "Edit Contact" })</h2>
                <div>
                    <a
                        class="button outline"
                        href=(page_url(
                            book_id.as_deref(),
                            contact.as_ref().map(|contact| contact.id.as_str()),
                            "",
                        ))
                    >
                        "Cancel"
                    </a>
                    <button class="button primary" type="submit">"Save"</button>
                </div>
            </header>
            if let Some(contact) = contact.as_ref() {
                <input type="hidden" name="id" value=(contact.id.clone())>
                <input type="hidden" name="uid" value=(contact.uid.clone())>
            }
            <div class="field-row">
                <label>
                    <span class="label">"First name"</span>
                    <input
                        type="text"
                        name="first_name"
                        value=(contact
                            .as_ref()
                            .map(|contact| contact.first_name.clone())
                            .unwrap_or_default())
                    >
                </label>
                <label>
                    <span class="label">"Last name"</span>
                    <input
                        type="text"
                        name="last_name"
                        value=(contact
                            .as_ref()
                            .map(|contact| contact.last_name.clone())
                            .unwrap_or_default())
                    >
                </label>
            </div>
            <label>
                <span class="label">"Display name"</span>
                <input
                    type="text"
                    name="display_name"
                    placeholder="Display name (auto-filled from first/last)"
                    value=(contact
                        .as_ref()
                        .map(|contact| contact.display_name.clone())
                        .unwrap_or_default())
                >
            </label>
            <label>
                <span class="label">"Email(s)"</span>
                <input
                    type="text"
                    name="emails"
                    value=(contact
                        .as_ref()
                        .map(|contact| contact.emails.join(", "))
                        .unwrap_or_default())
                    placeholder="email@example.com, other@example.com"
                >
            </label>
            <label>
                <span class="label">"Phone(s)"</span>
                <input
                    type="text"
                    name="phones"
                    value=(contact
                        .as_ref()
                        .map(|contact| contact.phones.join(", "))
                        .unwrap_or_default())
                    placeholder="+1 555 000 0000, +44 …"
                >
            </label>
            <label>
                <span class="label">"Organization"</span>
                <input
                    type="text"
                    name="org"
                    placeholder="Company or organization"
                    value=(contact
                        .as_ref()
                        .map(|contact| contact.org.clone())
                        .unwrap_or_default())
                >
            </label>
            <label>
                <span class="label">"Note"</span>
                <textarea name="note" rows="3" placeholder="Notes…">
                    (contact
                        .as_ref()
                        .map(|contact| contact.note.clone())
                        .unwrap_or_default())
                </textarea>
            </label>
            <label>
                <span class="label">"Addressbook"</span>
                <select name="book_id" disabled=(contact.is_some())>
                    for addressbook in &addressbooks {
                        <option
                            value=(&addressbook.id)
                            selected=(contact
                                    .as_ref()
                                    .map(|contact| contact.addressbook_id.as_str())
                                    .or(book_id.as_deref())
                                == Some(addressbook.id.as_str()))
                        >
                            (&addressbook.name)
                        </option>
                    }
                </select>
            </label>
            if contact.is_some() {
                <input
                    type="hidden"
                    name="book_id"
                    value=(contact
                        .as_ref()
                        .map(|contact| contact.addressbook_id.clone())
                        .unwrap_or_default())
                >
            }
        </form>
    })
}

fn initials(name: &str) -> String {
    let mut chars = name.split_whitespace().filter_map(|part| part.chars().next());
    let first = chars.next().unwrap_or(' ');
    let second = chars.next().unwrap_or_else(|| name.chars().nth(1).unwrap_or(first));
    format!("{first}{second}").to_uppercase()
}

fn viewer_initials(name: &str) -> String {
    name.chars().take(2).collect::<String>().to_uppercase()
}

fn encode(value: &str) -> String {
    utf8_percent_encode(value, NON_ALPHANUMERIC).to_string()
}

fn page_url(book: Option<&str>, contact: Option<&str>, search: &str) -> String {
    let mut params = Vec::new();
    if let Some(book) = book { params.push(format!("book={}", encode(book))); }
    if let Some(contact) = contact { params.push(format!("contact={}", encode(contact))); }
    if !search.is_empty() { params.push(format!("q={}", encode(search))); }
    if params.is_empty() { "/contacts".to_string() } else { format!("/contacts?{}", params.join("&")) }
}

fn new_contact_url(book: Option<&str>, search: &str) -> String {
    let mut url = page_url(book, None, search);
    url.push_str(if url.contains('?') { "&new=1" } else { "?new=1" });
    url
}

fn edit_contact_url(book: Option<&str>, contact: &str, search: &str) -> String {
    let mut url = page_url(book, None, search);
    url.push_str(if url.contains('?') { "&" } else { "?" });
    url.push_str("edit=");
    url.push_str(&encode(contact));
    url
}

fn redirect_to_contacts(book: Option<&str>, contact: Option<&str>) -> SeeOther {
    see_other(page_url(book, contact, ""))
}

#[derive(Deserialize)]
struct AddressbookForm {
    #[serde(default)]
    id: String,
    name: String,
}

#[route(POST "./create-addressbook")]
async fn create_addressbook(cx: &Cx, Form(input): Form<AddressbookForm>) -> Result<SeeOther> {
    let user = auth::require_user(cx).await?;
    let name = input.name.trim();
    if name.is_empty() { return Err(bad_request("address book name is required").into()); }
    let id = stalwart::protocol(feature::client(cx, &user.email).create_addressbook(name).await)?;
    Ok(redirect_to_contacts(Some(&id), None))
}

#[route(POST "./rename-addressbook")]
async fn rename_addressbook(cx: &Cx, Form(input): Form<AddressbookForm>) -> Result<SeeOther> {
    let user = auth::require_user(cx).await?;
    let data = feature::load(cx, &user.email).await?;
    let book = feature::selected_addressbook(&data, Some(&input.id)).ok_or_else(|| bad_request("address book not found"))?;
    stalwart::protocol(feature::client(cx, &user.email).rename_addressbook(book, input.name.trim()).await)?;
    Ok(redirect_to_contacts(Some(&input.id), None))
}

#[route(POST "./delete-addressbook")]
async fn delete_addressbook(cx: &Cx, Form(input): Form<AddressbookForm>) -> Result<SeeOther> {
    let user = auth::require_user(cx).await?;
    let data = feature::load(cx, &user.email).await?;
    let book = feature::selected_addressbook(&data, Some(&input.id)).ok_or_else(|| bad_request("address book not found"))?;
    stalwart::protocol(feature::client(cx, &user.email).delete_addressbook(book).await)?;
    Ok(redirect_to_contacts(None, None))
}

#[derive(Deserialize)]
struct ContactForm {
    #[serde(default)] id: String,
    #[serde(default)] uid: String,
    book_id: String,
    #[serde(default)] first_name: String,
    #[serde(default)] last_name: String,
    display_name: String,
    #[serde(default)] emails: String,
    #[serde(default)] phones: String,
    #[serde(default)] org: String,
    #[serde(default)] note: String,
}

#[route(POST "./save-contact")]
async fn save_contact(cx: &Cx, Form(input): Form<ContactForm>) -> Result<SeeOther> {
    let user = auth::require_user(cx).await?;
    let data = feature::load(cx, &user.email).await?;
    if feature::selected_addressbook(&data, Some(&input.book_id)).is_none() {
        return Err(bad_request("address book not found").into());
    }
    let first_name = input.first_name.trim().to_owned();
    let last_name = input.last_name.trim().to_owned();
    let display_name = input.display_name.trim().to_owned();
    let display_name = if display_name.is_empty() {
        format!("{first_name} {last_name}").trim().to_owned()
    } else {
        display_name
    };
    if display_name.is_empty() { return Err(bad_request("contact name is required").into()); }
    let values = |raw: String| raw.lines().flat_map(|line| line.split(',')).map(str::trim).filter(|value| !value.is_empty()).map(str::to_owned).collect();
    let contact = ContactCard {
        id: input.id,
        account_id: user.email.clone(),
        addressbook_id: input.book_id.clone(),
        href: String::new(),
        etag: None,
        uid: if input.uid.is_empty() { uuid::Uuid::new_v4().to_string() } else { input.uid },
        display_name,
        first_name,
        last_name,
        emails: values(input.emails),
        phones: values(input.phones),
        org: input.org.trim().to_owned(),
        note: input.note.trim().to_owned(),
        raw_vcard: String::new(),
    };
    let uid = contact.uid.clone();
    stalwart::protocol(feature::client(cx, &user.email).save_contact(&contact).await)?;
    let updated = feature::load(cx, &user.email).await?;
    let saved_id = updated
        .contacts
        .iter()
        .find(|saved| saved.uid == uid)
        .map(|saved| saved.id.clone());
    Ok(redirect_to_contacts(Some(&input.book_id), saved_id.as_deref()))
}

#[derive(Deserialize)]
struct DeleteContactForm { id: String, book_id: String }

#[route(POST "./delete-contact")]
async fn delete_contact(cx: &Cx, Form(input): Form<DeleteContactForm>) -> Result<SeeOther> {
    let user = auth::require_user(cx).await?;
    let data = feature::load(cx, &user.email).await?;
    let contact = feature::selected_contact(&data, Some(&input.id)).ok_or_else(|| bad_request("contact not found"))?;
    stalwart::protocol(feature::client(cx, &user.email).delete_contact(contact).await)?;
    Ok(redirect_to_contacts(Some(&input.book_id), None))
}

#[route(POST "./import")]
async fn import(cx: &Cx, mut multipart: Multipart) -> Result<SeeOther> {
    let user = auth::require_user(cx).await?;
    let mut book_id = String::new();
    let mut file = None;
    while let Some(field) = multipart.next_field().await? {
        let field_name = field.name().unwrap_or_default().to_owned();
        match field_name.as_str() {
            "book_id" => book_id = field.text().await?,
            "file" => file = Some(field.bytes().await?.to_vec()),
            _ => {}
        }
    }
    let data = feature::load(cx, &user.email).await?;
    let book = feature::selected_addressbook(&data, Some(&book_id))
        .ok_or_else(|| bad_request("address book not found"))?;
    let bytes = file.ok_or_else(|| bad_request("choose a vCard file"))?;
    let text = String::from_utf8(bytes).map_err(|_| bad_request("vCard file must be UTF-8"))?;
    let imported = import_vcards(&text, &user.email, &book.id);
    let client = feature::client(cx, &user.email);
    for mut contact in imported {
        contact.id.clear();
        stalwart::protocol(client.save_contact(&contact).await)?;
    }
    Ok(redirect_to_contacts(Some(&book_id), None))
}

#[query_params(error = bad_request)]
struct ExportQuery {
    book: String,
}

#[route(GET "./export")]
async fn export(cx: &Cx) -> Result<Response> {
    let user = auth::require_user(cx).await?;
    let query = query_params::<ExportQuery>(cx)?;
    let data = feature::load(cx, &user.email).await?;
    let book = feature::selected_addressbook(&data, Some(&query.book))
        .ok_or_else(|| bad_request("address book not found"))?;
    let contacts: Vec<_> = data
        .contacts
        .iter()
        .filter(|contact| contact.addressbook_id == book.id)
        .cloned()
        .collect();
    let body = export_vcards(&contacts);
    Ok(Response::builder()
        .header("Content-Type", "text/vcard; charset=utf-8")
        .header("Content-Disposition", "attachment; filename=contacts.vcf")
        .body(Body::from(body))?)
}
