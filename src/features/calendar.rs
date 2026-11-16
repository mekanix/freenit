use dziber::{
    models::{
        account::{AuthMode, CalendarAccount, DavOrJmap},
        calendar::Calendar,
        event::CalendarEvent,
        task::CalendarTask,
    },
    personal_data::calendar::CalendarClient,
};
use topcoat::{Result, context::Cx};

use crate::{
    features::stalwart,
};

pub struct CalendarData {
    pub calendars: Vec<Calendar>,
    pub events: Vec<CalendarEvent>,
    pub tasks: Vec<CalendarTask>,
}

pub fn client(cx: &Cx, email: &str) -> CalendarClient {
    let config = stalwart::stalwart_config(cx);
    let protocol = topcoat::context::context::<crate::config::Config>(cx)
        .calendar
        .protocol;
    CalendarClient::new(&CalendarAccount {
        id: email.to_owned(),
        server_url: config.url,
        username: email.to_owned(),
        password: config.password.clone(),
        auth_mode: AuthMode::StalwartImpersonation {
            admin_user: config.admin.trim_start_matches('%').to_owned(),
            admin_pass: config.password,
        },
        calendar_protocol: match protocol {
            crate::config::PersonalDataProtocol::Dav => DavOrJmap::Dav,
            crate::config::PersonalDataProtocol::Jmap => DavOrJmap::Jmap,
        },
    })
}

pub async fn load(cx: &Cx, email: &str, year: i32) -> Result<CalendarData> {
    let client = client(cx, email);
    let calendars = stalwart::protocol(client.list_calendars().await)?;
    let mut events = Vec::new();
    let mut tasks = Vec::new();
    for calendar in &calendars {
        events.extend(stalwart::protocol(
            client.list_events(calendar, year).await,
        )?);
        tasks.extend(stalwart::protocol(client.list_tasks(calendar, year).await)?);
    }
    events.sort_by_key(|event| event.start);
    tasks.sort_by(|left, right| {
        (left.status == "COMPLETED")
            .cmp(&(right.status == "COMPLETED"))
            .then(left.due.cmp(&right.due))
            .then(left.title.cmp(&right.title))
    });
    Ok(CalendarData {
        calendars,
        events,
        tasks,
    })
}

pub fn selected_calendar<'a>(data: &'a CalendarData, id: Option<&str>) -> Option<&'a Calendar> {
    id.and_then(|id| data.calendars.iter().find(|calendar| calendar.id == id))
}
