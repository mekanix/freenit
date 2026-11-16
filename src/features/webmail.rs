use dziber::{
    models::account::{AuthMode, MailAccount, MailProtocol, ManageSieveConfig, security_from_string},
    personal_data::webmail::{Mailbox, WebmailClient},
};
use std::collections::{HashMap, HashSet};
use topcoat::{Result, context::Cx};

pub use crate::features::stalwart::protocol;
use crate::features::stalwart;

pub async fn webmail(cx: &Cx, email: &str) -> Result<WebmailClient> {
    let config = stalwart::stalwart_config(cx);
    let jmap_identity = format!("{email}%{}", config.admin.trim_start_matches('%'));
    let mut client = WebmailClient::new(&account(cx, email));
    stalwart::protocol(
        client
            .discover()
            .await
            .map_err(|error| format!("{error} (JMAP identity: {jmap_identity})")),
    )?;
    Ok(client)
}

pub async fn sieve(cx: &Cx, email: &str) -> Result<WebmailClient> {
    webmail(cx, email).await
}

pub async fn mailboxes(
    client: &WebmailClient,
) -> Result<Vec<Mailbox>> {
    protocol(client.mailboxes().await)
}

pub struct MailboxNode {
    pub mailbox: Mailbox,
    pub children: Vec<MailboxNode>,
}

pub fn mailbox_tree(mailboxes: &[Mailbox]) -> Vec<MailboxNode> {
    fn build(
        id: &str,
        mailboxes: &HashMap<String, Mailbox>,
        children: &HashMap<String, Vec<String>>,
    ) -> MailboxNode {
        let mailbox = mailboxes.get(id).expect("mailbox id exists").clone();
        let children = children
            .get(id)
            .into_iter()
            .flatten()
            .map(|child_id| build(child_id, mailboxes, children))
            .collect();
        MailboxNode { mailbox, children }
    }

    let by_id: HashMap<_, _> = mailboxes
        .iter()
        .map(|mailbox| (mailbox.id.clone(), mailbox.clone()))
        .collect();
    let ids: HashSet<_> = by_id.keys().cloned().collect();
    let mut roots = Vec::new();
    let mut children: HashMap<String, Vec<String>> = HashMap::new();

    for mailbox in mailboxes {
        if let Some(parent_id) = mailbox.parent_id.as_ref().filter(|id| ids.contains(*id)) {
            children
                .entry(parent_id.clone())
                .or_default()
                .push(mailbox.id.clone());
        } else {
            roots.push(mailbox.id.clone());
        }
    }

    roots
        .iter()
        .map(|id| build(id, &by_id, &children))
        .collect()
}

pub fn flatten_mailbox_tree(tree: &[MailboxNode]) -> Vec<(Mailbox, usize, String)> {
    fn flatten(
        nodes: &[MailboxNode],
        depth: usize,
        parent_path: &str,
        result: &mut Vec<(Mailbox, usize, String)>,
    ) {
        for node in nodes {
            let path = if parent_path.is_empty() {
                node.mailbox.name.clone()
            } else {
                format!("{parent_path}/{}", node.mailbox.name)
            };
            result.push((node.mailbox.clone(), depth, path.clone()));
            flatten(&node.children, depth + 1, &path, result);
        }
    }

    let mut result = Vec::new();
    flatten(tree, 0, "", &mut result);
    result
}

fn account(cx: &Cx, email: &str) -> MailAccount {
    let config = stalwart::stalwart_config(cx);
    let app_config = topcoat::context::context::<crate::config::Config>(cx);
    let server = config
        .url
        .split_once("://")
        .map_or(config.url.as_str(), |(_, host)| host)
        .split('/')
        .next()
        .unwrap_or(&config.url)
        .split(':')
        .next()
        .unwrap_or(&config.url)
        .to_owned();
    MailAccount {
        id: email.to_owned(),
        server_url: config.url.clone(),
        username: email.to_owned(),
        password: config.password.clone(),
        auth_mode: AuthMode::StalwartImpersonation {
            admin_user: config.admin.trim_start_matches('%').to_owned(),
            admin_pass: config.password,
        },
        mail_protocol: MailProtocol::Jmap,
        sieve_config: Some(ManageSieveConfig {
            server,
            port: app_config.sieve.port,
            security: security_from_string(Some(&app_config.sieve.security)),
        }),
    }
}
