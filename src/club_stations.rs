// The Boîtiers page of the club: its stations, park by park. Readings from the Démo panel,
// the states and the changed batteries work on the test data.

use crate::cards::plural;
use crate::home::badge;
use crate::test_data::{CLOCK_MAX_GAP, Generated, Kind, Label, StockState, StockStation, UNIDENTIFIED_PARK};
use crate::{App, BatteryDueItem, CardAction, CardEntry, CardParkItem, ClubStationRow, ClubStations, LiveFilter, model};
use slint::{ComponentHandle, SharedString};

const FILTERS: [&str; 6] = ["Tous", "En service", "À vérifier", "Pile à changer", "En réparation", "Perdus"];

// What the buttons of the detail do.
const ACT_IN_SERVICE: i32 = 0;
const ACT_TO_CHECK: i32 = 1;
const ACT_REPAIR: i32 = 2;
const ACT_LOST: i32 = 3;
const ACT_DESTROYED: i32 = 4;

#[derive(Default)]
pub struct ClubStationsState {
    park: usize,
    filter: usize,
    search: String,
    selected: Option<u32>,
    to_confirm: Option<StockState>,
    last_read: Option<u32>,
}

fn state_label(state: StockState) -> Label {
    match state {
        StockState::InService => Label::new("En service", Kind::Valid),
        StockState::ToCheck => Label::new("À vérifier", Kind::ToCheck),
        StockState::BatteryDue => Label::new("Pile à changer", Kind::ToCheck),
        StockState::InRepair => Label::new("En réparation", Kind::Inactive),
        StockState::Lost => Label::new("Perdu", Kind::Inactive),
        StockState::Destroyed => Label::new("Détruit", Kind::Pm),
    }
}

fn battery_text(station: &StockStation) -> String {
    if station.old_battery() { format!("{} · ancienne", station.battery_year) } else { station.battery_year.to_string() }
}

fn charge_text(station: &StockStation) -> String {
    station.charge.map_or("—".into(), |charge| {
        let volts = format!("{},{:02} V", charge / 100, charge % 100);
        if station.low_charge() { format!("{volts} · faible") } else { volts }
    })
}

fn clock_text(station: &StockStation) -> String {
    station.clock_gap.map_or("—".into(), |gap| {
        let text = if gap == 0 { "0 s".to_string() } else { format!("+{gap} s") };
        if station.clock_off() { format!("{text} · à régler") } else { text }
    })
}

// What is wrong with the battery, when the table has no column left for it.
fn battery_warning(station: &StockStation) -> String {
    let mut warnings = Vec::new();
    if station.old_battery() {
        warnings.push(format!("Pile {}", battery_text(station)));
    }
    if station.low_charge() {
        warnings.push(format!("Charge {}", charge_text(station)));
    }
    warnings.join(" · ")
}

fn matches_filter(station: &StockStation, filter: usize) -> bool {
    match filter {
        1 => station.state == StockState::InService,
        2 => station.state == StockState::ToCheck,
        3 => station.state == StockState::BatteryDue,
        4 => station.state == StockState::InRepair,
        5 => station.state == StockState::Lost,
        _ => true,
    }
}

fn actions_of(station: &StockStation) -> Vec<CardAction> {
    let action = |id: i32, label: &str, danger: bool| CardAction { id, label: label.into(), danger };
    let mut actions = Vec::new();
    match station.state {
        StockState::InService => actions.push(action(ACT_TO_CHECK, "À vérifier", false)),
        StockState::ToCheck => actions.push(action(ACT_IN_SERVICE, "En service", false)),
        StockState::InRepair | StockState::Lost => actions.push(action(ACT_IN_SERVICE, "Remis en service", false)),
        StockState::BatteryDue | StockState::Destroyed => {}
    }
    if matches!(station.state, StockState::InService | StockState::ToCheck) {
        actions.push(action(ACT_REPAIR, "En réparation", false));
    }
    if matches!(station.state, StockState::InService | StockState::ToCheck | StockState::BatteryDue | StockState::InRepair) {
        actions.push(action(ACT_LOST, "Perdu…", true));
    }
    if station.state != StockState::Destroyed {
        actions.push(action(ACT_DESTROYED, "Détruit…", true));
    }
    actions
}

impl ClubStationsState {
    pub fn select_park(&mut self, park: usize) {
        self.park = park;
        self.filter = 0;
        self.close_detail();
    }

    pub fn select_filter(&mut self, filter: usize) {
        self.filter = filter;
    }

    pub fn set_search(&mut self, text: &str) {
        self.search = text.trim().to_lowercase();
    }

    pub fn select_row(&mut self, serial: u32) {
        self.selected = if self.selected == Some(serial) { None } else { Some(serial) };
        self.to_confirm = None;
    }

    pub fn close_detail(&mut self) {
        self.selected = None;
        self.to_confirm = None;
    }

    pub fn cancel_confirm(&mut self) {
        self.to_confirm = None;
    }

    // Shows the park of a station, whichever one was on screen, with its row visible.
    fn show_park_of(&mut self, data: &Generated, serial: u32) {
        if let Some(park) = data.station_parks.iter().position(|p| p.stations.iter().any(|s| s.serial == serial)) {
            self.park = park;
        }
        self.filter = 0;
        self.search.clear();
    }

    pub fn open_station(&mut self, data: &Generated, serial: u32) {
        self.show_park_of(data, serial);
        self.selected = Some(serial);
        self.to_confirm = None;
    }

    pub fn read(&mut self, data: &mut Generated) {
        // The station read is chosen: its detail opens, its row is selected (encart-lecture.md § 8).
        if let Some(serial) = data.read_stock_station() {
            self.open_station(data, serial);
            self.last_read = Some(serial);
        }
    }

    pub fn act(&mut self, data: &mut Generated, id: i32) {
        let Some(serial) = self.selected else { return };
        match id {
            ACT_IN_SERVICE => data.set_stock_state(serial, StockState::InService, "Remis en service"),
            ACT_TO_CHECK => data.set_stock_state(serial, StockState::ToCheck, "Marqué à vérifier"),
            ACT_REPAIR => data.set_stock_state(serial, StockState::InRepair, "En réparation"),
            ACT_LOST => self.to_confirm = Some(StockState::Lost),
            ACT_DESTROYED => self.to_confirm = Some(StockState::Destroyed),
            _ => {}
        }
    }

    pub fn confirm(&mut self, data: &mut Generated) {
        let (Some(serial), Some(state)) = (self.selected, self.to_confirm.take()) else { return };
        let text = if state == StockState::Lost { "Marqué perdu" } else { "Marqué détruit" };
        data.set_stock_state(serial, state, text);
    }

    pub fn change_battery(&mut self, data: &mut Generated) {
        if let Some(serial) = self.selected {
            data.change_battery(serial);
        }
    }

    pub fn show(&self, window: &App, data: Option<&Generated>) {
        let page = window.global::<ClubStations>();
        let parks = data.map_or(&[][..], |d| &d.station_parks[..]);

        page.set_parks(model(
            parks.iter().enumerate()
                .map(|(i, p)| CardParkItem { index: i as i32, name: p.name.into(), count: p.stations.len().to_string().into() })
                .collect(),
        ));
        page.set_park_chips(model(
            parks.iter().map(|p| LiveFilter { label: p.name.into(), tone: -1, detail: SharedString::new() }).collect(),
        ));
        page.set_active_park(self.park as i32);
        page.set_active_filter(self.filter as i32);
        page.set_search_text(self.search.as_str().into());

        let park = parks.get(self.park);
        let stations = park.map_or(&[][..], |p| &p.stations[..]);
        // Each filter says how many stations of the park shown it keeps.
        page.set_filters(model(
            FILTERS.iter()
                .enumerate()
                .map(|(i, f)| {
                    let count = stations.iter().filter(|s| matches_filter(s, i)).count();
                    LiveFilter { label: format!("{f} · {count}").into(), tone: -1, detail: SharedString::new() }
                })
                .collect(),
        ));
        page.set_can_add(park.is_some_and(|p| p.name != UNIDENTIFIED_PARK));
        page.set_title(park.map_or("", |p| p.name).into());
        page.set_description(park.map_or("", |p| p.description).into());
        page.set_chips(model(park.map(|p| SharedString::from(p.description)).into_iter().collect()));

        let total = plural(stations.len(), "boîtier");
        let in_service = stations.iter().filter(|s| s.state == StockState::InService).count();
        let due = stations.iter().filter(|s| s.state == StockState::BatteryDue).count();
        let due_text = if due > 1 { format!("{due} piles à changer") } else { format!("{due} pile à changer") };
        page.set_state(total.as_str().into());
        page.set_summary(format!("{total} · {in_service} en service · {due_text}").into());
        page.set_summary_short(format!("{total} · {due_text}").into());

        let words: Vec<&str> = self.search.split_whitespace().collect();
        let rows: Vec<ClubStationRow> = stations
            .iter()
            .filter(|s| matches_filter(s, self.filter))
            .filter(|s| {
                let text = format!("{} {} {} {}", s.code, s.serial, s.kind, s.note).to_lowercase();
                words.iter().all(|word| text.contains(word))
            })
            .map(|s| ClubStationRow {
                serial: s.serial as i32,
                code: s.code.as_str().into(),
                badge: badge(&state_label(s.state)),
                battery: battery_text(s).into(),
                battery_warning: s.old_battery(),
                charge: charge_text(s).into(),
                charge_warning: s.low_charge(),
                clock: clock_text(s).into(),
                clock_warning: s.clock_off(),
                warning: battery_warning(s).into(),
                note: s.note.into(),
                selected: self.selected == Some(s.serial),
            })
            .collect();
        page.set_empty(
            if !rows.is_empty() {
                SharedString::new()
            } else if self.filter == 0 && words.is_empty() {
                "Aucun boîtier".into()
            } else {
                "Aucun résultat".into()
            },
        );
        page.set_rows(model(rows));

        let batteries = data.map_or(Vec::new(), Generated::batteries_due);
        page.set_alert_title(match batteries.len() {
            0 => SharedString::new(),
            1 => "1 pile à changer".into(),
            n => format!("{n} piles à changer").into(),
        });
        page.set_alert_count(batteries.len().to_string().into());
        page.set_batteries(model(
            batteries.iter()
                .map(|s| BatteryDueItem {
                    serial: s.serial as i32,
                    label: format!("Boîtier {} · n° {}", s.code, s.serial).into(),
                    detail: battery_warning(s).into(),
                })
                .collect(),
        ));

        self.show_reading(&page, data);
        self.show_detail(&page, data);
    }

    // Three lines (encart-lecture.md § 8): the code, who it is, what is wrong. The time, the
    // battery and the note are in the detail.
    fn show_reading(&self, page: &ClubStations, data: Option<&Generated>) {
        let found = self.last_read.and_then(|serial| {
            let data = data?;
            let station = data.stock_station(serial)?;
            let park = data.station_parks.iter().find(|p| p.stations.iter().any(|s| s.serial == serial))?;
            Some((station, park.name))
        });
        page.set_has_reading(found.is_some());
        let Some((station, park)) = found else { return };
        page.set_read_code(station.code.as_str().into());
        page.set_read_identity(format!("n° {} · {} · {park}", station.serial, station.kind).into());
        let mut verdict = Vec::new();
        if station.state == StockState::BatteryDue || station.old_battery() || station.low_charge() {
            verdict.push("Pile à changer");
        }
        if station.clock_off() {
            verdict.push("Heure à régler");
        }
        page.set_read_verdict(if verdict.is_empty() { "OK".into() } else { verdict.join(" · ").into() });
    }

    fn show_detail(&self, page: &ClubStations, data: Option<&Generated>) {
        let found = self.selected.and_then(|serial| {
            let data = data?;
            let station = data.stock_station(serial)?;
            let park = data.station_parks.iter().find(|p| p.stations.iter().any(|s| s.serial == serial))?;
            Some((station, park.name))
        });
        page.set_detail_open(found.is_some());
        let Some((station, park)) = found else { return };

        page.set_detail_title(format!("Boîtier {}", station.code).into());
        page.set_detail_subtitle(format!("n° {} · {park}", station.serial).into());
        page.set_detail_serial(station.serial.to_string().into());
        page.set_detail_kind(station.kind.into());
        page.set_detail_code(station.code.as_str().into());
        page.set_detail_park(park.into());
        page.set_detail_badge(badge(&state_label(station.state)));
        page.set_detail_since(format!("depuis le {}", station.state_since).into());
        page.set_detail_battery(battery_text(station).into());
        page.set_detail_charge(charge_text(station).into());
        page.set_detail_can_change_battery(station.state != StockState::Destroyed);
        page.set_detail_clock(clock_text(station).into());
        page.set_detail_measured(station.measured.as_deref().unwrap_or("—").into());
        page.set_detail_clock_limit(format!("À régler au-delà de {CLOCK_MAX_GAP} s").into());
        page.set_detail_note(if station.note.is_empty() { "Aucune note" } else { station.note }.into());
        page.set_actions(model(actions_of(station)));
        page.set_history(model(
            station.history.iter().map(|e| CardEntry { date: e.date.as_str().into(), text: e.text.as_str().into() }).collect(),
        ));
        let (question, button) = match self.to_confirm {
            Some(StockState::Lost) => ("Marquer ce boîtier perdu ?", "Marquer perdu"),
            Some(StockState::Destroyed) => ("Marquer ce boîtier détruit ?", "Marquer détruit"),
            _ => ("", ""),
        };
        page.set_confirm_question(question.into());
        page.set_confirm_button(button.into());
    }
}
