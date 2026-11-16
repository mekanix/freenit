use std::collections::HashMap;

use dziber::personal_data::webmail::{Email, EmailAddress, OutgoingAttachment, OutgoingEmail};
use topcoat::{
    Result,
    context::Cx,
    router::{
        content::multipart::Multipart,
        error::{SeeOther, bad_request, see_other},
        href, page, query_params, route,
    },
    view::{View, view},
};

use crate::features::{auth, webmail};

use super::format_addresses;

#[query_params(error = bad_request)]
struct ComposeQuery {
    reply: Option<String>,
    reply_all: Option<String>,
    forward: Option<String>,
    mailbox: Option<String>,
}

#[page]
pub async fn page(cx: &Cx) -> Result<impl View> {
    let user = auth::require_user(cx).await?;
    let query = query_params::<ComposeQuery>(cx)?;
    let client = webmail::webmail(cx, &user.email).await?;
    let identities = webmail::protocol(client.identities().await)?;
    let email_id = query
        .reply
        .as_ref()
        .or(query.reply_all.as_ref())
        .or(query.forward.as_ref());
    let original = match email_id {
        Some(id) => webmail::protocol(client.email(id).await)?,
        None => None,
    };
    let original = original.as_ref();
    let reply_all = query.reply_all.is_some();
    let forward = query.forward.is_some();
    let my_address = identities
        .first()
        .map(|identity| identity.email.as_str())
        .unwrap_or(&user.email);
    let to = original.map_or_else(String::new, |email| {
        if forward {
            return String::new();
        }
        let mut addresses = email
            .reply_to
            .as_ref()
            .filter(|addresses| !addresses.is_empty())
            .or(email.from.as_ref())
            .cloned()
            .unwrap_or_default();
        if reply_all {
            addresses.extend(
                email
                    .to
                    .iter()
                    .flatten()
                    .filter(|address| address.email != my_address)
                    .cloned(),
            );
        }
        format_addresses(Some(&addresses))
    });
    let cc = if reply_all {
        original.map_or_else(String::new, |email| {
            let addresses: Vec<_> = email
                .cc
                .iter()
                .flatten()
                .filter(|address| address.email != my_address)
                .cloned()
                .collect();
            format_addresses(Some(&addresses))
        })
    } else {
        String::new()
    };
    let subject = original.map_or_else(String::new, |email| {
        let subject = email.subject.as_deref().unwrap_or_default();
        let prefix = if forward { "Fwd:" } else { "Re:" };
        if subject.starts_with(prefix) {
            subject.to_owned()
        } else {
            format!("{prefix} {subject}")
        }
    });
    let body = original.map_or_else(String::new, quote_email);
    let in_reply_to = if forward {
        Vec::new()
    } else {
        email_id.cloned().into_iter().collect()
    };
    let mut references = original
        .and_then(|email| email.references.clone())
        .unwrap_or_default();
    if !forward && let Some(id) = email_id {
        references.push(id.clone());
    }

    Ok(view! {
        <main class="mail-compose">
            <header class="compose-toolbar">
                <h1>"New Message"</h1>
                <div class="actions">
                    <button
                        class="button primary"
                        type="submit"
                        form="compose-form"
                        disabled=(identities.is_empty())
                    >
                        "Send"
                    </button>
                    topcoat::runtime::link(
                        href: href!(super::page),
                        attrs: topcoat::view::attributes! { class="button outline" },
                        "Discard"
                    )
                </div>
            </header>
            <form
                id="compose-form"
                method="post"
                action=(href!(send))
                enctype="multipart/form-data"
                class="compose-form"
            >
                <div class="compose-fields">
                    <label class="compose-field">
                        <span>"From"</span>
                        <select name="identity_id" required="">
                            for identity in &identities {
                                <option value=(&identity.id)>
                                    (&identity.name)
                                    " <"
                                    (&identity.email)
                                    ">"
                                </option>
                            }
                        </select>
                    </label>
                    <label class="compose-field">
                        <span>"To"</span>
                        <input
                            type="text"
                            name="to"
                            value=(to)
                            placeholder="Recipient(s)…"
                            required=""
                        >
                    </label>
                    <label class="compose-field">
                        <span>"Cc"</span>
                        <input
                            type="text"
                            name="cc"
                            value=(cc)
                            placeholder="CC recipients…"
                        >
                    </label>
                    <label class="compose-field">
                        <span>"Bcc"</span>
                        <input type="text" name="bcc" placeholder="BCC recipients…">
                    </label>
                    <label class="compose-field">
                        <span>"Subject"</span>
                        <input
                            type="text"
                            name="subject"
                            value=(subject)
                            placeholder="Subject…"
                        >
                    </label>
                </div>
                <input type="hidden" name="in_reply_to" value=(in_reply_to.join(" "))>
                <input type="hidden" name="references" value=(references.join(" "))>
                <input
                    type="hidden"
                    name="mailbox"
                    value=(query.mailbox.clone().unwrap_or_default())
                >
                <label class="compose-attachments">
                    <span>"Attachments"</span>
                    <input type="file" name="attachments" multiple="">
                </label>
                <textarea
                    class="compose-body"
                    name="body"
                    placeholder="Write your message…"
                >
                    (body)
                </textarea>
            </form>
        </main>
    })
}

#[route(POST)]
async fn send(cx: &Cx, mut multipart: Multipart) -> Result<SeeOther> {
    let user = auth::require_user(cx).await?;
    let mut fields = HashMap::new();
    let mut attachments = Vec::new();
    while let Some(field) = multipart.next_field().await? {
        let name = field.name().unwrap_or_default().to_owned();
        if name == "attachments" {
            let file_name = field.file_name().unwrap_or_default().to_owned();
            let content_type = field
                .content_type()
                .unwrap_or("application/octet-stream")
                .to_owned();
            let data = field.bytes().await?.to_vec();
            if !file_name.is_empty() && !data.is_empty() {
                attachments.push(OutgoingAttachment {
                    name: file_name,
                    content_type,
                    data,
                });
            }
        } else {
            fields.insert(name, field.text().await?);
        }
    }
    let client = webmail::webmail(cx, &user.email).await?;
    let identities = webmail::protocol(client.identities().await)?;
    let identity_id = required(&fields, "identity_id")?;
    let identity = identities
        .iter()
        .find(|identity| identity.id == identity_id)
        .ok_or_else(|| bad_request("unknown sender identity"))?;
    let to = parse_addresses(&required(&fields, "to")?)?;
    if to.is_empty() {
        return Err(bad_request("at least one recipient is required").into());
    }
    let outgoing = OutgoingEmail {
        identity_id,
        from: EmailAddress {
            name: Some(identity.name.clone()),
            email: identity.email.clone(),
        },
        to,
        cc: parse_addresses(fields.get("cc").map_or("", String::as_str))?,
        bcc: parse_addresses(fields.get("bcc").map_or("", String::as_str))?,
        subject: fields.get("subject").cloned().unwrap_or_default(),
        text_body: fields.get("body").cloned().unwrap_or_default(),
        in_reply_to: fields
            .get("in_reply_to")
            .map(|value| value.split_whitespace().map(str::to_owned).collect())
            .unwrap_or_default(),
        references: fields
            .get("references")
            .map(|value| value.split_whitespace().map(str::to_owned).collect())
            .unwrap_or_default(),
        attachments,
    };
    webmail::protocol(client.send(outgoing).await)?;
    let base = href!(super::page).resolve(cx);
    let destination = fields
        .get("mailbox")
        .filter(|mailbox| !mailbox.is_empty())
        .map_or(base.clone(), |mailbox| super::mailbox_url(&base, mailbox));
    Ok(see_other(destination))
}

fn required(fields: &HashMap<String, String>, name: &str) -> Result<String> {
    fields
        .get(name)
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .ok_or_else(|| bad_request(format!("{name} is required")).into())
}

fn parse_addresses(raw: &str) -> Result<Vec<EmailAddress>> {
    raw.split([',', ';'])
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| {
            if let Some((name, email)) = value.rsplit_once('<') {
                let email = email
                    .strip_suffix('>')
                    .ok_or_else(|| bad_request("invalid recipient address"))?
                    .trim();
                validate_address(email)?;
                Ok(EmailAddress {
                    name: Some(name.trim().trim_matches('"').to_owned()),
                    email: email.to_owned(),
                })
            } else {
                validate_address(value)?;
                Ok(EmailAddress {
                    name: None,
                    email: value.to_owned(),
                })
            }
        })
        .collect()
}

fn validate_address(address: &str) -> Result<()> {
    let valid = address.split_once('@').is_some_and(|(local, domain)| {
        !local.is_empty() && !domain.is_empty() && !address.chars().any(char::is_whitespace)
    });
    if valid {
        Ok(())
    } else {
        Err(bad_request(format!("invalid email address: {address}")).into())
    }
}

fn quote_email(email: &Email) -> String {
    let from = format_addresses(email.from.as_deref());
    let body = email.body(false).unwrap_or_default();
    let quoted = body
        .lines()
        .map(|line| format!("> {line}"))
        .collect::<Vec<_>>()
        .join("\n");
    format!("\n\n---\nOn {}, {from} wrote:\n{quoted}", email.received_at)
}
