// Turns the test data, or the lack of it, into what the home page shows.

use crate::test_data::{CURRENT_EVENT, Generated, Kind, Label, format_clock, format_time};
use crate::{App, Badge, BadgeKind, ClubCard, CurrentEvent, EventRow, Home, Icons, IndicatorItem, Reading, Stat, model};
use slint::{ComponentHandle, SharedString};

const READINGS_SHOWN: usize = 3;

pub fn badge(label: &Label) -> Badge {
    Badge {
        word: label.word.clone().into(),
        kind: match label.kind {
            Kind::None => BadgeKind::None,
            Kind::Valid => BadgeKind::Valid,
            Kind::Racing => BadgeKind::Racing,
            Kind::ToCheck => BadgeKind::ToCheck,
            Kind::Inactive => BadgeKind::Inactive,
            Kind::Ok => BadgeKind::Ok,
            Kind::Pm => BadgeKind::Pm,
            Kind::Unknown => BadgeKind::Unknown,
        },
    }
}

fn no_badge() -> Badge {
    badge(&Label::default())
}

// A count and its noun, with a plural s when needed.
fn plural(count: u32, noun: &str) -> String {
    format!("{count} {noun}{}", if count > 1 { "s" } else { "" })
}

fn thousands(value: u32) -> String {
    if value >= 1000 { format!("{} {:03}", value / 1000, value % 1000) } else { value.to_string() }
}

fn event_row(data: &Generated, id: usize) -> EventRow {
    let event = &data.events[id];
    EventRow {
        id: id as i32,
        day: event.day.to_string().into(),
        month: event.month_short.into(),
        name: event.name.clone().into(),
        place: format!("{} · {}", event.weekday, event.place).into(),
        participants: event.participants.clone().into(),
        courses: if event.courses.is_empty() { "aucun circuit".into() } else { plural(event.courses.len() as u32, "circuit").into() },
        badge: badge(&event.label),
    }
}

fn club_cards(data: Option<&Generated>) -> Vec<ClubCard> {
    let card = |title: &str, value: String, detail: String, badge: Badge| ClubCard {
        title: title.into(),
        value: value.into(),
        detail: detail.into(),
        badge,
    };
    let Some(data) = data else {
        return vec![
            card("Puces de prêt", "0".into(), "".into(), no_badge()),
            card("Boîtiers", "0".into(), "".into(), no_badge()),
            card("Coureurs et clubs", "0".into(), "".into(), no_badge()),
        ];
    };
    let club = &data.club;
    let overdue = data.overdue_cards().len() as u32;
    let parks = data.club_parks().count() as u32;
    let stations = data.club_parks().map(|p| p.stations.len()).sum::<usize>() as u32;
    let batteries = data.batteries_due().len() as u32;
    let alert = |count: u32, text: String| {
        if count > 0 { badge(&Label { word: text, kind: Kind::ToCheck }) } else { no_badge() }
    };
    vec![
        card(
            "Puces de prêt",
            data.cards_free().to_string(),
            format!("au parc · {}", plural(data.cards_lent(), "prêtée")),
            alert(overdue, plural(overdue, "non rendue")),
        ),
        card(
            "Boîtiers",
            stations.to_string(),
            format!("dans {}", plural(parks, "parc")),
            alert(batteries, if batteries > 1 { format!("{batteries} piles à changer") } else { format!("{batteries} pile à changer") }),
        ),
        card(
            "Coureurs et clubs",
            thousands(club.runners),
            format!("licenciés · {} clubs", club.clubs),
            no_badge(),
        ),
    ]
}

fn indicators(window: &App, data: Option<&Generated>) -> Vec<IndicatorItem> {
    let icons = window.global::<Icons>();
    let (readers, asking, screens) = data.map_or((0, 0, 0), |d| (d.readers.len() as u32, d.devices_asking, d.screens));
    let item = |icon, long: String, short: String, has_dot: bool, attention: bool| IndicatorItem {
        icon,
        long_text: long.into(),
        short_text: short.into(),
        has_dot,
        attention,
    };
    vec![
        if readers > 0 {
            item(icons.get_co_station(), format!("{readers} lecteurs prêts"), readers.to_string(), true, false)
        } else {
            item(icons.get_co_station(), "Lecteurs : aucun branché".into(), "".into(), false, false)
        },
        if asking > 0 {
            item(icons.get_monitor_smartphone(), "1 appareil demande à rejoindre".into(), "1 demande".into(), true, true)
        } else {
            item(icons.get_monitor_smartphone(), "Rôle et appareils : aucun appareil relié".into(), "".into(), false, false)
        },
        if screens > 0 {
            item(icons.get_tv(), format!("{} diffusent", plural(screens, "écran")), screens.to_string(), true, false)
        } else {
            item(icons.get_tv(), "Écrans publics : rien diffusé".into(), "".into(), false, false)
        },
    ]
}

pub fn show(window: &App, data: Option<&Generated>) {
    let home = window.global::<Home>();
    home.set_club(model(club_cards(data)));
    home.set_indicators(model(indicators(window, data)));

    let Some(data) = data else {
        home.set_has_current_event(false);
        home.set_readers(model(Vec::new()));
        home.set_readings(model(Vec::new()));
        home.set_to_handle("".into());
        home.set_upcoming(model(Vec::new()));
        home.set_past(model(Vec::new()));
        return;
    };

    let current = &data.events[CURRENT_EVENT];
    let stat = |value: u32, label: &str| Stat { value: value.to_string().into(), label: label.into() };
    home.set_has_current_event(true);
    home.set_current_event(CurrentEvent {
        id: CURRENT_EVENT as i32,
        name: current.name.clone().into(),
        detail: current.detail().into(),
        stats: model(vec![
            stat(data.racing(), "en course"),
            stat(data.back(), "arrivés"),
            stat(current.courses.len() as u32, "circuits"),
        ]),
    });
    home.set_readers(model(data.readers.iter().map(|r| SharedString::from(r.as_str())).collect()));
    home.set_readings(model(
        current
            .readings
            .iter()
            .rev()
            .take(READINGS_SHOWN)
            .map(|r| Reading {
                badge: badge(&current.reading_label(r)),
                name: current.reading_name(r).into(),
                time: format_time(r.seconds).into(),
                clock: format_clock(r.at).into(),
            })
            .collect(),
    ));
    let to_handle = current.to_handle();
    home.set_to_handle(if to_handle > 0 { format!("{to_handle} à traiter ›").into() } else { "".into() });
    home.set_upcoming(model(data.upcoming.iter().map(|id| event_row(data, *id)).collect()));
    home.set_past(model(data.past.iter().map(|id| event_row(data, *id)).collect()));
}
