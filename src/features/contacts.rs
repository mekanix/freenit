use dziber::{
    models::{
        account::{AuthMode, ContactsAccount, DavOrJmap},
        contact_card::{Addressbook, ContactCard},
    },
    personal_data::contacts::ContactsClient,
};
use topcoat::{Result, context::Cx};

use crate::features::stalwart;

pub struct ContactsData {
    pub addressbooks: Vec<Addressbook>,
    pub contacts: Vec<ContactCard>,
}

pub fn client(cx: &Cx, email: &str) -> ContactsClient {
    let config = stalwart::stalwart_config(cx);
    let protocol = topcoat::context::context::<crate::config::Config>(cx)
        .contacts
        .protocol;
    ContactsClient::new(&ContactsAccount {
        id: email.to_owned(),
        server_url: config.url,
        username: email.to_owned(),
        password: config.password.clone(),
        auth_mode: AuthMode::StalwartImpersonation {
            admin_user: config.admin.trim_start_matches('%').to_owned(),
            admin_pass: config.password,
        },
        contacts_protocol: match protocol {
            crate::config::PersonalDataProtocol::Dav => DavOrJmap::Dav,
            crate::config::PersonalDataProtocol::Jmap => DavOrJmap::Jmap,
        },
    })
}

pub async fn load(cx: &Cx, email: &str) -> Result<ContactsData> {
    let client = client(cx, email);
    let addressbooks = stalwart::protocol(client.list_addressbooks().await)?;
    let mut contacts = Vec::new();
    for addressbook in &addressbooks {
        contacts.extend(stalwart::protocol(client.list_contacts(addressbook).await)?);
    }
    contacts.sort_by(|left, right| {
        left.display_name
            .to_lowercase()
            .cmp(&right.display_name.to_lowercase())
    });
    Ok(ContactsData {
        addressbooks,
        contacts,
    })
}

pub fn selected_addressbook<'a>(data: &'a ContactsData, id: Option<&str>) -> Option<&'a Addressbook> {
    id.and_then(|id| data.addressbooks.iter().find(|book| book.id == id))
}

pub fn selected_contact<'a>(data: &'a ContactsData, id: Option<&str>) -> Option<&'a ContactCard> {
    id.and_then(|id| data.contacts.iter().find(|contact| contact.id == id))
}
