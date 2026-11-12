// The Participants tab: what is chosen, and the rows it shows. Display only for now.

use crate::home::badge;
use crate::test_data::{Event, Format, Kind, Label, Run, Runner, format_clock, format_time};
use crate::{ActiveFilter, App, ClubRow, FilterChoice, LoanRow, StatBlock, FilterGroup, Passage, ParticipantRow, Participants, PanelSet, SequenceStop, model};
use slint::{ComponentHandle, SharedString};
use std::cell::RefCell;
use std::collections::BTreeSet;

fn plural(count: usize, noun: &str) -> String {
    format!("{count} {noun}{}", if count > 1 { "s" } else { "" })
}

#[derive(Clone, Copy, PartialEq)]
enum SortKey {
    State,
    Name,
    Club,
    Category,
    Card,
    Start,
}

// The filter groups, in the Filtres bubble (brief zones-onglets § 5). One choice per group.
const STATE: usize = 0;
const COURSE: usize = 1;
const CARD: usize = 2;
const CHECK: usize = 3;
const CLUB: usize = 4;   // last (Robin, 11/11)
// Every state of a runner (Robin, 11/11): the word in the bubble, then the word of its label.
const STATES: [(&str, &str); 7] = [
    ("Pas encore parti", "Pas encore parti"),
    ("En course", "En course"),
    ("Arrivé", "Arrivé"),
    ("PM", "PM"),
    ("Abandon", "Abandon"),
    ("Disqualifié", "Disqualifié"),
    ("Absent", "Absent"),
];
const CARDS: [&str; 1] = ["Sans puce"];

// The panel (Robin, 11/11): the participants, or one of two tables of their own.
const ALL: usize = 0;   // 1: the clubs; 2: the loan cards
const CHECKS: [&str; 2] = ["Doublons possibles", "Catégorie inconnue"];

pub struct ParticipantsState {
    sort: SortKey,
    ascending: bool,
    filters: [Option<usize>; 4],   // the choice of each group, if any
    club: Option<String>,          // the club filter, last group of the bubble
    club_search: String,           // the field above the clubs, in the Filtres bubble
    shown_clubs: RefCell<Vec<String>>,   // the clubs the bubble shows, set by show
    returned: Vec<usize>,          // the loan cards given back, by runner
    set: usize,                    // the choice of the panel
    search: String,
    selected: Option<usize>,
    problems: RefCell<Vec<(usize, usize)>>, // the filter of each line of the alert, set by show
}

impl Default for ParticipantsState {
    fn default() -> Self {
        ParticipantsState {
            sort: SortKey::Name,
            ascending: true,
            filters: [None; 4],
            club: None,
            club_search: String::new(),
            shown_clubs: RefCell::new(Vec::new()),
            returned: Vec::new(),
            set: ALL,
            search: String::new(),
            selected: None,
            problems: RefCell::new(Vec::new()),
        }
    }
}

fn card_text(runner: &Runner) -> String {
    runner.card.map_or("—".into(), |card| card.to_string())
}

// The card, and whether it is a loan (participants-resultats.md § 1).
pub fn card_label(runner: &Runner) -> String {
    match runner.card {
        Some(card) if runner.lent => format!("{card} · prêtée"),
        _ => card_text(runner),
    }
}

fn category_text(runner: &Runner) -> String {
    if runner.runs.len() > 1 {
        format!("{} · {} passages", runner.category, runner.runs.len())
    } else {
        runner.category.clone()
    }
}

// One course colour per passage, in passage order.
fn tones(runner: &Runner) -> slint::ModelRc<i32> {
    model(runner.runs.iter().map(|r| r.course as i32).collect())
}

fn passage_label(run: &Run) -> Label {
    match run.result {
        Some((_, true)) => Label { word: "OK".into(), kind: Kind::Ok },
        Some((_, false)) => Label { word: "PM".into(), kind: Kind::Pm },
        None => Label::default(),
    }
}

impl ParticipantsState {
    pub fn select_row(&mut self, index: usize) {
        self.selected = if self.selected == Some(index) { None } else { Some(index) };
    }

    // « Voir ses participants » on a club, or « Filtrer sur ce club » in a detail: all the
    // participants, with that club as a filter (Robin, 11/11).
    pub fn show_club(&mut self, name: &str) {
        self.club = Some(name.to_string());
        self.set = ALL;
        self.selected = None;
    }

    pub fn search_filter(&mut self, text: &str) {
        self.club_search = text.trim().to_lowercase();
    }

    // « Rendre » on a loan card (Robin, 11/11): it is back at the park.
    pub fn give_back(&mut self, runner: usize) {
        if !self.returned.contains(&runner) {
            self.returned.push(runner);
        }
    }

    // A block of figures at the bottom of the panel: its filter, alone, on all the participants;
    // a second click takes it away. Group -1: no filter; -2: the loan cards (Robin, 11/11).
    pub fn pick_stat(&mut self, group: i32, choice: usize) {
        match group {
            -2 => self.set = 2,
            -1 => {
                self.set = ALL;
                self.clear_filters();
            }
            g => {
                let g = g as usize;
                let again = self.filters.get(g).copied().flatten() == Some(choice);
                self.clear_filters();
                self.set = ALL;
                if !again && g < self.filters.len() {
                    self.filters[g] = Some(choice);
                }
            }
        }
        self.selected = None;
    }

    pub fn select_set(&mut self, set: usize) {
        self.set = set;
        self.selected = None;
    }

    pub fn close_detail(&mut self) {
        self.selected = None;
    }

    // A second click on the chosen choice takes it away.
    pub fn toggle_filter(&mut self, group: usize, choice: usize) {
        if group == CLUB {
            let name = self.shown_clubs.borrow().get(choice).cloned();
            self.club = if name.is_none() || self.club == name { None } else { name };
            return;
        }
        if let Some(slot) = self.filters.get_mut(group) {
            *slot = if *slot == Some(choice) { None } else { Some(choice) };
        }
    }

    pub fn clear_filters(&mut self) {
        self.filters = [None; 4];
        self.club = None;
    }

    // « Voir » on a line of the alert: its filter, alone.
    pub fn show_problem(&mut self, index: usize) {
        let problem = self.problems.borrow().get(index).copied();
        if let Some((group, choice)) = problem {
            self.clear_filters();
            self.filters[group] = Some(choice);
        }
    }

    pub fn set_search(&mut self, text: &str) {
        self.search = text.trim().to_lowercase();
    }

    pub fn sort_by(&mut self, column: i32) {
        let key = match column {
            0 => SortKey::State,
            2 => SortKey::Club,
            3 => SortKey::Category,
            4 => SortKey::Card,
            5 => SortKey::Start,
            _ => SortKey::Name,
        };
        if self.sort == key {
            self.ascending = !self.ascending;
        } else {
            self.sort = key;
            self.ascending = true;
        }
    }

    fn matches(&self, runner: &Runner, duplicate: bool) -> bool {
        let in_club = self.club.as_deref().is_none_or(|c| runner.club == c);
        let state = self.filters[STATE].is_none_or(|choice| runner.label().word == STATES[choice].1);
        let course = self.filters[COURSE].is_none_or(|c| runner.runs.iter().any(|r| r.course == c));
        let card = self.filters[CARD].is_none() || runner.card.is_none();
        let check = match self.filters[CHECK] {
            None => true,
            Some(0) => duplicate,
            Some(_) => runner.unknown_category,
        };
        let text = format!("{} {} {}", runner.name(), runner.club, card_text(runner)).to_lowercase();
        in_club && state && course && card && check && self.search.split_whitespace().all(|word| text.contains(word))
    }

    fn sorted(&self, event: &Event) -> Vec<usize> {
        let names: Vec<String> = event.runners.iter().map(|r| r.name()).collect();
        let duplicate = |i: usize| names.iter().filter(|n| **n == names[i]).count() > 1;
        let mut rows: Vec<usize> =
            (0..event.runners.len()).filter(|i| self.matches(&event.runners[*i], duplicate(*i))).collect();
        rows.sort_by_key(|i| {
            let runner = &event.runners[*i];
            let key = match self.sort {
                SortKey::State => runner.label().word,
                SortKey::Name => String::new(),
                SortKey::Club => runner.club.to_string(),
                SortKey::Category => runner.category.clone(),
                SortKey::Card => card_text(runner),
                SortKey::Start => format!("{:06}", runner.start),
            };
            (key, runner.name())
        });
        if !self.ascending {
            rows.reverse();
        }
        rows
    }

    pub fn show(&self, window: &App, event: Option<&Event>, cards_free: u32) {
        let page = window.global::<Participants>();
        let Some(event) = event else { return };
        let runners = &event.runners;

        page.set_chips(model(vec!["Inscription par catégorie".into(), "Sans dossards".into(), "Champs libres : repas".into()]));

        let lent: Vec<&Runner> = runners.iter().filter(|r| r.lent).collect();

        let without_card = runners.iter().filter(|r| r.card.is_none()).count();
        // « · N passages » only when someone runs more than once (participants-resultats.md § 1).
        let passage_count: usize = runners.iter().map(|r| r.runs.len()).sum();
        let people = if passage_count > runners.len() {
            format!("{} · {}", plural(runners.len(), "participant"), plural(passage_count, "passage"))
        } else {
            plural(runners.len(), "participant")
        };
        let clubs: BTreeSet<&str> = runners.iter().map(|r| r.club).collect();
        page.set_sets(model(vec![
            PanelSet { name: "Tous".into(), count: runners.len().to_string().into(), detail: SharedString::new() },
            PanelSet { name: "Clubs".into(), count: clubs.len().to_string().into(), detail: SharedString::new() },
            PanelSet {
                name: "Puces prêtées".into(),
                count: lent.len().to_string().into(),
                detail: format!("{} au parc", cards_free + self.returned.len() as u32).into(),
            },
        ]));
        page.set_active_set(self.set as i32);
        // The loan cards: who has one, and whether it must come back now.
        page.set_loans(model(
            runners
                .iter()
                .enumerate()
                .filter(|(_, r)| r.lent)
                .map(|(i, r)| LoanRow {
                    index: i as i32,
                    card: card_text(r).into(),
                    family: r.family.into(),
                    first: r.first.into(),
                    club: r.club.into(),
                    badge: badge(&r.label()),
                    back: if self.returned.contains(&i) {
                        "Rendue".into()
                    } else if r.is_back() {
                        "À rendre".into()
                    } else {
                        SharedString::new()
                    },
                    returned: self.returned.contains(&i),
                })
                .collect(),
        ));
        page.set_clubs(model(
            clubs
                .iter()
                .map(|club| {
                    let members: Vec<&Runner> = runners.iter().filter(|r| r.club == *club).collect();
                    ClubRow {
                        name: (*club).into(),
                        entrants: members.len().to_string().into(),
                        racing: members.iter().filter(|r| r.is_racing()).count().to_string().into(),
                        back: members.iter().filter(|r| r.started && !r.is_racing()).count().to_string().into(),
                        lent: members.iter().filter(|r| r.lent).count().to_string().into(),
                    }
                })
                .collect(),
        ));
        // The band counts what is shown, not what the panel says (Robin, 11/11).
        let shown = self.sorted(event);
        let filtered = shown.len() < runners.len();
        let shown_clubs: BTreeSet<&str> = shown.iter().map(|i| runners[*i].club).collect();
        let shown_without = shown.iter().filter(|i| runners[**i].card.is_none()).count();
        let shown_lent = shown.iter().filter(|i| runners[**i].lent).count();
        page.set_summary(
            if filtered {
                format!(
                    "{} sur {} · {} · {} sans puce",
                    plural(shown.len(), "participant"),
                    runners.len(),
                    plural(shown_clubs.len(), "club"),
                    shown_without
                )
            } else {
                format!(
                    "{} · {} · {} sans puce · {} prêtée{}",
                    people,
                    plural(clubs.len(), "club"),
                    without_card,
                    shown_lent,
                    if shown_lent > 1 { "s" } else { "" }
                )
            }
            .into(),
        );
        page.set_summary_short(
            if filtered { format!("{} sur {}", plural(shown.len(), "participant"), runners.len()) } else { format!("{people} · {without_card} sans puce") }
                .into(),
        );

        // The blocks of figures, the most useful first for the moment of the race (Robin, 11/11):
        // before, who is entered; during, who is to start, out, back; after, what went wrong.
        let count = |word: &str| runners.iter().filter(|r| r.label().word == word).count();
        let moment = if runners.iter().all(|r| !r.started) {
            0
        } else if runners.iter().any(|r| r.is_racing() || (!r.started && r.label().word == "Pas encore parti")) {
            1
        } else {
            2
        };
        let loans_out = lent.len().saturating_sub(self.returned.len());
        // (label, count, group, choice, rank before / during / after; 9: not then)
        let all: [(&str, usize, i32, usize, [u8; 3]); 9] = [
            ("Inscrits", runners.len(), -1, 0, [1, 9, 9]),
            ("Sans puce", without_card, CARD as i32, 0, [2, 4, 9]),
            ("Puces prêtées", loans_out, -2, 0, [3, 5, 4]),
            ("À partir", count("Pas encore parti"), STATE as i32, 0, [9, 1, 9]),
            ("En forêt", count("En course"), STATE as i32, 1, [9, 2, 9]),
            ("Rentrés", count("Arrivé"), STATE as i32, 2, [9, 3, 9]),
            ("PM", count("PM"), STATE as i32, 3, [9, 6, 1]),
            ("Absents", count("Absent"), STATE as i32, 6, [9, 7, 2]),
            ("Abandons", count("Abandon"), STATE as i32, 4, [9, 9, 3]),
        ];
        let mut blocks: Vec<_> = all.iter().filter(|b| b.4[moment] < 9 && b.1 > 0).collect();
        blocks.sort_by_key(|b| b.4[moment]);
        page.set_stats(model(
            blocks
                .iter()
                .take(4)
                .map(|(label, count, group, choice, _)| StatBlock {
                    label: (*label).into(),
                    count: count.to_string().into(),
                    group: *group,
                    choice: *choice as i32,
                    active: match *group {
                        -2 => self.set == 2,
                        -1 => false,
                        g => self.set == ALL && self.filters[g as usize] == Some(*choice),
                    },
                })
                .collect(),
        ));

        let names: Vec<String> = runners.iter().map(|r| r.name()).collect();
        let duplicates = names.iter().enumerate().filter(|(i, n)| names[..*i].contains(n)).count();
        let unknown = runners.iter().filter(|r| r.unknown_category).count();
        // Each line of the alert, with the filter its « Voir » applies.
        let mut problems: Vec<(String, usize, usize)> = Vec::new();
        if duplicates > 0 {
            problems.push((plural(duplicates, "doublon possible").replace("doublon possibles", "doublons possibles"), CHECK, 0));
        }
        if without_card > 0 {
            problems.push((format!("{} sans puce", plural(without_card, "inscrit")), CARD, 0));
        }
        if unknown > 0 {
            problems.push((plural(unknown, "catégorie inconnue").replace("catégorie inconnues", "catégories inconnues"), CHECK, 1));
        }
        *self.problems.borrow_mut() = problems.iter().map(|(_, group, choice)| (*group, *choice)).collect();
        page.set_alert_count(problems.len().to_string().into());
        page.set_alert_title(
            if problems.is_empty() {
                SharedString::new()
            } else {
                format!("{} à vérifier", plural(problems.len(), "point")).into()
            },
        );
        page.set_problems(model(problems.iter().map(|p| SharedString::from(p.0.as_str())).collect()));

        let courses: Vec<(String, i32)> = event.courses.iter().enumerate().map(|(i, c)| (c.name.clone(), i as i32)).collect();
        let groups: [(&str, Vec<(String, i32)>); 4] = [
            ("État", STATES.iter().map(|s| (s.0.to_string(), -1)).collect()),
            ("Circuit", courses),
            ("Puce", CARDS.iter().map(|s| (s.to_string(), -1)).collect()),
            ("À vérifier", CHECKS.iter().map(|s| (s.to_string(), -1)).collect()),
        ];
        // The clubs last, with a field above them: a long list (Robin, 11/11).
        let shown_clubs: Vec<String> = clubs
            .iter()
            .filter(|c| c.to_lowercase().contains(&self.club_search))
            .map(|c| c.to_string())
            .collect();
        let club_group = FilterGroup {
            search: self.club_search.as_str().into(),
            searchable: true,
            key: CLUB as i32,
            title: "Club".into(),
            choices: model(
                shown_clubs
                    .iter()
                    .map(|c| FilterChoice { label: c.as_str().into(), tone: -1, active: self.club.as_deref() == Some(c.as_str()) })
                    .collect(),
            ),
        };
        *self.shown_clubs.borrow_mut() = shown_clubs;
        page.set_filter_groups(model(
            groups
                .iter()
                .enumerate()
                .map(|(g, (title, choices))| FilterGroup {
                    search: SharedString::new(),
                    searchable: false,
                    key: g as i32,
                    title: (*title).into(),
                    choices: model(
                        choices
                            .iter()
                            .enumerate()
                            .map(|(c, (label, tone))| FilterChoice {
                                label: label.as_str().into(),
                                tone: *tone,
                                active: self.filters[g] == Some(c),
                            })
                            .collect(),
                    ),
                })
                .chain(std::iter::once(club_group))
                .collect(),
        ));
        let club_filter = self.club.as_ref().map(|club| ActiveFilter {
            label: format!("Club : {club}").into(),
            tone: -1,
            group: CLUB as i32,
            // Its place in the bubble's list, or past its end when the field hides it: a click
            // on the cross takes it away all the same.
            choice: self.shown_clubs.borrow().iter().position(|c| c == club).unwrap_or(usize::MAX >> 1) as i32,
        });
        page.set_active_filters(model(
            groups
                .iter()
                .enumerate()
                .filter_map(|(g, (_, choices))| {
                    let c = self.filters[g]?;
                    let (label, tone) = choices.get(c)?;
                    Some(ActiveFilter { label: label.as_str().into(), tone: *tone, group: g as i32, choice: c as i32 })
                })
                .chain(club_filter)
                .collect(),
        ));
        page.set_search_text(self.search.as_str().into());
        page.set_sort_column(match self.sort {
            SortKey::State => 0,
            SortKey::Name => 1,
            SortKey::Club => 2,
            SortKey::Category => 3,
            SortKey::Card => 4,
            SortKey::Start => 5,
        });
        page.set_sort_ascending(self.ascending);

        page.set_rows(model(
            self.sorted(event)
                .into_iter()
                .map(|i| {
                    let runner = &runners[i];
                    let time = runner.runs.iter().rev().find_map(|r| r.result).map(|(seconds, _)| format_time(seconds));
                    ParticipantRow {
                        index: i as i32,
                        badge: badge(&runner.label()),
                        family: runner.family.into(),
                        first: runner.first.into(),
                        club: runner.club.into(),
                        category: category_text(runner).into(),
                        category_short: runner.category.as_str().into(),
                        tones: tones(runner),
                        card: card_label(runner).into(),
                        no_card: runner.card.is_none(),
                        start: format_clock(runner.start).into(),
                        result: time.unwrap_or_else(|| format_clock(runner.start)).into(),
                        selected: self.selected == Some(i),
                    }
                })
                .collect(),
        ));

        let selected = self.selected.and_then(|i| runners.get(i));
        page.set_detail_open(selected.is_some());
        if let Some(runner) = selected {
            page.set_detail_name(runner.name().into());
            page.set_detail_category(category_text(runner).into());
            page.set_detail_tones(tones(runner));
            page.set_detail_club(runner.club.into());
            page.set_detail_card(card_label(runner).into());
            page.set_detail_start(format_clock(runner.start).into());
            page.set_passages(model(
                runner
                    .runs
                    .iter()
                    .map(|run| {
                        let course = &event.courses[run.course];
                        let ordered = course.format == Format::Imposed;
                        Passage {
                            course: format!("{} · {}", course.name, plural(course.controls.len(), "poste")).into(),
                            tone: run.course as i32,
                            ordered,
                            stops: model(
                                course
                                    .controls
                                    .iter()
                                    .enumerate()
                                    .map(|(n, code)| SequenceStop {
                                        code: code.to_string().into(),
                                        rank: if ordered { n as i32 + 1 } else { -1 },
                                    })
                                    .collect(),
                            ),
                            time: run.result.map_or(SharedString::new(), |(seconds, _)| format_time(seconds).into()),
                            badge: badge(&passage_label(run)),
                        }
                    })
                    .collect(),
            ));
        }
    }
}
