// The Participants tab: what is chosen, and the rows it shows. Display only for now.

use crate::home::badge;
use crate::test_data::{Event, Format, Kind, Label, Run, Runner, format_clock, format_time};
use crate::{App, Passage, ParticipantFilter, ParticipantRow, Participants, SequenceStop, model};
use slint::{ComponentHandle, SharedString};
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

pub struct ParticipantsState {
    sort: SortKey,
    ascending: bool,
    filter: usize,      // 0 all, otherwise a course + 1
    search: String,
    selected: Option<usize>,
}

impl Default for ParticipantsState {
    fn default() -> Self {
        ParticipantsState { sort: SortKey::Name, ascending: true, filter: 0, search: String::new(), selected: None }
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

    pub fn close_detail(&mut self) {
        self.selected = None;
    }

    pub fn select_filter(&mut self, filter: usize) {
        self.filter = filter;
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

    fn matches(&self, runner: &Runner) -> bool {
        let in_filter = self.filter == 0 || runner.runs.iter().any(|r| r.course + 1 == self.filter);
        let text = format!("{} {} {}", runner.name(), runner.club, card_text(runner)).to_lowercase();
        in_filter && self.search.split_whitespace().all(|word| text.contains(word))
    }

    fn sorted(&self, event: &Event) -> Vec<usize> {
        let mut rows: Vec<usize> = (0..event.runners.len()).filter(|i| self.matches(&event.runners[*i])).collect();
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
        page.set_lent_count(lent.len().to_string().into());
        page.set_lent_detail(format!("en circulation · {cards_free} au parc").into());
        page.set_lent_name(lent.first().map_or(SharedString::new(), |r| format!("{} · {}", r.name(), card_text(r)).into()));

        let without_card = runners.iter().filter(|r| r.card.is_none()).count();
        // « · N passages » only when someone runs more than once (participants-resultats.md § 1).
        let passage_count: usize = runners.iter().map(|r| r.runs.len()).sum();
        let people = if passage_count > runners.len() {
            format!("{} · {}", plural(runners.len(), "participant"), plural(passage_count, "passage"))
        } else {
            plural(runners.len(), "participant")
        };
        let clubs: BTreeSet<&str> = runners.iter().map(|r| r.club).collect();
        page.set_summary(
            format!(
                "{} · {} · {} sans puce · {} prêtée{}",
                people,
                plural(clubs.len(), "club"),
                without_card,
                lent.len(),
                if lent.len() > 1 { "s" } else { "" }
            )
            .into(),
        );

        page.set_summary_short(format!("{people} · {without_card} sans puce").into());

        let names: Vec<String> = runners.iter().map(|r| r.name()).collect();
        let duplicates = names.iter().enumerate().filter(|(i, n)| names[..*i].contains(n)).count();
        let unknown = runners.iter().filter(|r| r.unknown_category).count();
        let mut problems: Vec<String> = Vec::new();
        if duplicates > 0 {
            problems.push(plural(duplicates, "doublon possible").replace("doublon possibles", "doublons possibles"));
        }
        if without_card > 0 {
            problems.push(format!("{} sans puce", plural(without_card, "inscrit")));
        }
        if unknown > 0 {
            problems.push(plural(unknown, "catégorie inconnue").replace("catégorie inconnues", "catégories inconnues"));
        }
        page.set_alert_count(problems.len().to_string().into());
        page.set_alert_title(
            if problems.is_empty() {
                SharedString::new()
            } else {
                format!("{} à vérifier", plural(problems.len(), "point")).into()
            },
        );
        page.set_problems(model(problems.iter().map(|p| SharedString::from(p.as_str())).collect()));

        let mut filters = vec![ParticipantFilter { label: "Tous".into(), tone: -1 }];
        filters.extend(event.courses.iter().enumerate().map(|(i, c)| {
            let count = runners.iter().filter(|r| r.runs.iter().any(|run| run.course == i)).count();
            ParticipantFilter { label: format!("{} · {count}", c.name.replace("Circuit ", "")).into(), tone: i as i32 }
        }));
        page.set_filters(model(filters));
        page.set_active_filter(self.filter as i32);
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
