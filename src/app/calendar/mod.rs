use std::collections::HashMap;

use chrono::{DateTime, Datelike, Duration, NaiveDate, NaiveDateTime, TimeZone, Utc};
use dziber::{
    models::{event::CalendarEvent, task::CalendarTask},
    personal_data::calendar::{export_ics, fetch_ical, import_ics},
};
use percent_encoding::{NON_ALPHANUMERIC, utf8_percent_encode};
use serde::Deserialize;
use topcoat::{
    Result,
    asset::{Asset, asset},
    context::Cx,
    router::{
        Body,
        content::{Form, multipart::Multipart},
        error::{SeeOther, bad_request, see_other},
        href, page, query_params,
        response::Response,
        route,
    },
    view::{View, view},
};

use crate::features::{auth, calendar as feature, stalwart};

pub(super) const CALENDAR_CSS: Asset = asset!("./style.css");

#[query_params(error = bad_request)]
struct CalendarQuery {
    calendar: Option<String>,
    year: Option<i32>,
    month: Option<u32>,
    day: Option<u32>,
    view: Option<String>,
    event: Option<String>,
    edit_event: Option<String>,
    new_event: Option<String>,
    task: Option<String>,
    edit_task: Option<String>,
    new_task: Option<String>,
    import: Option<String>,
    message: Option<String>,
}

#[page]
pub async fn page(cx: &Cx) -> Result<impl View> {
    let user = auth::require_user(cx).await?;
    let query = query_params::<CalendarQuery>(cx)?;
    let today = Utc::now().date_naive();
    let anchor = NaiveDate::from_ymd_opt(
        query.year.unwrap_or_else(|| today.year()),
        query.month.unwrap_or_else(|| today.month()),
        query.day.unwrap_or_else(|| today.day()),
    )
    .ok_or_else(|| bad_request("invalid calendar date"))?;
    let view_name = query.view.as_deref().unwrap_or("month");
    if !matches!(view_name, "month" | "multiweek" | "week" | "3day" | "day") {
        return Err(bad_request("unknown calendar view").into());
    }
    let data = feature::load(cx, &user.email, anchor.year()).await?;
    let selected_id = query.calendar.as_deref().filter(|id| !id.is_empty());
    let selected_event = query
        .event
        .as_ref()
        .or(query.edit_event.as_ref())
        .and_then(|id| data.events.iter().find(|event| &event.id == id).cloned());
    let selected_event_uid = selected_event.as_ref().map(|event| event.id.clone());
    let selected_task = query
        .task
        .as_ref()
        .or(query.edit_task.as_ref())
        .and_then(|uid| data.tasks.iter().find(|task| &task.uid == uid).cloned());
    let event_editor = if query.new_event.is_some() {
        Some(EventEditor::new(selected_id, anchor))
    } else if query.edit_event.is_some() {
        Some(EventEditor::from(
            selected_event
                .as_ref()
                .ok_or_else(|| bad_request("event not found"))?,
        ))
    } else {
        None
    };
    let task_editor = if query.new_task.is_some() {
        Some(TaskEditor::new(selected_id))
    } else if query.edit_task.is_some() {
        Some(TaskEditor::from(
            selected_task
                .as_ref()
                .ok_or_else(|| bad_request("task not found"))?,
        ))
    } else {
        None
    };
    let base = href!(page).resolve(cx);
    let title = view_title(anchor, view_name);
    let previous = page_url(
        &base,
        selected_id,
        view_name,
        shift_anchor(anchor, view_name, -1),
        "",
    );
    let today_url = page_url(&base, selected_id, view_name, today, "");
    let next = page_url(
        &base,
        selected_id,
        view_name,
        shift_anchor(anchor, view_name, 1),
        "",
    );
    let all_url = page_url(&base, None, view_name, anchor, "");
    let new_event_url = page_url(&base, selected_id, view_name, anchor, "&new_event=1");
    let new_task_url = page_url(&base, selected_id, view_name, anchor, "&new_task=1");
    let import_url = page_url(&base, selected_id, view_name, anchor, "&import=1");
    let export_url = format!(
        "{}?calendar={}",
        href!(export).resolve(cx),
        encode(selected_id.unwrap_or_default())
    );
    let cells = calendar_cells(anchor, view_name);
    let show_import = query.import.is_some();

    Ok(view! {
        <main class="calendar-layout">
            <aside class="calendar-sidebar">
                <div class="actions">
                    <h2>"Calendars"</h2>
                    <span>(data.calendars.len())</span>
                    <details class="calendar-create">
                        <summary aria-label="New calendar">"+"</summary>
                        <form
                            class="form-grid"
                            method="post"
                            action=(href!(create_calendar))
                        >
                            <input
                                type="text"
                                name="name"
                                placeholder="Calendar name"
                                required=""
                            >
                            navigation_fields(
                                calendar: selected_id,
                                view: view_name,
                                anchor: anchor
                            )
                            <button class="button primary" type="submit">
                                "Create"
                            </button>
                        </form>
                    </details>
                </div>
                <div class="actions">
                    topcoat::runtime::link(
                        href: import_url.clone(),
                        attrs: topcoat::view::attributes! { class="button outline" },
                        "Import"
                    )
                    <a
                        class="button outline"
                        href=(export_url)
                        target="_blank"
                        rel="noopener"
                        download="calendar.ics"
                    >
                        "Export"
                    </a>
                    if let Some(id) = selected_id {
                        <form method="post" action=(href!(clear_calendar))>
                            <input type="hidden" name="id" value=(id)>
                            navigation_fields(
                                calendar: selected_id,
                                view: view_name,
                                anchor: anchor
                            )
                            <button class="button error" type="submit">"Clear"</button>
                        </form>
                    }
                </div>
                if let Some(message) = &query.message {
                    <p class="message">(message)</p>
                }
                <nav class="calendar-items">
                    <a
                        class=(if selected_id.is_none() {
                            "calendar-item active"
                        } else {
                            "calendar-item"
                        })
                        (link_attrs(cx, all_url))
                    >
                        <span class="calendar-name">"All"</span>
                        <span class="calendar-count">(data.events.len())</span>
                    </a>
                    for calendar in &data.calendars {
                        <div
                            class=(if selected_id == Some(calendar.id.as_str()) {
                                "calendar-item active"
                            } else {
                                "calendar-item"
                            })
                        >
                            <div class="calendar-item-heading">
                                <a
                                    class="calendar-name"
                                    (link_attrs(
                                        cx,
                                        page_url(&base, Some(&calendar.id), view_name, anchor, ""),
                                    ))
                                >
                                    (&calendar.name)
                                </a>
                                <details class="calendar-management">
                                    <summary aria-label="Manage calendar">"•••"</summary>
                                    <div class="calendar-management-menu">
                                        <details class="calendar-rename">
                                            <summary class="button clear">"Rename"</summary>
                                            <form method="post" action=(href!(rename_calendar))>
                                                <input type="hidden" name="id" value=(&calendar.id)>
                                                navigation_fields(
                                                    calendar: selected_id,
                                                    view: view_name,
                                                    anchor: anchor
                                                )
                                                <input
                                                    type="text"
                                                    name="name"
                                                    value=(&calendar.name)
                                                    aria-label="Calendar name"
                                                    required=""
                                                >
                                                <button class="button clear" type="submit">"Save"</button>
                                            </form>
                                        </details>
                                        <form method="post" action=(href!(delete_calendar))>
                                            <input type="hidden" name="id" value=(&calendar.id)>
                                            navigation_fields(
                                                calendar: selected_id,
                                                view: view_name,
                                                anchor: anchor
                                            )
                                            <button class="button clear error" type="submit">
                                                "Delete"
                                            </button>
                                        </form>
                                    </div>
                                </details>
                            </div>
                            <span class="calendar-count">
                                (data
                                    .events
                                    .iter()
                                    .filter(|event| event.calendar_id == calendar.id)
                                    .count())
                            </span>
                        </div>
                    }
                </nav>
            </aside>
            <section class="calendar-main">
                <div class="calendar-toolbar">
                    if selected_id.is_some() {
                        <a class="button primary" (link_attrs(cx, new_event_url))>
                            "New Event"
                        </a>
                        <a class="button outline" (link_attrs(cx, import_url.clone()))>
                            "Subscribe to Calendar"
                        </a>
                    } else {
                        <button class="button primary" type="button" disabled="">
                            "New Event"
                        </button>
                        <button class="button outline" type="button" disabled="">
                            "Subscribe to Calendar"
                        </button>
                    }
                </div>
                <header class="calendar-header">
                    <div class="actions">
                        <a class="button outline" (link_attrs(cx, previous))>
                            "Previous"
                        </a>
                        <h1>(title)</h1>
                        <a class="button outline" (link_attrs(cx, next))>"Next"</a>
                        <a class="button outline" (link_attrs(cx, today_url))>
                            "Today"
                        </a>
                    </div>
                    <div class="actions">
                        for (name, label) in [
                            ("day", "Day"),
                            ("3day", "3 Day"),
                            ("week", "Week"),
                            ("multiweek", "Multiweek"),
                            ("month", "Month"),
                        ] {
                            <a
                                class=(if name == view_name {
                                    "button primary"
                                } else {
                                    "button outline"
                                })
                                (link_attrs(
                                    cx,
                                    page_url(&base, selected_id, name, anchor, ""),
                                ))
                            >
                                (label)
                            </a>
                        }
                    </div>
                </header>
                if view_name == "month" || view_name == "multiweek" {
                    <div class="calendar-grid month-grid">
                        for name in ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"] {
                            <div class="day-name">(name)</div>
                        }
                        for cell in cells {
                            if let Some(date) = cell {
                                <div
                                    class=(if date == today {
                                        "calendar-cell today"
                                    } else {
                                        "calendar-cell"
                                    })
                                >
                                    <span class="day-number">(date.day())</span>
                                    day_events(
                                        base: &base,
                                        calendar: selected_id,
                                        view: view_name,
                                        anchor: anchor,
                                        events: &data.events,
                                        selected: selected_event_uid.as_deref(),
                                        date: date
                                    )
                                </div>
                            } else {
                                <div class="calendar-cell empty"></div>
                            }
                        }
                    </div>
                } else {
                    <div
                        class=(match view_name {
                            "day" => "time-grid day-view",
                            "3day" => "time-grid three-day-view",
                            "week" => "time-grid week-view",
                            _ => "time-grid",
                        })
                    >
                        <div class="time-header">
                            <div class="time-column-header"></div>
                            for date in cells.iter().flatten() {
                                <div
                                    class=(if *date == today {
                                        "day-header today"
                                    } else {
                                        "day-header"
                                    })
                                >
                                    <span class="day-header-day">
                                        (date.format("%a").to_string())
                                    </span>
                                    <span class="day-header-date">(date.day())</span>
                                </div>
                            }
                        </div>
                        <div class="time-body">
                            <div class="time-gutter">
                                for hour in 0..24 {
                                    <div class="hour-label">
                                        (match hour {
                                            0 => "Midnight".to_owned(),
                                            12 => "Noon".to_owned(),
                                            h if h < 12 => format!("{h} AM"),
                                            h => format!("{} PM", h - 12),
                                        })
                                    </div>
                                }
                            </div>
                            for date in cells.iter().flatten().copied() {
                                <div class="day-column">
                                    for _ in 0..24 {
                                        <div class="hour-slot"></div>
                                    }
                                    time_events(
                                        base: &base,
                                        calendar: selected_id,
                                        view: view_name,
                                        anchor: anchor,
                                        events: &data.events,
                                        selected: selected_event_uid.as_deref(),
                                        date: date
                                    )
                                </div>
                            }
                        </div>
                    </div>
                }
            </section>
            <aside class="calendar-detail">
                if show_import {
                    <h2>"Subscribe to Calendar"</h2>
                    <form
                        class="form-grid"
                        method="post"
                        action=(href!(import))
                        enctype="multipart/form-data"
                    >
                        navigation_fields(
                            calendar: selected_id,
                            view: view_name,
                            anchor: anchor
                        )
                        <label>
                            "Destination"
                            <select name="calendar" required="">
                                <option value="">"Choose calendar"</option>
                                for calendar in &data.calendars {
                                    <option
                                        value=(&calendar.id)
                                        selected=(selected_id == Some(calendar.id.as_str()))
                                    >
                                        (&calendar.name)
                                    </option>
                                }
                            </select>
                        </label>
                        <label>
                            "ICS file"
                            <input
                                type="file"
                                name="file"
                                accept=".ics,.ical,text/calendar"
                            >
                        </label>
                        <label>
                            "Remote iCal URL"
                            <input
                                type="url"
                                name="url"
                                placeholder="https://example.com/calendar.ics"
                            >
                        </label>
                        <p class="hint">
                            "Choose a local file or a remote URL. All events and tasks are imported."
                        </p>
                        <button class="button primary" type="submit">"Import"</button>
                    </form>
                } else if let Some(editor) = event_editor {
                    <h2>
                        if editor.id.is_empty() {
                            "New event"
                        } else {
                            "Edit event"
                        }
                    </h2>
                    <form class="form-grid" method="post" action=(href!(save_event))>
                        <input type="hidden" name="id" value=(editor.id)>
                        <input type="hidden" name="uid" value=(editor.uid)>
                        <input type="hidden" name="href" value=(editor.href)>
                        <input type="hidden" name="etag" value=(editor.etag)>
                        navigation_fields(
                            calendar: selected_id,
                            view: view_name,
                            anchor: anchor
                        )
                        <label>
                            "Calendar"
                            <select name="calendar" required="">
                                for calendar in &data.calendars {
                                    <option
                                        value=(&calendar.id)
                                        selected=(editor.calendar == calendar.id)
                                    >
                                        (&calendar.name)
                                    </option>
                                }
                            </select>
                        </label>
                        <label>
                            "Title"
                            <input
                                type="text"
                                name="title"
                                value=(editor.title)
                                required=""
                            >
                        </label>
                        <label class="checkbox">
                            <input
                                type="checkbox"
                                name="all_day"
                                value="true"
                                checked=(editor.all_day)
                            >
                            " All day"
                        </label>
                        <label>
                            "Start"
                            <input
                                type=(if editor.all_day {
                                    "date"
                                } else {
                                    "datetime-local"
                                })
                                name="start"
                                value=(editor.start)
                                required=""
                            >
                        </label>
                        <label>
                            "End"
                            <input
                                type=(if editor.all_day {
                                    "date"
                                } else {
                                    "datetime-local"
                                })
                                name="end"
                                value=(editor.end)
                                required=""
                            >
                        </label>
                        <label>
                            "Location"
                            <input type="text" name="location" value=(editor.location)>
                        </label>
                        <label>
                            "Description"
                            <textarea name="description" rows="6">
                                (editor.description)
                            </textarea>
                        </label>
                        <input type="hidden" name="status" value=(editor.status)>
                        <button class="button primary" type="submit">
                            "Save event"
                        </button>
                    </form>
                } else if let Some(event) = selected_event {
                    <h2>(&event.title)</h2>
                    <dl class="event-meta">
                        <dt>"Start"</dt>
                        <dd>(event.start.format("%Y-%m-%d %H:%M UTC").to_string())</dd>
                        <dt>"End"</dt>
                        <dd>(event.end.format("%Y-%m-%d %H:%M UTC").to_string())</dd>
                        if !event.location.is_empty() {
                            <dt>"Location"</dt>
                            <dd>(&event.location)</dd>
                        }
                        if !event.description.is_empty() {
                            <dt>"Description"</dt>
                            <dd>(&event.description)</dd>
                        }
                    </dl>
                    <div class="actions">
                        <a
                            class="button primary"
                            (link_attrs(
                                cx,
                                page_url(
                                    &base,
                                    selected_id,
                                    view_name,
                                    anchor,
                                    &format!("&edit_event={}", encode(&event.id)),
                                ),
                            ))
                        >
                            "Edit"
                        </a>
                        <form method="post" action=(href!(delete_event))>
                            <input type="hidden" name="id" value=(&event.id)>
                            <input type="hidden" name="href" value=(&event.href)>
                            <input
                                type="hidden"
                                name="etag"
                                value=(event.etag.as_deref().unwrap_or_default())
                            >
                            navigation_fields(
                                calendar: selected_id,
                                view: view_name,
                                anchor: anchor
                            )
                            <button class="button error" type="submit">"Delete"</button>
                        </form>
                    </div>
                } else if let Some(editor) = task_editor {
                    <h2>
                        if editor.id.is_empty() {
                            "New task"
                        } else {
                            "Edit task"
                        }
                    </h2>
                    <form class="form-grid" method="post" action=(href!(save_task))>
                        <input type="hidden" name="id" value=(editor.id)>
                        <input type="hidden" name="uid" value=(editor.uid)>
                        <input type="hidden" name="href" value=(editor.href)>
                        <input type="hidden" name="etag" value=(editor.etag)>
                        navigation_fields(
                            calendar: selected_id,
                            view: view_name,
                            anchor: anchor
                        )
                        <label>
                            "Calendar"
                            <select name="calendar" required="">
                                for calendar in &data.calendars {
                                    <option
                                        value=(&calendar.id)
                                        selected=(editor.calendar == calendar.id)
                                    >
                                        (&calendar.name)
                                    </option>
                                }
                            </select>
                        </label>
                        <label>
                            "Title"
                            <input
                                type="text"
                                name="title"
                                value=(editor.title)
                                required=""
                            >
                        </label>
                        <label class="checkbox">
                            <input
                                type="checkbox"
                                name="all_day"
                                value="true"
                                checked=(editor.all_day)
                            >
                            " All day"
                        </label>
                        <label>
                            "Due"
                            <input
                                type=(if editor.all_day {
                                    "date"
                                } else {
                                    "datetime-local"
                                })
                                name="due"
                                value=(editor.due)
                            >
                        </label>
                        <label>
                            "Location"
                            <input type="text" name="location" value=(editor.location)>
                        </label>
                        <label>
                            "Description"
                            <textarea name="description" rows="6">
                                (editor.description)
                            </textarea>
                        </label>
                        <label>
                            "Priority"
                            <input
                                type="number"
                                name="priority"
                                min="0"
                                max="9"
                                value=(editor.priority)
                            >
                        </label>
                        <label>
                            "Percent complete"
                            <input
                                type="number"
                                name="percent_complete"
                                min="0"
                                max="100"
                                value=(editor.percent_complete)
                            >
                        </label>
                        <input type="hidden" name="status" value=(editor.status)>
                        <input type="hidden" name="completed" value=(editor.completed)>
                        <button class="button primary" type="submit">
                            "Save task"
                        </button>
                    </form>
                } else if let Some(task) = selected_task {
                    <h2>(&task.title)</h2>
                    if let Some(due) = task.due {
                        <p>
                            "Due: "
                            (due.format("%Y-%m-%d %H:%M UTC").to_string())
                        </p>
                    }
                    <p>
                        "Status: "
                        (&task.status)
                    </p>
                    if !task.location.is_empty() {
                        <p>(&task.location)</p>
                    }
                    if !task.description.is_empty() {
                        <p>(&task.description)</p>
                    }
                    <div class="actions">
                        <a
                            class="button primary"
                            (link_attrs(
                                cx,
                                page_url(
                                    &base,
                                    selected_id,
                                    view_name,
                                    anchor,
                                    &format!("&edit_task={}", encode(&task.uid)),
                                ),
                            ))
                        >
                            "Edit"
                        </a>
                        <form method="post" action=(href!(delete_task))>
                            <input type="hidden" name="id" value=(&task.id)>
                            <input type="hidden" name="href" value=(&task.href)>
                            <input
                                type="hidden"
                                name="etag"
                                value=(task.etag.as_deref().unwrap_or_default())
                            >
                            navigation_fields(
                                calendar: selected_id,
                                view: view_name,
                                anchor: anchor
                            )
                            <button class="button error" type="submit">"Delete"</button>
                        </form>
                    </div>
                } else {
                    <div class="actions">
                        <h2>"Tasks"</h2>
                        <a
                            class="button outline"
                            (link_attrs(cx, new_task_url.clone()))
                        >
                            "New task"
                        </a>
                    </div>
                    for task in data
                        .tasks
                        .iter()
                        .filter(
                            |task| selected_id.is_none_or(|id| task.calendar_id == id),
                        ) {
                        <article class="task-row">
                            <div>
                                <a
                                    (link_attrs(
                                        cx,
                                        page_url(
                                            &base,
                                            selected_id,
                                            view_name,
                                            anchor,
                                            &format!("&task={}", encode(&task.uid)),
                                        ),
                                    ))
                                >
                                    <strong>(&task.title)</strong>
                                </a>
                                if let Some(due) = task.due {
                                    <p>
                                        <small>
                                            (due.format("%Y-%m-%d %H:%M UTC").to_string())
                                        </small>
                                    </p>
                                }
                            </div>
                            <form method="post" action=(href!(toggle_task))>
                                navigation_fields(
                                    calendar: selected_id,
                                    view: view_name,
                                    anchor: anchor
                                )
                                task_fields(task: task)
                                <button class="button outline" type="submit">
                                    if task.status == "COMPLETED" {
                                        "Reopen"
                                    } else {
                                        "Complete"
                                    }
                                </button>
                            </form>
                        </article>
                    }
                }
            </aside>
        </main>
    })
}

#[topcoat::view::component]
async fn day_events(
    cx: &Cx,
    base: &str,
    calendar: Option<&str>,
    view: &str,
    anchor: NaiveDate,
    events: &[CalendarEvent],
    selected: Option<&str>,
    date: NaiveDate,
) -> Result<impl View> {
    let day_start = date.and_hms_opt(0, 0, 0).unwrap().and_utc();
    let day_end = day_start + Duration::days(1);
    let links: Vec<_> = events
        .iter()
        .filter(|event| {
            calendar.is_none_or(|id| event.calendar_id == id)
                && event.start < day_end
                && event.end > day_start
        })
        .map(|event| {
            let url = page_url(
                base,
                calendar,
                view,
                anchor,
                &format!("&event={}", encode(&event.id)),
            );
            (
                event.title.clone(),
                event.id.clone(),
                url,
                event.start,
                event.end,
                event.all_day,
            )
        })
        .collect();
    let hidden_count = links.len().saturating_sub(3);
    Ok(view! {
        <div class="cell-events">
            for (title, uid, url, start, end, all_day) in links.into_iter().take(3) {
                <a
                    class=(if selected == Some(uid.as_str()) {
                        "event-pill selected"
                    } else {
                        "event-pill"
                    })
                    (topcoat::runtime::link_attrs(
                        cx,
                        url,
                        topcoat::runtime::prefetch_mode(cx),
                    ))
                    title=(title.clone())
                >
                    (title)
                    if !all_day && view != "month" && view != "multiweek" {
                        <small>
                            " "
                            (start.format("%H:%M").to_string())
                            "–"
                            (end.format("%H:%M").to_string())
                        </small>
                    }
                </a>
            }
            if hidden_count > 0 {
                <span class="more">(format!("+{hidden_count} more"))</span>
            }
        </div>
    })
}

#[topcoat::view::component]
async fn time_events(
    cx: &Cx,
    base: &str,
    calendar: Option<&str>,
    view: &str,
    anchor: NaiveDate,
    events: &[CalendarEvent],
    selected: Option<&str>,
    date: NaiveDate,
) -> Result<impl View> {
    let day_start = date.and_hms_opt(0, 0, 0).unwrap().and_utc();
    let day_end = day_start + Duration::days(1);
    let positioned: Vec<_> = events
        .iter()
        .filter(|event| {
            calendar.is_none_or(|id| event.calendar_id == id)
                && event.start < day_end
                && event.end > day_start
        })
        .map(|event| {
            let start = event.start.max(day_start);
            let end = event.end.min(day_end);
            let top = (start - day_start).num_seconds() as f64 / 864.0;
            let height = ((end - start).num_seconds() as f64 / 864.0).max(2.0);
            let url = page_url(
                base,
                calendar,
                view,
                anchor,
                &format!("&event={}", encode(&event.id)),
            );
            (
                event.title.clone(),
                event.id.clone(),
                url,
                format!("top:{top:.3}%;height:{height:.3}%"),
            )
        })
        .collect();

    Ok(view! {
        for (title, uid, url, style) in positioned {
            <a
                class=(if selected == Some(uid.as_str()) {
                    "time-event selected"
                } else {
                    "time-event"
                })
                style=(style)
                (link_attrs(cx, url))
                title=(title.clone())
            >
                <span class="time-event-title">(title)</span>
            </a>
        }
    })
}

#[topcoat::view::component]
async fn navigation_fields(
    cx: &Cx,
    calendar: Option<&str>,
    view: &str,
    anchor: NaiveDate,
) -> Result<impl View> {
    Ok(view! {
        <input type="hidden" name="calendar" value=(calendar.unwrap_or_default())>
        <input type="hidden" name="year" value=(anchor.year())>
        <input type="hidden" name="month" value=(anchor.month())>
        <input type="hidden" name="day" value=(anchor.day())>
        <input type="hidden" name="view" value=(view)>
    })
}

#[topcoat::view::component]
async fn task_fields(cx: &Cx, task: &CalendarTask) -> Result<impl View> {
    Ok(view! {
        <input type="hidden" name="id" value=(&task.id)>
        <input type="hidden" name="uid" value=(&task.uid)>
        <input type="hidden" name="href" value=(&task.href)>
        <input
            type="hidden"
            name="etag"
            value=(task.etag.as_deref().unwrap_or_default())
        >
        <input type="hidden" name="calendar" value=(&task.calendar_id)>
        <input type="hidden" name="title" value=(&task.title)>
        <input
            type="hidden"
            name="due"
            value=(task
                .due
                .map(
                    |due| due
                            .format(
                                if task.all_day { "%Y-%m-%d" } else { "%Y-%m-%dT%H:%M" },
                            )
                            .to_string(),
                )
                .unwrap_or_default())
        >
        <input type="hidden" name="all_day" value=(task.all_day.to_string())>
        <input type="hidden" name="location" value=(&task.location)>
        <input type="hidden" name="description" value=(&task.description)>
        <input type="hidden" name="status" value=(&task.status)>
        <input type="hidden" name="priority" value=(task.priority)>
        <input type="hidden" name="percent_complete" value=(task.percent_complete)>
        <input
            type="hidden"
            name="completed"
            value=(task.completed.map(|date| date.to_rfc3339()).unwrap_or_default())
        >
    })
}

fn link_attrs(cx: &Cx, url: String) -> topcoat::view::Attributes {
    topcoat::runtime::link_attrs(cx, url, topcoat::runtime::prefetch_mode(cx))
}

#[derive(Clone)]
struct EventEditor {
    id: String,
    uid: String,
    href: String,
    etag: String,
    calendar: String,
    title: String,
    start: String,
    end: String,
    all_day: bool,
    location: String,
    description: String,
    status: String,
}

impl EventEditor {
    fn new(calendar: Option<&str>, anchor: NaiveDate) -> Self {
        let start = anchor.and_hms_opt(9, 0, 0).unwrap();
        Self {
            id: String::new(),
            uid: String::new(),
            href: String::new(),
            etag: String::new(),
            calendar: calendar.unwrap_or_default().to_owned(),
            title: String::new(),
            start: start.format("%Y-%m-%dT%H:%M").to_string(),
            end: (start + Duration::hours(1))
                .format("%Y-%m-%dT%H:%M")
                .to_string(),
            all_day: false,
            location: String::new(),
            description: String::new(),
            status: "CONFIRMED".to_owned(),
        }
    }
}

impl From<&CalendarEvent> for EventEditor {
    fn from(event: &CalendarEvent) -> Self {
        let format = if event.all_day {
            "%Y-%m-%d"
        } else {
            "%Y-%m-%dT%H:%M"
        };
        Self {
            id: event.id.clone(),
            uid: event.uid.clone(),
            href: event.href.clone(),
            etag: event.etag.clone().unwrap_or_default(),
            calendar: event.calendar_id.clone(),
            title: event.title.clone(),
            start: event.start.format(format).to_string(),
            end: event.end.format(format).to_string(),
            all_day: event.all_day,
            location: event.location.clone(),
            description: event.description.clone(),
            status: event.status.clone(),
        }
    }
}

#[derive(Clone)]
struct TaskEditor {
    id: String,
    uid: String,
    href: String,
    etag: String,
    calendar: String,
    title: String,
    due: String,
    all_day: bool,
    location: String,
    description: String,
    status: String,
    priority: i32,
    percent_complete: i32,
    completed: String,
}

impl TaskEditor {
    fn new(calendar: Option<&str>) -> Self {
        Self {
            id: String::new(),
            uid: String::new(),
            href: String::new(),
            etag: String::new(),
            calendar: calendar.unwrap_or_default().to_owned(),
            title: String::new(),
            due: String::new(),
            all_day: false,
            location: String::new(),
            description: String::new(),
            status: "NEEDS-ACTION".to_owned(),
            priority: 0,
            percent_complete: 0,
            completed: String::new(),
        }
    }
}

impl From<&CalendarTask> for TaskEditor {
    fn from(task: &CalendarTask) -> Self {
        Self {
            id: task.id.clone(),
            uid: task.uid.clone(),
            href: task.href.clone(),
            etag: task.etag.clone().unwrap_or_default(),
            calendar: task.calendar_id.clone(),
            title: task.title.clone(),
            due: task
                .due
                .map(|due| {
                    due.format(if task.all_day {
                        "%Y-%m-%d"
                    } else {
                        "%Y-%m-%dT%H:%M"
                    })
                    .to_string()
                })
                .unwrap_or_default(),
            all_day: task.all_day,
            location: task.location.clone(),
            description: task.description.clone(),
            status: task.status.clone(),
            priority: task.priority,
            percent_complete: task.percent_complete,
            completed: task
                .completed
                .map(|date| date.to_rfc3339())
                .unwrap_or_default(),
        }
    }
}

#[derive(Clone, Default, Deserialize)]
struct Navigation {
    calendar: String,
    #[serde(deserialize_with = "number")]
    year: i32,
    #[serde(default = "default_month", deserialize_with = "number")]
    month: u32,
    #[serde(default = "default_day", deserialize_with = "number")]
    day: u32,
    #[serde(default = "default_view")]
    view: String,
}

fn number<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: serde::Deserializer<'de>,
    T: std::str::FromStr,
    T::Err: std::fmt::Display,
{
    let value = String::deserialize(deserializer)?;
    value.parse().map_err(serde::de::Error::custom)
}

fn default_month() -> u32 {
    1
}
fn default_day() -> u32 {
    1
}
fn default_view() -> String {
    "month".to_owned()
}

#[derive(Deserialize)]
struct CalendarForm {
    #[serde(default)]
    id: String,
    #[serde(default)]
    name: String,
    #[serde(flatten)]
    nav: Navigation,
}

#[route(POST "./create-calendar")]
async fn create_calendar(cx: &Cx, Form(input): Form<CalendarForm>) -> Result<SeeOther> {
    let user = auth::require_user(cx).await?;
    let name = input.name.trim();
    if name.is_empty() {
        return Err(bad_request("calendar name is required").into());
    }
    let id = slug(name);
    let id = stalwart::protocol(
        feature::client(cx, &user.email)
            .create_calendar(&id, name)
            .await,
    )?;
    Ok(see_other(page_url(
        &href!(page).resolve(cx),
        Some(&id),
        &input.nav.view,
        nav_date(&input.nav)?,
        "",
    )))
}

#[route(POST "./rename-calendar")]
async fn rename_calendar(cx: &Cx, Form(input): Form<CalendarForm>) -> Result<SeeOther> {
    let user = auth::require_user(cx).await?;
    let data = feature::load(cx, &user.email, input.nav.year).await?;
    let calendar = feature::selected_calendar(&data, Some(&input.id))
        .ok_or_else(|| bad_request("calendar not found"))?;
    stalwart::protocol(
        feature::client(cx, &user.email)
            .rename_calendar(calendar, input.name.trim())
            .await,
    )?;
    Ok(redirect(input.nav))
}

#[route(POST "./delete-calendar")]
async fn delete_calendar(cx: &Cx, Form(input): Form<CalendarForm>) -> Result<SeeOther> {
    let user = auth::require_user(cx).await?;
    let data = feature::load(cx, &user.email, input.nav.year).await?;
    let calendar = feature::selected_calendar(&data, Some(&input.id))
        .ok_or_else(|| bad_request("calendar not found"))?;
    stalwart::protocol(
        feature::client(cx, &user.email)
            .delete_calendar(calendar)
            .await,
    )?;
    Ok(redirect(Navigation {
        calendar: String::new(),
        ..input.nav
    }))
}

#[derive(Deserialize)]
struct EventForm {
    #[serde(default)]
    id: String,
    #[serde(default)]
    uid: String,
    #[serde(default)]
    href: String,
    #[serde(default)]
    etag: String,
    title: String,
    start: String,
    end: String,
    #[serde(default)]
    all_day: bool,
    #[serde(default)]
    location: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    status: String,
    #[serde(flatten)]
    nav: Navigation,
}

#[route(POST "./save-event")]
async fn save_event(cx: &Cx, Form(input): Form<EventForm>) -> Result<SeeOther> {
    let user = auth::require_user(cx).await?;
    let data = feature::load(cx, &user.email, input.nav.year).await?;
    let calendar = feature::selected_calendar(&data, Some(&input.nav.calendar))
        .ok_or_else(|| bad_request("calendar not found"))?;
    let uid = if input.uid.is_empty() {
        uuid::Uuid::new_v4().to_string()
    } else {
        input.uid
    };
    let start = parse_datetime(&input.start, input.all_day)?;
    let end = parse_datetime(&input.end, input.all_day)?;
    if end <= start {
        return Err(bad_request("event end must be after its start").into());
    }
    let href = if input.href.is_empty() {
        format!("{}{uid}.ics", calendar.href)
    } else {
        input.href
    };
    let event = CalendarEvent {
        id: input.id,
        account_id: user.email.clone(),
        calendar_id: calendar.id.clone(),
        href,
        etag: (!input.etag.is_empty()).then_some(input.etag),
        uid,
        title: input.title,
        start,
        end,
        all_day: input.all_day,
        description: input.description,
        location: input.location,
        status: input.status,
        raw_ics: String::new(),
    };
    stalwart::protocol(feature::client(cx, &user.email).save_event(&event).await)?;
    Ok(redirect(input.nav))
}

#[derive(Deserialize)]
struct EventRef {
    #[serde(default)]
    id: String,
    #[serde(default)]
    href: String,
    #[serde(default)]
    etag: String,
    #[serde(flatten)]
    nav: Navigation,
}

#[route(POST "./delete-event")]
async fn delete_event(cx: &Cx, Form(input): Form<EventRef>) -> Result<SeeOther> {
    let user = auth::require_user(cx).await?;
    let event = CalendarEvent {
        id: input.id,
        href: input.href,
        etag: (!input.etag.is_empty()).then_some(input.etag),
        ..Default::default()
    };
    stalwart::protocol(feature::client(cx, &user.email).delete_event(&event).await)?;
    Ok(redirect(input.nav))
}

#[derive(Deserialize)]
struct TaskForm {
    #[serde(default)]
    id: String,
    #[serde(default)]
    uid: String,
    #[serde(default)]
    href: String,
    #[serde(default)]
    etag: String,
    title: String,
    #[serde(default)]
    due: String,
    #[serde(default)]
    all_day: bool,
    #[serde(default)]
    location: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    status: String,
    #[serde(default)]
    priority: i32,
    #[serde(default)]
    percent_complete: i32,
    #[serde(default)]
    completed: String,
    #[serde(flatten)]
    nav: Navigation,
}

#[route(POST "./save-task")]
async fn save_task(cx: &Cx, Form(input): Form<TaskForm>) -> Result<SeeOther> {
    let user = auth::require_user(cx).await?;
    let nav = input.nav.clone();
    let task = task_from_form(
        &user.email,
        &data_for_form(cx, &user.email, nav.year).await?,
        input,
    )
    .await?;
    stalwart::protocol(feature::client(cx, &user.email).save_task(&task).await)?;
    Ok(redirect(nav))
}

#[derive(Deserialize)]
struct TaskRef {
    #[serde(default)]
    id: String,
    #[serde(default)]
    href: String,
    #[serde(default)]
    etag: String,
    #[serde(flatten)]
    nav: Navigation,
}

#[route(POST "./delete-task")]
async fn delete_task(cx: &Cx, Form(input): Form<TaskRef>) -> Result<SeeOther> {
    let user = auth::require_user(cx).await?;
    let task = CalendarTask {
        id: input.id,
        href: input.href,
        etag: (!input.etag.is_empty()).then_some(input.etag),
        ..Default::default()
    };
    stalwart::protocol(feature::client(cx, &user.email).delete_task(&task).await)?;
    Ok(redirect(input.nav))
}

#[route(POST "./toggle-task")]
async fn toggle_task(cx: &Cx, Form(input): Form<TaskForm>) -> Result<SeeOther> {
    let user = auth::require_user(cx).await?;
    let nav = input.nav.clone();
    let mut task = task_from_form(
        &user.email,
        &data_for_form(cx, &user.email, nav.year).await?,
        input,
    )
    .await?;
    let complete = task.status != "COMPLETED";
    task.status = if complete {
        "COMPLETED".to_owned()
    } else {
        "NEEDS-ACTION".to_owned()
    };
    task.percent_complete = if complete { 100 } else { 0 };
    task.completed = complete.then(Utc::now);
    stalwart::protocol(feature::client(cx, &user.email).save_task(&task).await)?;
    Ok(redirect(nav))
}

#[route(POST "./clear-calendar")]
async fn clear_calendar(cx: &Cx, Form(input): Form<EventRef>) -> Result<SeeOther> {
    let user = auth::require_user(cx).await?;
    let data = feature::load(cx, &user.email, input.nav.year).await?;
    let client = feature::client(cx, &user.email);
    for event in data
        .events
        .iter()
        .filter(|event| event.calendar_id == input.nav.calendar)
    {
        stalwart::protocol(client.delete_event(event).await)?;
    }
    for task in data
        .tasks
        .iter()
        .filter(|task| task.calendar_id == input.nav.calendar)
    {
        stalwart::protocol(client.delete_task(task).await)?;
    }
    Ok(redirect(input.nav))
}

#[route(POST "./import")]
async fn import(cx: &Cx, mut multipart: Multipart) -> Result<SeeOther> {
    let user = auth::require_user(cx).await?;
    let mut fields = HashMap::new();
    let mut file = None;
    while let Some(field) = multipart.next_field().await? {
        let name = field.name().unwrap_or_default().to_owned();
        if name == "file" {
            let data = field.bytes().await?.to_vec();
            if !data.is_empty() {
                file = Some(data)
            }
        } else {
            fields.insert(name, field.text().await?);
        }
    }
    let nav = navigation_from(&fields)?;
    let calendar_id = nav.calendar.clone();
    let data = feature::load(cx, &user.email, nav.year).await?;
    let calendar = feature::selected_calendar(&data, Some(&calendar_id))
        .ok_or_else(|| bad_request("calendar not found"))?;
    let text = if let Some(file) = file {
        String::from_utf8(file).map_err(|_| bad_request("ICS file must be UTF-8"))?
    } else if let Some(url) = fields
        .get("url")
        .map(|url| url.trim())
        .filter(|url| !url.is_empty())
    {
        stalwart::protocol(fetch_ical(url).await)?
    } else {
        return Err(bad_request("choose an ICS file or URL").into());
    };
    let (events, tasks) = import_ics(&text, &user.email, &calendar.id);
    let client = feature::client(cx, &user.email);
    let mut imported = 0;
    for mut event in events {
        event.uid = uuid::Uuid::new_v4().to_string();
        event.href = format!("{}{}.ics", calendar.href, event.uid);
        event.calendar_id = calendar.id.clone();
        stalwart::protocol(client.save_event(&event).await)?;
        imported += 1;
    }
    for mut task in tasks {
        task.uid = uuid::Uuid::new_v4().to_string();
        task.href = format!("{}{}.ics", calendar.href, task.uid);
        task.calendar_id = calendar.id.clone();
        stalwart::protocol(client.save_task(&task).await)?;
        imported += 1;
    }
    Ok(see_other(format!(
        "{}&message={}",
        page_url(
            &href!(page).resolve(cx),
            Some(&calendar.id),
            &nav.view,
            nav_date(&nav)?,
            ""
        ),
        encode(&format!("Imported {imported} item(s)"))
    )))
}

#[query_params(error = bad_request)]
struct ExportQuery {
    calendar: String,
}

#[route(GET "./export")]
async fn export(cx: &Cx) -> Result<Response> {
    let user = auth::require_user(cx).await?;
    let query = query_params::<ExportQuery>(cx)?;
    let data = feature::load(cx, &user.email, 0).await?;
    let events: Vec<_> = data
        .events
        .into_iter()
        .filter(|event| query.calendar.is_empty() || event.calendar_id == query.calendar)
        .collect();
    let tasks: Vec<_> = data
        .tasks
        .into_iter()
        .filter(|task| query.calendar.is_empty() || task.calendar_id == query.calendar)
        .collect();
    let body = export_ics(&events, &tasks);
    Ok(Response::builder()
        .header("Content-Type", "text/calendar; charset=utf-8")
        .header("Content-Disposition", "attachment; filename=calendar.ics")
        .body(Body::from(body))?)
}

async fn data_for_form(cx: &Cx, email: &str, year: i32) -> Result<feature::CalendarData> {
    feature::load(cx, email, year).await
}

async fn task_from_form(
    email: &str,
    data: &feature::CalendarData,
    input: TaskForm,
) -> Result<CalendarTask> {
    let calendar = feature::selected_calendar(data, Some(&input.nav.calendar))
        .ok_or_else(|| bad_request("calendar not found"))?;
    let uid = if input.uid.is_empty() {
        uuid::Uuid::new_v4().to_string()
    } else {
        input.uid
    };
    let href = if input.href.is_empty() {
        format!("{}{uid}.ics", calendar.href)
    } else {
        input.href
    };
    let due = if input.due.is_empty() {
        None
    } else {
        Some(parse_datetime(&input.due, input.all_day)?)
    };
    let completed = if input.completed.is_empty() {
        None
    } else {
        Some(
            DateTime::parse_from_rfc3339(&input.completed)
                .map_err(|_| bad_request("invalid completion date"))?
                .with_timezone(&Utc),
        )
    };
    Ok(CalendarTask {
        id: input.id,
        account_id: email.to_owned(),
        calendar_id: calendar.id.clone(),
        href,
        etag: (!input.etag.is_empty()).then_some(input.etag),
        uid,
        title: input.title,
        due,
        all_day: input.all_day,
        description: input.description,
        location: input.location,
        status: input.status,
        priority: input.priority,
        percent_complete: input.percent_complete,
        completed,
        raw_ics: String::new(),
    })
}

fn navigation_from(fields: &HashMap<String, String>) -> Result<Navigation> {
    Ok(Navigation {
        calendar: fields.get("calendar").cloned().unwrap_or_default(),
        year: fields
            .get("year")
            .and_then(|value| value.parse().ok())
            .unwrap_or_else(|| Utc::now().year()),
        month: fields
            .get("month")
            .and_then(|value| value.parse().ok())
            .unwrap_or(1),
        day: fields
            .get("day")
            .and_then(|value| value.parse().ok())
            .unwrap_or(1),
        view: fields.get("view").cloned().unwrap_or_else(default_view),
    })
}

fn parse_datetime(value: &str, all_day: bool) -> Result<DateTime<Utc>> {
    if all_day {
        if let Ok(date) = NaiveDate::parse_from_str(value, "%Y-%m-%d") {
            return Ok(Utc.from_utc_datetime(&date.and_hms_opt(0, 0, 0).unwrap()));
        }
        let date = NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M")
            .map_err(|_| bad_request("invalid date"))?;
        Ok(Utc.from_utc_datetime(&date.date().and_hms_opt(0, 0, 0).unwrap()))
    } else {
        let date = NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M")
            .map_err(|_| bad_request("invalid date and time"))?;
        Ok(Utc.from_utc_datetime(&date))
    }
}

fn redirect(nav: Navigation) -> SeeOther {
    let date = nav_date(&nav).unwrap_or_else(|_| Utc::now().date_naive());
    see_other(page_url(
        "/calendar",
        Some(&nav.calendar),
        &nav.view,
        date,
        "",
    ))
}

fn nav_date(nav: &Navigation) -> Result<NaiveDate> {
    NaiveDate::from_ymd_opt(nav.year, nav.month, nav.day)
        .ok_or_else(|| bad_request("invalid calendar date").into())
}

fn page_url(
    base: &str,
    calendar: Option<&str>,
    view: &str,
    anchor: NaiveDate,
    extra: &str,
) -> String {
    let mut url = format!(
        "{base}?view={}&year={}&month={}&day={}",
        encode(view),
        anchor.year(),
        anchor.month(),
        anchor.day()
    );
    if let Some(calendar) = calendar.filter(|calendar| !calendar.is_empty()) {
        url.push_str(&format!("&calendar={}", encode(calendar)));
    }
    url.push_str(extra);
    url
}

fn shift_anchor(anchor: NaiveDate, view: &str, direction: i64) -> NaiveDate {
    match view {
        "day" => anchor + Duration::days(direction),
        "3day" => anchor + Duration::days(3 * direction),
        "week" | "multiweek" => anchor + Duration::weeks(direction),
        _ => shift_month(anchor, direction),
    }
}

fn shift_month(anchor: NaiveDate, direction: i64) -> NaiveDate {
    let index = anchor.year() * 12 + anchor.month0() as i32 + direction as i32;
    NaiveDate::from_ymd_opt(index.div_euclid(12), index.rem_euclid(12) as u32 + 1, 1).unwrap()
}

fn start_of_week(date: NaiveDate) -> NaiveDate {
    date - Duration::days(date.weekday().num_days_from_monday() as i64)
}

fn calendar_cells(anchor: NaiveDate, view: &str) -> Vec<Option<NaiveDate>> {
    match view {
        "day" => vec![Some(anchor)],
        "3day" => (0..3)
            .map(|day| Some(anchor + Duration::days(day)))
            .collect(),
        "week" => (0..7)
            .map(|day| Some(start_of_week(anchor) + Duration::days(day)))
            .collect(),
        "multiweek" => (0..28)
            .map(|day| Some(start_of_week(anchor) + Duration::days(day)))
            .collect(),
        _ => {
            let first = anchor.with_day(1).unwrap();
            let offset = first.weekday().num_days_from_monday() as i64;
            let days = (shift_month(first, 1) - first).num_days();
            let total = ((offset + days + 6) / 7) * 7;
            (0..total)
                .map(|index| {
                    let day = index - offset + 1;
                    (day >= 1 && day <= days).then(|| first + Duration::days(day - 1))
                })
                .collect()
        }
    }
}

fn view_title(anchor: NaiveDate, view: &str) -> String {
    match view {
        "day" => anchor.format("%A %d %B %Y").to_string(),
        "3day" => format!(
            "{} – {}",
            anchor.format("%d %b"),
            (anchor + Duration::days(2)).format("%d %b %Y")
        ),
        "week" => format!(
            "{} – {}",
            start_of_week(anchor).format("%d %b"),
            (start_of_week(anchor) + Duration::days(6)).format("%d %b %Y")
        ),
        "multiweek" => format!(
            "{} – {}",
            start_of_week(anchor).format("%d %b"),
            (start_of_week(anchor) + Duration::days(27)).format("%d %b %Y")
        ),
        _ => anchor.format("%B %Y").to_string(),
    }
}

fn slug(name: &str) -> String {
    let slug: String = name
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    if slug.is_empty() {
        "calendar".to_owned()
    } else {
        slug.chars().take(40).collect()
    }
}

fn encode(value: &str) -> String {
    utf8_percent_encode(value, NON_ALPHANUMERIC).to_string()
}
