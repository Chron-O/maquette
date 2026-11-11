// Turns the test data, or the lack of it, into what the home page shows.

use crate::test_data::{CURRENT_EVENT, Generated, Kind, Label, format_clock, format_time};
use crate::live::last_reading;
use crate::{App, Badge, BadgeKind, ClubCard, CurrentEvent, EventRow, Home, HomeLast, Icons, IndicatorItem, Reading, Stat, model};
use slint::{ComponentHandle, SharedString};

// The small rows under the last reading: 3 on a phone, 2 on wider windows.
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
    let licensed = data.runner_base.iter().filter(|r| r.licence.is_some()).count() as u32;
    let clubs = data.runner_base.iter().map(|r| r.club).collect::<std::collections::BTreeSet<_>>().len() as u32;
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
            thousands(licensed),
            format!("licenciés · {}", plural(clubs, "club")),
            badge(&Label::new(format!("Archive FFCO du {}", data.archive_date), Kind::Inactive)),
        ),
    ]
}

fn indicators(window: &App, data: Option<&Generated>) -> Vec<IndicatorItem> {
    let icons = window.global::<Icons>();
    let readers = data.map_or(0, |d| d.readers.len() as u32);
    let screens = data.map_or(0, |d| d.screens.iter().filter(|s| s.on).count() as u32);
    let asking = data.is_some_and(|d| d.request.is_some());
    let devices = data.map_or(0, |d| d.devices.len() as u32);
    let offline = data.map_or(0, |d| d.devices.iter().filter(|x| x.waiting.is_some()).count() as u32);
    let unplugged = data.and_then(|d| d.readers.iter().find(|r| r.unplugged));
    // A small pill: icon, dot and a number (Robin, 11/11); the sentence goes to screen readers.
    // Each pill keeps the width of the longest number it may show by itself during the race: a
    // reader that unplugs or a device that asks to join never moves the others (charte-ui, rule 16).
    let item = |icon, long: String, count: u32, reserve: u32, has_dot: bool, attention: bool| IndicatorItem {
        icon,
        long_text: long.into(),
        short_text: count.to_string().into(),
        long_reserve: SharedString::new(),
        short_reserve: "0".repeat(reserve.max(count).to_string().len()).into(),
        has_dot,
        attention,
    };
    let screens_word = plural(screens, "écran");
    vec![
        if let Some(reader) = unplugged {
            item(icons.get_co_station(), format!("Lecteur {} débranché", reader.name), 1, readers, true, true)
        } else if readers > 0 {
            item(icons.get_co_station(), format!("{readers} lecteurs prêts"), readers, readers, true, false)
        } else {
            item(icons.get_co_station(), "Lecteurs : aucun branché".into(), 0, 0, false, false)
        },
        if asking {
            item(icons.get_monitor_smartphone(), "1 appareil demande à rejoindre".into(), 1, devices, true, true)
        } else if offline > 0 {
            item(icons.get_monitor_smartphone(), format!("{devices} appareils · {offline} hors ligne"), devices, devices, true, true)
        } else if devices > 0 {
            item(icons.get_monitor_smartphone(), format!("{devices} appareils synchronisés"), devices, devices, true, false)
        } else {
            item(icons.get_monitor_smartphone(), "Rôle et appareils : aucun appareil relié".into(), 0, 0, false, false)
        },
        if screens > 0 {
            item(icons.get_tv(), format!("{screens_word} diffusent"), screens, screens, true, false)
        } else {
            item(icons.get_tv(), "Écrans publics : rien diffusé".into(), 0, 0, false, false)
        },
    ]
}

pub fn show(window: &App, data: Option<&Generated>) {
    let home = window.global::<Home>();
    home.set_club(model(club_cards(data)));
    home.set_indicators(model(indicators(window, data)));

    let Some(data) = data else {
        home.set_has_current_event(false);
        home.set_last(HomeLast::default());
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
    // The last reading in big while it is live, then the ones before it in small rows.
    let last = last_reading(current, &data.readers, data.clock(), false);
    home.set_last(HomeLast {
        live: last.live,
        badge: last.badge,
        name: last.name,
        category: last.category,
        time: last.time,
        place: last.place,
        left: last.left,
    });
    home.set_readings(model(
        current
            .readings
            .iter()
            .rev()
            .skip(usize::from(last.live))
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
