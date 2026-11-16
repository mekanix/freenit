use std::collections::{BTreeSet, HashMap};

use percent_encoding::{NON_ALPHANUMERIC, utf8_percent_encode};
use serde::Deserialize;
use topcoat::{
    Result,
    asset::{Asset, asset},
    context::Cx,
    router::{
        content::Form,
        error::{SeeOther, bad_request, see_other},
        href, page, query_params, route,
    },
    view::{View, view},
};

use crate::features::{auth, webmail};

pub(super) const SIEVE_CSS: Asset = asset!("./style.css");

#[query_params(error = bad_request)]
struct SieveQuery {
    name: Option<String>,
    new: Option<String>,
    add_rule: Option<String>,
}

#[query_params(error = bad_request)]
struct RuleMoveQuery {
    move_rule_up: Option<usize>,
    move_rule_down: Option<usize>,
}

struct ScriptDetail {
    name: String,
    content: String,
    active: bool,
}

#[derive(Clone)]
struct SieveRuleView {
    match_all: bool,
    conditions: Vec<SieveConditionView>,
    action: String,
    action_value: String,
    action_subject: String,
    action_days: String,
    stop: bool,
}

#[derive(Clone)]
struct SieveConditionView {
    field: String,
    operator: String,
    value: String,
}

impl SieveRuleView {
    fn empty() -> Self {
        Self {
            match_all: false,
            conditions: Vec::new(),
            action: "fileinto".to_owned(),
            action_value: String::new(),
            action_subject: String::new(),
            action_days: "7".to_owned(),
            stop: false,
        }
    }
}

impl SieveConditionView {
    fn empty() -> Self {
        Self {
            field: "from".to_owned(),
            operator: "contains".to_owned(),
            value: String::new(),
        }
    }
}

fn parse_sieve_rules(script: &str) -> Option<Vec<SieveRuleView>> {
    let lines: Vec<_> = script.lines().collect();
    let mut rules = Vec::new();
    let mut index = 0;

    while index < lines.len() {
        let line = lines[index].trim();
        if line.is_empty()
            || line.starts_with('#')
            || line.starts_with("require ")
            || line.eq_ignore_ascii_case("keep;")
        {
            index += 1;
            continue;
        }

        let test = line.strip_prefix("if ")?.trim().strip_suffix('{')?.trim();
        let (match_all, condition_source) = if let Some(inner) = test
            .strip_prefix("allof(")
            .and_then(|test| test.strip_suffix(')'))
        {
            (true, inner)
        } else if let Some(inner) = test
            .strip_prefix("anyof(")
            .and_then(|test| test.strip_suffix(')'))
        {
            (false, inner)
        } else {
            (false, test)
        };

        let conditions = split_sieve_list(condition_source)
            .into_iter()
            .map(parse_sieve_condition)
            .collect::<Option<Vec<_>>>()?;
        if conditions.is_empty() {
            return None;
        }

        index += 1;
        let mut action = None;
        let mut stop = false;
        let mut closed = false;
        while index < lines.len() {
            let line = lines[index].trim();
            index += 1;
            if line == "}" {
                closed = true;
                break;
            }
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if line.eq_ignore_ascii_case("stop;") {
                stop = true;
            } else {
                action = Some(parse_sieve_action(line)?);
            }
        }
        if !closed {
            return None;
        }

        let action = action.unwrap_or_else(SieveActionView::keep);
        rules.push(SieveRuleView {
            match_all,
            conditions,
            action: action.kind,
            action_value: action.value,
            action_subject: action.subject,
            action_days: action.days,
            stop,
        });
    }

    Some(rules)
}

fn split_sieve_list(source: &str) -> Vec<&str> {
    let mut values = Vec::new();
    let mut start = 0;
    let mut depth = 0usize;
    let mut quoted = false;
    let mut escaped = false;

    for (index, character) in source.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        match character {
            '\\' if quoted => escaped = true,
            '"' => quoted = !quoted,
            '(' if !quoted => depth += 1,
            ')' if !quoted => depth = depth.saturating_sub(1),
            ',' if !quoted && depth == 0 => {
                values.push(source[start..index].trim());
                start = index + character.len_utf8();
            }
            _ => {}
        }
    }
    values.push(source[start..].trim());
    values
}

fn parse_sieve_condition(source: &str) -> Option<SieveConditionView> {
    let source = source.trim();
    let mut rest = source
        .get(..6)?
        .eq_ignore_ascii_case("header")
        .then(|| &source[6..])?
        .trim_start();
    let operator_end = rest.find(char::is_whitespace)?;
    let operator = match &rest[..operator_end] {
        ":is" => "is",
        ":contains" => "contains",
        ":matches" => "matches",
        _ => return None,
    };
    rest = rest[operator_end..].trim_start();
    let (field, remaining) = parse_sieve_string(rest)?;
    let (value, remaining) = parse_sieve_string(remaining)?;
    if !remaining.trim().is_empty() {
        return None;
    }
    let field = match field.to_ascii_lowercase().as_str() {
        "from" => "from",
        "to" => "to",
        "subject" => "subject",
        "cc" => "cc",
        "list-id" => "list-id",
        "x-spam-flag" => "x-spam-flag",
        "x-priority" => "x-priority",
        _ => return None,
    };
    Some(SieveConditionView {
        field: field.to_owned(),
        operator: operator.to_owned(),
        value,
    })
}

fn parse_sieve_string(source: &str) -> Option<(String, &str)> {
    let source = source.trim_start();
    let mut characters = source.char_indices();
    if characters.next()?.1 != '"' {
        return None;
    }

    let mut value = String::new();
    let mut escaped = false;
    for (index, character) in characters {
        if escaped {
            value.push(if character == 'n' { '\n' } else { character });
            escaped = false;
        } else if character == '\\' {
            escaped = true;
        } else if character == '"' {
            return Some((value, &source[index + character.len_utf8()..]));
        } else {
            value.push(character);
        }
    }
    None
}

struct SieveActionView {
    kind: String,
    value: String,
    subject: String,
    days: String,
}

impl SieveActionView {
    fn new(kind: &str, value: String) -> Self {
        Self {
            kind: kind.to_owned(),
            value,
            subject: String::new(),
            days: "7".to_owned(),
        }
    }

    fn keep() -> Self {
        Self::new("keep", String::new())
    }
}

fn parse_sieve_action(source: &str) -> Option<SieveActionView> {
    let source = source.trim().strip_suffix(';')?.trim();
    if source.eq_ignore_ascii_case("keep") {
        return Some(SieveActionView::keep());
    }
    if source.eq_ignore_ascii_case("discard") {
        return Some(SieveActionView::new("discard", String::new()));
    }
    if let Some(value) = source.strip_prefix("fileinto ") {
        let (folder, rest) = parse_sieve_string(value)?;
        return rest
            .trim()
            .is_empty()
            .then(|| SieveActionView::new("fileinto", folder));
    }
    if let Some(value) = source.strip_prefix("redirect ") {
        let (address, rest) = parse_sieve_string(value)?;
        return rest
            .trim()
            .is_empty()
            .then(|| SieveActionView::new("redirect", address));
    }
    if let Some(value) = source.strip_prefix("addflag ") {
        let (flag, rest) = parse_sieve_string(value)?;
        if rest.trim().is_empty() && flag.eq_ignore_ascii_case("\\Seen") {
            return Some(SieveActionView::new("mark-read", String::new()));
        }
    }
    if let Some(mut rest) = source.strip_prefix("vacation ") {
        let mut action = SieveActionView::new("vacation", String::new());
        if let Some(days) = rest.strip_prefix(":days ") {
            let end = days.find(char::is_whitespace).unwrap_or(days.len());
            action.days = days[..end].to_owned();
            rest = days[end..].trim_start();
        }
        if let Some(subject) = rest.strip_prefix(":subject ") {
            (action.subject, rest) = parse_sieve_string(subject)?;
        }
        let (message, remaining) = parse_sieve_string(rest)?;
        if remaining.trim().is_empty() {
            action.value = message;
            return Some(action);
        }
    }
    None
}

fn generate_sieve(
    fields: &HashMap<String, String>,
    move_up: Option<usize>,
    move_down: Option<usize>,
) -> Result<String, String> {
    let rule_count = field_number(fields, "rule_count")?;
    if rule_count > 256 {
        return Err("too many Sieve rules".to_owned());
    }
    if move_up.is_some() && move_down.is_some() {
        return Err("cannot move a rule up and down at the same time".to_owned());
    }
    let mut rule_order: Vec<_> = (0..rule_count).collect();
    if let Some(index) = move_up {
        if index >= rule_count {
            return Err("invalid rule index".to_owned());
        }
        if index > 0 {
            rule_order.swap(index - 1, index);
        }
    }
    if let Some(index) = move_down {
        if index >= rule_count {
            return Err("invalid rule index".to_owned());
        }
        if index + 1 < rule_count {
            rule_order.swap(index, index + 1);
        }
    }

    let mut required = BTreeSet::new();
    let mut blocks = Vec::new();
    for rule_index in rule_order {
        if fields.contains_key(&format!("rule_{rule_index}_remove")) {
            continue;
        }
        let condition_count = condition_count(fields, rule_index)?;
        if condition_count > 256 {
            return Err("too many Sieve conditions".to_owned());
        }
        let mut conditions = Vec::new();
        for condition_index in 0..condition_count {
            let prefix = format!("rule_{rule_index}_condition_{condition_index}");
            if fields.contains_key(&format!("{prefix}_remove")) {
                continue;
            }
            let field = fields
                .get(&format!("{prefix}_field"))
                .map(String::as_str)
                .unwrap_or_default();
            let operator = fields
                .get(&format!("{prefix}_operator"))
                .map(String::as_str)
                .unwrap_or_default();
            let value = fields
                .get(&format!("{prefix}_value"))
                .map(String::as_str)
                .unwrap_or_default();
            if value.is_empty() {
                continue;
            }
            if ![
                "from",
                "to",
                "subject",
                "cc",
                "list-id",
                "x-spam-flag",
                "x-priority",
            ]
            .contains(&field)
                || !["is", "contains", "matches"].contains(&operator)
            {
                return Err("invalid Sieve condition".to_owned());
            }
            let operator = match operator {
                "is" => ":is",
                "matches" => ":matches",
                _ => ":contains",
            };
            conditions.push(format!(
                "header {operator} {} {}",
                sieve_quote(field),
                sieve_quote(value)
            ));
        }
        if conditions.is_empty() {
            continue;
        }

        let test = if conditions.len() == 1 {
            conditions.pop().unwrap_or_default()
        } else {
            let matcher = if fields
                .get(&format!("rule_{rule_index}_match"))
                .is_some_and(|value| value == "all")
            {
                "allof"
            } else {
                "anyof"
            };
            format!("{matcher}({})", conditions.join(", "))
        };

        let action = fields
            .get(&format!("rule_{rule_index}_action"))
            .map(String::as_str)
            .unwrap_or("keep");
        let value = fields
            .get(&format!("rule_{rule_index}_action_value"))
            .map(String::as_str)
            .unwrap_or_default();
        let subject = fields
            .get(&format!("rule_{rule_index}_action_subject"))
            .map(String::as_str)
            .unwrap_or_default();
        let days = fields
            .get(&format!("rule_{rule_index}_action_days"))
            .and_then(|value| value.parse::<u32>().ok())
            .filter(|days| *days > 0)
            .unwrap_or(7);
        let action_line = match action {
            "keep" => "keep;".to_owned(),
            "discard" => "discard;".to_owned(),
            "fileinto" => {
                required.insert("fileinto");
                format!("fileinto {};", sieve_quote(value))
            }
            "redirect" => format!("redirect {};", sieve_quote(value)),
            "vacation" => {
                required.insert("vacation");
                if subject.is_empty() {
                    format!("vacation :days {days} {};", sieve_quote(value))
                } else {
                    format!(
                        "vacation :days {days} :subject {} {};",
                        sieve_quote(subject),
                        sieve_quote(value)
                    )
                }
            }
            "mark-read" => {
                required.insert("imap4flags");
                "addflag \"\\\\Seen\";".to_owned()
            }
            _ => return Err("invalid Sieve action".to_owned()),
        };
        let mut block = format!("if {test} {{\n    {action_line}\n");
        if fields.contains_key(&format!("rule_{rule_index}_stop")) {
            block.push_str("    stop;\n");
        }
        block.push('}');
        blocks.push(block);
    }

    if blocks.is_empty() {
        return Ok("keep;\n".to_owned());
    }
    let mut script = String::new();
    if !required.is_empty() {
        let required = required
            .iter()
            .map(|extension| sieve_quote(extension))
            .collect::<Vec<_>>()
            .join(", ");
        script.push_str(&format!("require [{required}];\n\n"));
    }
    script.push_str(&blocks.join("\n"));
    script.push('\n');
    Ok(script)
}

fn field_number(fields: &HashMap<String, String>, name: &str) -> Result<usize, String> {
    fields
        .get(name)
        .ok_or_else(|| format!("missing Sieve form field {name}"))?
        .parse::<usize>()
        .map_err(|_| format!("invalid Sieve form field {name}"))
}

fn condition_count(fields: &HashMap<String, String>, rule_index: usize) -> Result<usize, String> {
    let name = format!("rule_{rule_index}_condition_count");
    if fields.contains_key(&name) {
        return field_number(fields, &name);
    }

    let prefix = format!("rule_{rule_index}_condition_");
    Ok(fields
        .keys()
        .filter_map(|field| field.strip_prefix(&prefix))
        .filter_map(|field| field.split_once('_')?.0.parse::<usize>().ok())
        .max()
        .map_or(0, |index| index + 1))
}

fn sieve_quote(value: &str) -> String {
    format!(
        "\"{}\"",
        value
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n")
    )
}

#[page]
pub async fn page(cx: &Cx) -> Result<impl View> {
    let user = auth::require_user(cx).await?;
    let client = webmail::sieve(cx, &user.email).await?;
    let scripts = webmail::protocol(client.sieve_scripts().await)?;
    let mailboxes = webmail::mailboxes(&client).await?;
    let mailbox_tree = webmail::mailbox_tree(&mailboxes);
    let mailbox_options = webmail::flatten_mailbox_tree(&mailbox_tree);
    let query = query_params::<SieveQuery>(cx)?;
    let selected = match (&query.name, &query.new) {
        (Some(name), _) => {
            let content = webmail::protocol(client.sieve_script(name).await)?;
            Some(ScriptDetail {
                name: name.clone(),
                content,
                active: scripts
                    .iter()
                    .any(|(script, active)| script == name && *active),
            })
        }
        (None, Some(_)) => Some(ScriptDetail {
            name: String::new(),
            content: String::new(),
            active: false,
        }),
        (None, None) => None,
    };
    let has_active = scripts.iter().any(|(_, active)| *active);
    let parsed_rules = selected
        .as_ref()
        .map(|script| parse_sieve_rules(&script.content));
    let editor_rules = parsed_rules.as_ref().and_then(|rules| {
        rules.as_ref().map(|rules| {
            let mut editor_rules = rules.clone();
            for rule in &mut editor_rules {
                rule.conditions.push(SieveConditionView::empty());
            }
            if query.add_rule.is_some() {
                let mut rule = SieveRuleView::empty();
                rule.conditions.push(SieveConditionView::empty());
                editor_rules.push(rule);
            }
            editor_rules
        })
    });
    let reorderable_rule_count = editor_rules.as_ref().map_or(0, |rules| {
        rules
            .len()
            .saturating_sub(if query.add_rule.is_some() { 1 } else { 0 })
    });
    let selected_name = selected.as_ref().map(|script| script.name.clone());
    Ok(view! {
        <main class="sieve-shell">
            <aside class="sieve-sidebar">
                <div class="sidebar-header">
                    <h2 class="sidebar-title">"Sieve Scripts"</h2>
                    topcoat::runtime::link(
                        href: format!("{}?new=1", href!(page).resolve(cx)),
                        attrs: topcoat::view::attributes! { class="new-btn" },
                        "+ New"
                    )
                </div>
                <nav class="script-list">
                    for (name, active) in scripts {
                        <div
                            class=(if selected_name.as_deref() == Some(name.as_str()) {
                                "script-item selected"
                            } else {
                                "script-item"
                            })
                        >
                            topcoat::runtime::link(
                                href: format!(
                                    "{}?name={}",
                                    href!(page).resolve(cx),
                                    utf8_percent_encode(&name, NON_ALPHANUMERIC),
                                ),
                                attrs: topcoat::view::attributes! { class="script-link" },
                                <span class="script-name">(&name)</span>
                            )
                            if active {
                                <span class="active-badge">"Active"</span>
                            }
                        </div>
                    }
                </nav>
            </aside>
            <section class="sieve-editor">
                if let Some(script) = selected {
                    <form class="sieve-form" method="post" action=(href!(save))>
                        <input type="hidden" name="original_name" value=(&script.name)>
                        <input
                            type="hidden"
                            name="has_active"
                            value=(has_active.to_string())
                        >
                        if script.active {
                            <input type="hidden" name="was_active" value="true">
                        }
                        <div class="editor-toolbar">
                            <input
                                type="text"
                                name="name"
                                value=(&script.name)
                                class="name-input"
                                placeholder="Script name"
                                required=""
                            >
                            if !script.name.is_empty() {
                                <button
                                    class="toolbar-btn"
                                    type="submit"
                                    formaction=(href!(toggle))
                                >
                                    if script.active {
                                        "Deactivate"
                                    } else {
                                        "Activate"
                                    }
                                </button>
                                <button
                                    class="toolbar-btn danger"
                                    type="submit"
                                    formaction=(href!(delete))
                                >
                                    "Delete"
                                </button>
                            }
                            <button
                                class="save-btn"
                                type="submit"
                                name="editor_mode"
                                value=(if editor_rules.is_some() {
                                    "visual"
                                } else {
                                    "raw"
                                })
                            >
                                "Save"
                            </button>
                        </div>
                        if !script.active && has_active {
                            <p class="inactive-notice">
                                "This script is saved but inactive. Activate it to apply these rules to incoming mail."
                            </p>
                        }
                        <section class="rule-builder">
                            <div class="builder-toolbar">
                                <div class="mode-toggle">
                                    <span class="mode-btn active">"Visual"</span>
                                    <details class="raw-mode">
                                        <summary class="mode-btn">"Raw"</summary>
                                        <div class="raw-panel">
                                            <textarea
                                                name="content"
                                                rows="28"
                                                spellcheck="false"
                                                placeholder="require [&quot;fileinto&quot;];"
                                            >
                                                (&script.content)
                                            </textarea>
                                            <button
                                                class="save-btn"
                                                type="submit"
                                                name="editor_mode"
                                                value="raw"
                                            >
                                                "Save raw source"
                                            </button>
                                        </div>
                                    </details>
                                </div>
                                if editor_rules.is_some() {
                                    <button
                                        class="add-rule-btn"
                                        type="submit"
                                        name="add_rule"
                                        value="true"
                                    >
                                        "+ Add rule"
                                    </button>
                                }
                            </div>
                            if let Some(rules) = editor_rules.as_ref() {
                                <section class="rules-list" aria-label="Filter rules">
                                    <input type="hidden" name="rule_count" value=(rules.len())>
                                    for (index, rule) in rules.iter().enumerate() {
                                        <article
                                            class="rule-card"
                                            id=(if query.add_rule.is_some() && index == rules.len() - 1 {
                                                "new-rule".to_owned()
                                            } else {
                                                format!("rule-{index}")
                                            })
                                        >
                                            <header class="rule-header">
                                                <div class="rule-actions">
                                                    <button
                                                        class="icon-btn"
                                                        type="submit"
                                                        formaction=(format!(
                                                            "{}?move_rule_up={index}",
                                                            href!(save).resolve(cx),
                                                        ))
                                                        disabled=(index == 0 || index >= reorderable_rule_count)
                                                        aria-label="Move rule up"
                                                    >
                                                        "↑"
                                                    </button>
                                                    <button
                                                        class="icon-btn"
                                                        type="submit"
                                                        formaction=(format!(
                                                            "{}?move_rule_down={index}",
                                                            href!(save).resolve(cx),
                                                        ))
                                                        disabled=(index + 1 >= reorderable_rule_count)
                                                        aria-label="Move rule down"
                                                    >
                                                        "↓"
                                                    </button>
                                                    <button
                                                        class="icon-btn"
                                                        type="button"
                                                        disabled=""
                                                        aria-label="Duplicate rule"
                                                    >
                                                        "⧉"
                                                    </button>
                                                    <button
                                                        class="icon-btn delete"
                                                        type="submit"
                                                        name="remove_rule"
                                                        value=(index)
                                                        title="Remove rule"
                                                        aria-label="Remove rule"
                                                    >
                                                        "×"
                                                    </button>
                                                </div>
                                            </header>
                                            <p class="rule-match">
                                                "Match "
                                                <select name=(format!("rule_{index}_match"))>
                                                    <option value="any" selected=(!rule.match_all)>
                                                        "any"
                                                    </option>
                                                    <option value="all" selected=(rule.match_all)>
                                                        "all"
                                                    </option>
                                                </select>
                                                " of the following conditions:"
                                            </p>
                                            <input
                                                type="hidden"
                                                name=(format!("rule_{index}_condition_count"))
                                                value=(rule.conditions.len())
                                            >
                                            <div class="conditions">
                                                for (condition_index, condition) in rule
                                                    .conditions
                                                    .iter()
                                                    .enumerate() {
                                                    <div
                                                        class="condition-row"
                                                        id=(format!("rule-{index}-condition-{condition_index}"))
                                                    >
                                                        <select
                                                            class="condition-field"
                                                            name=(format!(
                                                                "rule_{index}_condition_{condition_index}_field",
                                                            ))
                                                        >
                                                            <option value="from" selected=(condition.field == "from")>
                                                                "From"
                                                            </option>
                                                            <option value="to" selected=(condition.field == "to")>
                                                                "To"
                                                            </option>
                                                            <option
                                                                value="subject"
                                                                selected=(condition.field == "subject")
                                                            >
                                                                "Subject"
                                                            </option>
                                                            <option value="cc" selected=(condition.field == "cc")>
                                                                "CC"
                                                            </option>
                                                            <option
                                                                value="list-id"
                                                                selected=(condition.field == "list-id")
                                                            >
                                                                "List-ID"
                                                            </option>
                                                            <option
                                                                value="x-spam-flag"
                                                                selected=(condition.field == "x-spam-flag")
                                                            >
                                                                "Spam flag"
                                                            </option>
                                                            <option
                                                                value="x-priority"
                                                                selected=(condition.field == "x-priority")
                                                            >
                                                                "Priority"
                                                            </option>
                                                        </select>
                                                        <select
                                                            class="condition-operator"
                                                            name=(format!(
                                                                "rule_{index}_condition_{condition_index}_operator",
                                                            ))
                                                        >
                                                            <option
                                                                value="contains"
                                                                selected=(condition.operator == "contains")
                                                            >
                                                                "contains"
                                                            </option>
                                                            <option value="is" selected=(condition.operator == "is")>
                                                                "is"
                                                            </option>
                                                            <option
                                                                value="matches"
                                                                selected=(condition.operator == "matches")
                                                            >
                                                                "matches"
                                                            </option>
                                                        </select>
                                                        <input
                                                            type="text"
                                                            class="condition-value"
                                                            name=(format!(
                                                                "rule_{index}_condition_{condition_index}_value",
                                                            ))
                                                            value=(&condition.value)
                                                            placeholder="value"
                                                        >
                                                        if condition_index + 1 < rule.conditions.len() {
                                                            <label class="remove-condition" title="Remove condition">
                                                                <input
                                                                    class="visually-hidden"
                                                                    type="checkbox"
                                                                    name=(format!(
                                                                        "rule_{index}_condition_{condition_index}_remove",
                                                                    ))
                                                                    value="true"
                                                                >
                                                                "×"
                                                            </label>
                                                        }
                                                    </div>
                                                }
                                                if index < rules.len() - 1 {
                                                    <a
                                                        class="add-condition-btn"
                                                        href=(format!(
                                                            "#rule-{index}-condition-{}",
                                                            rule.conditions.len() - 1,
                                                        ))
                                                    >
                                                        "+ Add condition"
                                                    </a>
                                                }
                                            </div>
                                            <div class="rule-action">
                                                <span class="then-label">"Then"</span>
                                                <label>
                                                    <select name=(format!("rule_{index}_action"))>
                                                        <option value="keep" selected=(rule.action == "keep")>
                                                            "Keep in inbox"
                                                        </option>
                                                        <option
                                                            value="discard"
                                                            selected=(rule.action == "discard")
                                                        >
                                                            "Discard"
                                                        </option>
                                                        <option
                                                            value="fileinto"
                                                            selected=(rule.action == "fileinto")
                                                        >
                                                            "Move to folder"
                                                        </option>
                                                        <option
                                                            value="redirect"
                                                            selected=(rule.action == "redirect")
                                                        >
                                                            "Forward to"
                                                        </option>
                                                        <option
                                                            value="vacation"
                                                            selected=(rule.action == "vacation")
                                                        >
                                                            "Vacation reply"
                                                        </option>
                                                        <option
                                                            value="mark-read"
                                                            selected=(rule.action == "mark-read")
                                                        >
                                                            "Mark as read"
                                                        </option>
                                                    </select>
                                                </label>
                                                if rule.action == "fileinto" {
                                                    <select
                                                        class="action-value"
                                                        name=(format!("rule_{index}_action_value"))
                                                        aria-label="Folder name"
                                                    >
                                                        <option value="" selected=(rule.action_value.is_empty())>
                                                            "Choose a folder"
                                                        </option>
                                                        if !rule.action_value.is_empty()
                                                            && !mailbox_options
                                                                .iter()
                                                                .any(
                                                                    |(mailbox, _, path)| {
                                                                        path == &rule.action_value
                                                                            || mailbox.name == rule.action_value
                                                                    },
                                                                ) {
                                                            <option value=(&rule.action_value) selected="">
                                                                (format!("{} (not found)", rule.action_value))
                                                            </option>
                                                        }
                                                        for (mailbox, depth, path) in &mailbox_options {
                                                            <option
                                                                value=(path)
                                                                selected=(path == &rule.action_value
                                                                    || mailbox.name == rule.action_value)
                                                            >
                                                                (format!("{}{}", "\u{00a0}".repeat(depth * 2), mailbox.name))
                                                            </option>
                                                        }
                                                    </select>
                                                }
                                                if rule.action == "redirect" {
                                                    <input
                                                        type="text"
                                                        class="action-value"
                                                        name=(format!("rule_{index}_action_value"))
                                                        value=(&rule.action_value)
                                                        placeholder="Email address"
                                                    >
                                                }
                                                if rule.action == "vacation" {
                                                    <div class="vacation-fields">
                                                        <input
                                                            type="number"
                                                            min="1"
                                                            aria-label="Vacation days"
                                                            name=(format!("rule_{index}_action_days"))
                                                            value=(&rule.action_days)
                                                        >
                                                        <input
                                                            type="text"
                                                            aria-label="Vacation subject"
                                                            placeholder="Subject"
                                                            name=(format!("rule_{index}_action_subject"))
                                                            value=(&rule.action_subject)
                                                        >
                                                        <textarea
                                                            aria-label="Vacation message"
                                                            placeholder="Vacation message"
                                                            name=(format!("rule_{index}_action_value"))
                                                        >
                                                            (&rule.action_value)
                                                        </textarea>
                                                    </div>
                                                }
                                            </div>
                                            <label class="stop-label">
                                                <input
                                                    type="checkbox"
                                                    name=(format!("rule_{index}_stop"))
                                                    value="true"
                                                    checked=(rule.stop)
                                                >
                                                "Stop processing rules"
                                            </label>
                                        </article>
                                    }
                                </section>
                            } else {
                                <p class="parse-error">
                                    "This script uses syntax that cannot be shown as separate rules. Edit it in raw mode below."
                                </p>
                            }
                        </section>
                    </form>
                } else {
                    <div class="empty-state">
                        <p>"Select a script or create a new one."</p>
                    </div>
                }
            </section>
        </main>
    })
}

#[derive(Deserialize)]
struct ScriptForm {
    name: String,
    #[serde(default)]
    original_name: String,
    #[serde(default)]
    content: String,
    #[serde(default)]
    was_active: bool,
    #[serde(default)]
    has_active: bool,
    #[serde(default)]
    editor_mode: String,
    #[serde(default)]
    rule_count: Option<usize>,
    #[serde(default)]
    remove_rule: Option<usize>,
    #[serde(default)]
    add_rule: Option<String>,
    #[serde(flatten)]
    visual_fields: HashMap<String, String>,
}

#[route(POST)]
pub async fn save(cx: &Cx, Form(input): Form<ScriptForm>) -> Result<SeeOther> {
    let movement = query_params::<RuleMoveQuery>(cx)?;
    let user = auth::require_user(cx).await?;
    let name = input.name.trim();
    if name.is_empty() {
        return Err(bad_request("script name is required").into());
    }
    let client = webmail::sieve(cx, &user.email).await?;
    let mut visual_fields = input.visual_fields;
    if let Some(rule_count) = input.rule_count {
        visual_fields.insert("rule_count".to_owned(), rule_count.to_string());
    }
    if let Some(rule_index) = input.remove_rule {
        visual_fields.insert(format!("rule_{rule_index}_remove"), "true".to_owned());
    }
    let content = if input.editor_mode == "visual"
        || input.remove_rule.is_some()
        || movement.move_rule_up.is_some()
        || movement.move_rule_down.is_some()
        || input.add_rule.is_some()
        || visual_fields.contains_key("rule_count")
    {
        generate_sieve(
            &visual_fields,
            movement.move_rule_up,
            movement.move_rule_down,
        )
        .map_err(bad_request)?
    } else {
        input.content
    };
    webmail::protocol(
        client
            .save_sieve_script(
                &input.original_name,
                name,
                &content,
                input.was_active || !input.has_active,
            )
            .await,
    )?;
    let mut location = format!(
        "{}?name={}",
        href!(page).resolve(cx),
        utf8_percent_encode(name, NON_ALPHANUMERIC)
    );
    if input.add_rule.is_some() {
        location.push_str("&add_rule=1");
    }
    Ok(see_other(location))
}

#[route(POST "./delete")]
pub async fn delete(cx: &Cx, Form(input): Form<ScriptForm>) -> Result<SeeOther> {
    let user = auth::require_user(cx).await?;
    let client = webmail::sieve(cx, &user.email).await?;
    webmail::protocol(client.delete_sieve_script(&input.original_name).await)?;
    Ok(see_other(href!(page).resolve(cx)))
}

#[route(POST "./toggle")]
pub async fn toggle(cx: &Cx, Form(input): Form<ScriptForm>) -> Result<SeeOther> {
    let user = auth::require_user(cx).await?;
    let client = webmail::sieve(cx, &user.email).await?;
    webmail::protocol(
        client
            .set_active_sieve_script(&input.original_name, !input.was_active)
            .await,
    )?;
    Ok(see_other(format!(
        "{}?name={}",
        href!(page).resolve(cx),
        utf8_percent_encode(&input.original_name, NON_ALPHANUMERIC)
    )))
}
