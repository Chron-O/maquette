// The Boîtiers tab: what is chosen, and the rows it shows. Reading a station is simulated
// from the Démo panel, on the test data only.

use crate::home::badge;
use crate::test_data::{Event, Kind, Label, Station, StationState, missed_control, punches_not_in_event};
use crate::{App, CourseMark, StationProblem, StationRow, Stations, TabItem, UsefulStation, model};
use slint::{ComponentHandle, SharedString};
use std::cmp::Ordering;

// The words of the prototype's sub-tabs (22/10), kept as one segment in the column.
const MODES: [&str; 2] = ["Contrôle", "Ramassage"];
const SPARE_PARK: &str = "Secours";

fn plural(count: usize, noun: &str) -> String {
    format!("{count} {noun}{}", if count > 1 { "s" } else { "" })
}

fn word(label: &str) -> TabItem {
    TabItem { label: label.into(), icon: Default::default() }
}

// French alphabetical order: accents and capitals do not count (charte-ui.md, Tableaux).
fn french_key(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .flat_map(|c| match c {
            'à' | 'â' | 'ä' | 'á' => vec!['a'],
            'é' | 'è' | 'ê' | 'ë' => vec!['e'],
            'î' | 'ï' | 'í' => vec!['i'],
            'ô' | 'ö' | 'ó' => vec!['o'],
            'ù' | 'û' | 'ü' | 'ú' => vec!['u'],
            'ç' => vec!['c'],
            'ÿ' => vec!['y'],
            'œ' => vec!['o', 'e'],
            'æ' => vec!['a', 'e'],
            other => vec![other],
        })
        .collect()
}

#[derive(Clone, Copy, PartialEq)]
enum SortKey {
    Need,
    Serial,
    State,
    Battery,
}

pub struct StationsState {
    collecting: bool,
    only_to_handle: bool,
    sort: SortKey,
    ascending: bool,
    search: String,
    selected: Option<usize>,
    reading: Option<usize>,
    // Settings « À la lecture » (prototype 22/10, bxSettings): no effect for now.
    check_clock: bool,
    check_battery: bool,
}

impl Default for StationsState {
    fn default() -> Self {
        StationsState {
            collecting: false,
            only_to_handle: false,
            sort: SortKey::Need,
            ascending: true,
            search: String::new(),
            selected: None,
            reading: None,
            check_clock: true,
            check_battery: true,
        }
    }
}

fn is_out(station: &Station) -> bool {
    matches!(station.state, StationState::Checked | StationState::ClockGap(_))
}

impl StationsState {
    pub fn select_mode(&mut self, mode: usize) {
        self.collecting = mode == 1;
        self.only_to_handle = false;
        self.reading = None;
    }

    pub fn select_filter(&mut self, filter: usize) {
        self.only_to_handle = filter == 1;
    }

    pub fn show_only_to_handle(&mut self) {
        self.only_to_handle = true;
    }

    pub fn select_row(&mut self, index: usize) {
        self.selected = if self.selected == Some(index) { None } else { Some(index) };
    }

    pub fn close_detail(&mut self) {
        self.selected = None;
    }

    pub fn set_search(&mut self, text: &str) {
        self.search = text.trim().to_lowercase();
    }

    pub fn sort_by(&mut self, column: i32) {
        let key = match column {
            1 => SortKey::Serial,
            2 => SortKey::State,
            3 => SortKey::Battery,
            _ => SortKey::Need,
        };
        if self.sort == key {
            self.ascending = !self.ascending;
        } else {
            self.sort = key;
            self.ascending = true;
        }
    }

    // The next station on the reader: one to check, or one back from the forest.
    pub fn read_next(&mut self, event: &mut Event, punches: u32) {
        let next = if self.collecting {
            event.stations.iter().position(is_out)
        } else {
            event.stations.iter().position(|s| s.state == StationState::ToTakeOut)
        };
        if let (true, Some(index)) = (self.collecting, next) {
            event.stations[index].state = StationState::Read(punches);
        }
        self.reading = next;
    }

    pub fn confirm_reading(&mut self, event: &mut Event) {
        let Some(station) = self.reading.take().and_then(|i| event.stations.get_mut(i)) else { return };
        station.state = match station.state {
            StationState::Read(punches) => StationState::Back(punches),
            _ => StationState::Checked,
        };
    }

    pub fn add_to_event(&mut self, event: &mut Event, index: usize) {
        if let Some(station) = event.stations.get_mut(index)
            && let StationState::Read(punches) = station.state
        {
            station.state = StationState::Back(punches);
        }
        if self.reading == Some(index) {
            self.reading = None;
        }
    }

    // In the alert while collecting: all the memories read so far.
    pub fn alert_action(&mut self, event: &mut Event) {
        if self.collecting {
            for index in 0..event.stations.len() {
                self.add_to_event(event, index);
            }
        } else {
            self.show_only_to_handle();
        }
    }

    pub fn state_label(&self, station: &Station) -> Label {
        match station.state {
            StationState::ToTakeOut => Label::new("À sortir", Kind::Inactive),
            StationState::ToProgram => Label::new("À programmer", Kind::ToCheck),
            StationState::Missing => Label::new("Manque un boîtier", Kind::Pm),
            StationState::Checked | StationState::ClockGap(_) if self.collecting => Label::new("Posé · pas rentré", Kind::Racing),
            StationState::Checked => Label::new("Contrôlé", Kind::Ok),
            StationState::ClockGap(gap) => Label::new(format!("Heure décalée +{gap} s"), Kind::ToCheck),
            StationState::Read(_) => Label::new("Mémoire lue", Kind::Racing),
            StationState::Back(_) => Label::new("Rentré", Kind::Ok),
        }
    }

    fn to_handle(&self, station: &Station) -> bool {
        match station.state {
            StationState::ToProgram | StationState::Missing | StationState::ClockGap(_) => !self.collecting,
            StationState::Read(_) => self.collecting,
            _ => false,
        }
    }

    fn in_filter(&self, station: &Station) -> bool {
        let filtered = !self.only_to_handle || if self.collecting { is_out(station) } else { self.to_handle(station) };
        let text = format!("{} {}", station.name, station.serial.map_or(String::new(), |s| s.to_string())).to_lowercase();
        filtered && self.search.split_whitespace().all(|word| text.contains(word))
    }

    // The order of the states: what needs a hand first, then the way a station goes.
    fn state_rank(&self, station: &Station) -> u8 {
        match station.state {
            StationState::Missing => 0,
            StationState::ToProgram => 1,
            StationState::ClockGap(_) if !self.collecting => 2,
            StationState::ToTakeOut => 3,
            StationState::Checked | StationState::ClockGap(_) => 4,
            StationState::Read(_) => 5,
            StationState::Back(_) => 6,
        }
    }

    fn battery_text(station: &Station) -> Option<&'static str> {
        match (station.serial, station.low_battery) {
            (None, _) => None,
            (_, true) => Some("Pile faible"),
            _ => Some("OK"),
        }
    }

    // Empty cells stay at the end, whichever way the column is sorted; at a tie, the order of the needs.
    fn sorted(&self, event: &Event) -> Vec<usize> {
        let mut rows: Vec<usize> = (0..event.stations.len()).filter(|i| self.in_filter(&event.stations[*i])).collect();
        let compare = |a: &usize, b: &usize| -> Option<Ordering> {
            let (x, y) = (&event.stations[*a], &event.stations[*b]);
            match self.sort {
                SortKey::Need => Some(a.cmp(b)),
                SortKey::State => Some(self.state_rank(x).cmp(&self.state_rank(y))),
                SortKey::Serial => match (x.serial, y.serial) {
                    (Some(p), Some(q)) => Some(p.cmp(&q)),
                    _ => None,
                },
                SortKey::Battery => match (Self::battery_text(x), Self::battery_text(y)) {
                    (Some(p), Some(q)) => Some(french_key(p).cmp(&french_key(q))),
                    _ => None,
                },
            }
        };
        let is_empty = |i: &usize| {
            let station = &event.stations[*i];
            match self.sort {
                SortKey::Serial | SortKey::Battery => station.serial.is_none(),
                _ => false,
            }
        };
        rows.sort_by(|a, b| {
            let order = match (is_empty(a), is_empty(b)) {
                (false, true) => Ordering::Less,
                (true, false) => Ordering::Greater,
                (true, true) => Ordering::Equal,
                (false, false) => {
                    let order = compare(a, b).unwrap_or(Ordering::Equal);
                    if self.ascending { order } else { order.reverse() }
                }
            };
            order.then_with(|| a.cmp(b))
        });
        rows
    }

    fn row(&self, event: &Event, index: usize) -> StationRow {
        let station = &event.stations[index];
        let (action, short) = match station.state {
            StationState::ToProgram if !self.collecting => ("Programmer…", "Régler"),
            StationState::Missing if !self.collecting => ("Choisir un boîtier…", "Régler"),
            StationState::ClockGap(_) if !self.collecting => ("Corriger les temps…", "Corriger"),
            StationState::Read(_) => ("Ajouter à la course", "Ajouter"),
            _ => ("", ""),
        };
        let courses = station.control.map_or(Vec::new(), |code| {
            event
                .courses
                .iter()
                .enumerate()
                .filter(|(_, c)| c.controls.contains(&code))
                .map(|(i, c)| CourseMark { letter: c.name.replace("Circuit ", "").chars().take(1).collect::<String>().into(), tone: i as i32 })
                .collect()
        });
        StationRow {
            index: index as i32,
            name: station.name.as_str().into(),
            courses: model(courses),
            serial: station.serial.map_or("—".into(), |s| s.to_string().into()),
            serial_long: station.serial.map_or("—".into(), |s| format!("n° {s} · {}", station.park).into()),
            badge: badge(&self.state_label(station)),
            battery: Self::battery_text(station).unwrap_or("—").into(),
            low_battery: station.low_battery,
            action: action.into(),
            action_short: short.into(),
            can_add: matches!(station.state, StationState::Read(_)),
            to_handle: self.to_handle(station),
            selected: self.selected == Some(index) || self.reading == Some(index),
        }
    }

    // The settings of the event's stations, from the stations themselves.
    fn settings(event: &Event) -> (usize, Vec<&str>, Vec<&str>) {
        let with_station = || event.stations.iter().filter(|s| s.serial.is_some());
        let spares = with_station().filter(|s| s.park == SPARE_PARK).count();
        let mut parks: Vec<&str> = Vec::new();
        for station in with_station() {
            if !parks.contains(&station.park) {
                parks.push(station.park);
            }
        }
        let specials: Vec<&str> = event
            .stations
            .iter()
            .filter(|s| s.control.is_none() && !s.name.starts_with("Départ") && !s.name.starts_with("Arrivée"))
            .map(|s| s.name.as_str())
            .collect();
        (spares, parks, specials)
    }

    // The stations worth reading to settle a doubt (lot1-securite.md § 2): the finish and the
    // starts while runners are out, the controls missing on cards read. Ticked once read.
    fn useful(event: &Event) -> Vec<UsefulStation> {
        let read = |station: &Station| matches!(station.state, StationState::Read(_) | StationState::Back(_));
        let line = |station: &Station, detail: String| UsefulStation {
            label: station.name.as_str().into(),
            detail: detail.into(),
            done: read(station),
        };
        let racing = |point: Option<u32>| {
            event
                .runners
                .iter()
                .filter(|r| r.is_racing())
                .filter(|r| {
                    point.is_none_or(|p| {
                        r.runs.iter().any(|run| run.result.is_none() && event.courses.get(run.course).is_some_and(|c| c.start_point == p))
                    })
                })
                .count()
        };
        let out = |count: usize| format!("{} encore en course", plural(count, "coureur"));
        let mut lines = Vec::new();
        for station in event.stations.iter().filter(|s| s.code.starts_with('F')) {
            let count = racing(None);
            if count > 0 {
                lines.push(line(station, out(count)));
            }
        }
        for station in event.stations.iter().filter(|s| s.code.starts_with('S')) {
            let count = station.code[1..].parse().ok().map_or(0, |point| racing(Some(point)));
            if count > 0 {
                lines.push(line(station, out(count)));
            }
        }
        // The missing controls, in the order of the needs.
        let mut missing: Vec<(u32, Vec<String>)> = Vec::new();
        for runner in &event.runners {
            for run in &runner.runs {
                let (Some((seconds, false)), Some(course)) = (run.result, event.courses.get(run.course)) else { continue };
                let Some(code) = course.controls.get(missed_control(seconds, course.controls.len())) else { continue };
                match missing.iter_mut().find(|(c, _)| c == code) {
                    Some((_, names)) => names.push(runner.name()),
                    None => missing.push((*code, vec![runner.name()])),
                }
            }
        }
        for station in &event.stations {
            let Some((_, names)) = station.control.and_then(|code| missing.iter().find(|(c, _)| *c == code)) else { continue };
            let who = if names.len() == 1 { names[0].clone() } else { plural(names.len(), "coureur") };
            lines.push(line(station, format!("manquant pour {who}")));
        }
        lines
    }

    pub fn show(&self, window: &App, event: Option<&Event>) {
        let page = window.global::<Stations>();
        let Some(event) = event else { return };
        let stations = &event.stations;
        let count = |test: &dyn Fn(&Station) -> bool| stations.iter().filter(|s| test(s)).count();

        let (spares, parks, specials) = Self::settings(event);
        let spares_text = if spares == 0 { "Aucun secours".to_string() } else { format!("{spares} secours") };
        let mut chips: Vec<SharedString> = vec![spares_text.into(), format!("Parcs : {}", parks.join(", ")).into()];
        if self.check_clock {
            chips.push("Heure vérifiée".into());
        }
        page.set_chips(model(chips));
        page.set_settings_spares(spares.to_string().into());
        page.set_settings_parks(model(parks.iter().map(|p| SharedString::from(*p)).collect()));
        page.set_settings_specials(model(specials.iter().map(|p| SharedString::from(*p)).collect()));
        page.set_settings_check_clock(self.check_clock);
        page.set_settings_check_battery(self.check_battery);
        page.set_modes(model(MODES.iter().map(|m| word(m)).collect()));
        page.set_active_mode(self.collecting as i32);
        page.set_search_text(self.search.as_str().into());
        page.set_sort_column(match self.sort {
            SortKey::Need => 0,
            SortKey::Serial => 1,
            SortKey::State => 2,
            SortKey::Battery => 3,
        });
        page.set_sort_ascending(self.ascending);

        let checked = count(&|s| is_out(s) || matches!(s.state, StationState::Read(_) | StationState::Back(_)));
        let out = count(&is_out);
        let back = count(&|s| matches!(s.state, StationState::Read(_) | StationState::Back(_)));
        let memories = count(&|s| matches!(s.state, StationState::Read(_)));
        let to_take_out = count(&|s| s.state == StationState::ToTakeOut);
        let to_program = count(&|s| s.state == StationState::ToProgram);
        let missing = count(&|s| s.state == StationState::Missing);
        let gaps = count(&|s| matches!(s.state, StationState::ClockGap(_)));

        let mut summary = format!("{} · {} contrôlé{}", plural(stations.len(), "besoin"), checked, if checked > 1 { "s" } else { "" });
        if self.collecting {
            summary += &format!(" · {} rentré{}", back, if back > 1 { "s" } else { "" });
        }
        page.set_summary(summary.into());

        if self.collecting {
            page.set_filters(model(vec![word("Tous"), word(&format!("Dehors · {out}"))]));
            page.set_gauge_title("Pas encore rentrés".into());
            page.set_gauge_count(out.to_string().into());
            page.set_gauge_total(format!("/ {} posés", out + back).into());
            page.set_gauge_share(if out + back == 0 { 0.0 } else { back as f32 / (out + back) as f32 });
            page.set_gauge_detail(SharedString::new());
            page.set_useful(model(Self::useful(event)));
        } else {
            let to_handle = to_program + missing + gaps;
            page.set_filters(model(vec![word("Tous"), word(&format!("À régler · {to_handle}"))]));
            page.set_gauge_title("Contrôle".into());
            page.set_gauge_count(checked.to_string().into());
            page.set_gauge_total(format!("/ {} contrôlés", stations.len()).into());
            page.set_gauge_share(if stations.is_empty() { 0.0 } else { checked as f32 / stations.len() as f32 });
            page.set_gauge_detail(if to_take_out > 0 { format!("{to_take_out} encore à sortir").into() } else { SharedString::new() });
            page.set_useful(model(Vec::new()));
        }
        page.set_active_filter(self.only_to_handle as i32);

        let mut problems: Vec<String> = Vec::new();
        if self.collecting {
            if memories > 0 {
                problems.push(format!("{} pas encore dans la course", plural(memories, "mémoire lue").replace("mémoire lues", "mémoires lues")));
            }
            page.set_alert_title(if memories > 0 { problems[0].as_str().into() } else { SharedString::new() });
            page.set_alert_button("Tout ajouter".into());
        } else {
            if to_program > 0 {
                problems.push(format!("{} à programmer", plural(to_program, "boîtier")));
            }
            if missing > 0 {
                problems.push(format!("{} sans boîtier", plural(missing, "besoin")));
            }
            if gaps > 0 {
                problems.push(plural(gaps, "heure décalée").replace("heure décalées", "heures décalées"));
            }
            let total = to_program + missing + gaps;
            page.set_alert_title(if total > 0 { format!("{} à régler", plural(total, "besoin")).into() } else { SharedString::new() });
            page.set_alert_button("Voir".into());
        }
        page.set_alert_count(if self.collecting { memories } else { to_program + missing + gaps }.to_string().into());
        page.set_problems(model(problems.iter().map(|p| StationProblem { label: p.as_str().into() }).collect()));

        page.set_rows(model(self.sorted(event).into_iter().map(|i| self.row(event, i)).collect()));

        // What was read, in big, in its own card above the table (prototype: bxBig).
        let reading = self.reading.and_then(|i| stations.get(i));
        page.set_reading(reading.is_some());
        if let Some(station) = reading {
            let serial = station.serial.map_or(String::new(), |s| s.to_string());
            page.set_reading_code(station.code.as_str().into());
            if let StationState::Read(punches) = station.state {
                page.set_reading_label("Rentré".into());
                page.set_reading_question(format!("Mémoire lue : {}", plural(punches as usize, "poinçon")).into());
                let missing = punches_not_in_event(punches);
                let detail = format!("{} ne {} pas encore dans la course", plural(missing, "poinçon"), if missing > 1 { "sont" } else { "est" });
                page.set_reading_detail(format!("Boîtier n° {serial} · {detail}").into());
                page.set_reading_detail_short(detail.into());
                page.set_reading_button("Ajouter à la course".into());
                page.set_reading_button_short("Ajouter à la course".into());
                page.set_reading_can_refuse(false);
            } else {
                page.set_reading_label("Code lu".into());
                page.set_reading_question(format!("Le numéro écrit sur le boîtier est-il {} ?", station.code).into());
                let battery = if station.low_battery { "pile faible" } else { "pile OK" };
                page.set_reading_detail(format!("Boîtier n° {serial} · heure 0 s · {battery} · mémoire vide").into());
                page.set_reading_detail_short(format!("Boîtier n° {serial}").into());
                page.set_reading_button("Oui (Entrée)".into());
                page.set_reading_button_short("Oui".into());
                page.set_reading_can_refuse(true);
            }
        }

        let selected = self.selected.and_then(|i| stations.get(i));
        page.set_detail_open(selected.is_some());
        if let Some(station) = selected {
            page.set_detail_index(self.selected.unwrap_or_default() as i32);
            page.set_detail_name(station.name.as_str().into());
            page.set_detail_subtitle(badge(&self.state_label(station)).word);
            page.set_detail_has_station(station.serial.is_some());
            page.set_detail_serial(station.serial.map_or(SharedString::new(), |s| format!("n° {s}").into()));
            page.set_detail_park(station.park.into());
            page.set_detail_low_battery(station.low_battery);
            page.set_detail_gap(match station.state {
                StationState::ClockGap(gap) => format!("+{gap} s · à corriger").into(),
                StationState::ToTakeOut => "—".into(),
                _ => "0 s".into(),
            });
            page.set_detail_punches(match station.state {
                StationState::Read(punches) | StationState::Back(punches) => plural(punches as usize, "poinçon").into(),
                _ => "aucun".into(),
            });
        }
    }
}
