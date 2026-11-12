// The Résultats tab: one ranking at a time, by category or by course. Each ranking is worked
// out again from the test data at every reading.

use crate::home::badge;
use crate::test_data::{Event, Kind, Label, Outcome, format_time};
use crate::{ActiveFilter, App, FilterChoice, FilterGroup, LiveFilter, ResultColumn, ResultRow, Results, StatBlock, model};
use slint::{ComponentHandle, SharedString};
use std::collections::BTreeMap;

const MODES: [&str; 2] = ["Par catégorie", "Par circuit"];

// The views of the panel (lot6a-analyse.md § 1).
const VIEWS: [&str; 3] = ["Classements", "Analyse", "Remise des prix"];
// The state filter of the columns (Robin, 12/11): what a row says.
const STATES: [&str; 5] = ["Classés", "En forêt", "PM", "Abandons", "Disqualifiés"];
// The filter groups of the columns.
const RANKING: usize = 0;
const STATE: usize = 1;

#[derive(Default)]
pub struct ResultsState {
    by_course: bool,
    ranking: usize,
    category: usize,    // 0 all, otherwise a category of the course + 1
    search: String,
    ranking_search: String,
    view: usize,                 // the panel: Classements, Analyse, Remise des prix
    column: Option<usize>,       // Filtres › the one ranking shown as a column
    state: Option<usize>,        // Filtres › État
}

// One ranking: its name, its color, and the runs it holds.
fn place_text(place: usize) -> String {
    if place == 1 { "1er".into() } else { format!("{place}e") }
}

struct Ranking {
    label: String,
    tone: i32,
    title: String,
    entries: Vec<(usize, usize)>,   // runner, run
}

fn rankings(event: &Event, by_course: bool) -> Vec<Ranking> {
    if by_course {
        return event
            .courses
            .iter()
            .enumerate()
            .map(|(course, c)| Ranking {
                label: c.name.replace("Circuit ", ""),
                tone: course as i32,
                title: c.name.clone(),
                entries: event
                    .runners
                    .iter()
                    .enumerate()
                    .flat_map(|(i, r)| r.runs.iter().enumerate().map(move |(n, run)| (i, n, run.course)))
                    .filter(|(_, _, run_course)| *run_course == course)
                    .map(|(i, n, _)| (i, n))
                    .collect(),
            })
            .collect();
    }
    // By category, a runner's first course decides: a category stays on one course.
    let mut categories: BTreeMap<&str, Vec<(usize, usize)>> = BTreeMap::new();
    for (i, runner) in event.runners.iter().enumerate() {
        if let Some(first) = runner.runs.first() {
            let entries = categories.entry(runner.category.as_str()).or_default();
            entries.extend(runner.runs.iter().enumerate().filter(|(_, run)| run.course == first.course).map(|(n, _)| (i, n)));
        }
    }
    categories
        .into_iter()
        .map(|(category, entries)| {
            let course = event.runners[entries[0].0].runs[entries[0].1].course;
            Ranking {
                label: category.to_string(),
                tone: course as i32,
                title: format!("{category} · {}", event.courses[course].name),
                entries,
            }
        })
        .collect()
}

impl ResultsState {
    pub fn select_view(&mut self, view: usize) {
        self.view = view;
    }

    // A block of figures: its state filter alone; again, it goes away (Robin, 12/11).
    pub fn pick_stat(&mut self, choice: usize) {
        self.view = 0;
        self.column = None;
        self.state = if self.state == Some(choice) { None } else { Some(choice) };
    }

    // The Filtres bubble of the columns: group 0 the rankings, group 1 the states.
    pub fn toggle_column_filter(&mut self, group: usize, choice: usize) {
        let slot = if group == RANKING { &mut self.column } else { &mut self.state };
        *slot = if *slot == Some(choice) { None } else { Some(choice) };
    }

    pub fn clear_column_filters(&mut self) {
        self.column = None;
        self.state = None;
    }

    pub fn select_mode(&mut self, mode: usize) {
        self.by_course = mode == 1;
        self.ranking = 0;
        self.category = 0;
        self.column = None;
        self.ranking_search.clear();
    }

    pub fn select_ranking(&mut self, ranking: usize) {
        self.ranking = ranking;
        self.category = 0;
    }

    pub fn select_category(&mut self, category: usize) {
        self.category = category;
    }

    // The Filtres bubble: one group, the categories of a course ranking.
    pub fn toggle_filter(&mut self, _group: usize, choice: usize) {
        self.category = if self.category == choice + 1 { 0 } else { choice + 1 };
    }

    pub fn clear_filters(&mut self) {
        self.category = 0;
    }

    pub fn set_search(&mut self, text: &str) {
        self.search = text.trim().to_lowercase();
    }

    pub fn set_ranking_search(&mut self, text: &str) {
        self.ranking_search = text.trim().to_lowercase();
    }

    pub fn show(&self, window: &App, event: Option<&Event>, selected_runner: Option<usize>) {
        let page = window.global::<Results>();
        let Some(event) = event else { return };
        let all = rankings(event, self.by_course);

        page.set_modes(model(MODES.iter().map(|m| (*m).into()).collect()));
        page.set_views(model(VIEWS.iter().map(|v| (*v).into()).collect()));
        page.set_active_view(self.view as i32);
        self.show_columns(&page, event, &all, selected_runner);
        page.set_active_mode(self.by_course as i32);
        page.set_ranking_search_text(self.ranking_search.as_str().into());
        page.set_search_text(self.search.as_str().into());

        let back = event.runners.iter().filter(|r| r.is_back()).count();
        let racing = event.runners.iter().filter(|r| r.is_racing()).count();
        let mut chips = vec![SharedString::from(MODES[self.by_course as usize])];
        if racing > 0 {
            chips.push("Résultats provisoires".into());
        }
        page.set_chips(model(chips));
        let arrived = format!("{back} arrivé{}", if back > 1 { "s" } else { "" });
        // The band of the prototype: no count of rankings; on a phone, without the withdrawals.
        let retired = event.runners.iter().filter(|r| r.outcome == Outcome::Retired).count();
        page.set_summary_short(format!("{arrived} · {racing} en course").into());
        page.set_summary(format!("{arrived} · {racing} en course · {retired} abandon{}", if retired > 1 { "s" } else { "" }).into());

        let arrived_count = |ranking: &Ranking| {
            ranking.entries.iter().filter(|(i, n)| event.runners[*i].runs[*n].result.is_some()).count()
        };
        page.set_rankings(model(
            all.iter()
                .enumerate()
                .filter(|(_, r)| self.ranking_search.is_empty() || r.title.to_lowercase().contains(&self.ranking_search))
                .map(|(i, r)| crate::ResultRanking {
                    index: i as i32,
                    label: r.label.as_str().into(),
                    tone: r.tone,
                    count: format!("{} / {}", arrived_count(r), r.entries.len()).into(),
                })
                .collect(),
        ));
        page.set_ranking_chips(model(
            all.iter().map(|r| LiveFilter { label: r.label.as_str().into(), tone: r.tone, detail: SharedString::new() }).collect(),
        ));
        page.set_active_ranking(self.ranking as i32);

        let Some(ranking) = all.get(self.ranking) else {
            page.set_title("".into());
            page.set_title_tone(-1);
            page.set_state("".into());
            page.set_category_chips(model(Vec::new()));
            page.set_filter_groups(model(Vec::new()));
            page.set_active_filters(model(Vec::new()));
            page.set_rows(model(Vec::new()));
            page.set_empty("Aucun classement".into());
            return;
        };
        page.set_title(ranking.title.as_str().into());
        page.set_title_tone(ranking.tone);
        page.set_by_course(self.by_course);
        let done = arrived_count(ranking);
        let in_progress = done < ranking.entries.len();
        page.set_state(
            format!("{}{done} arrivé{} sur {}", if in_progress { "Provisoire · " } else { "" }, if done > 1 { "s" } else { "" }, ranking.entries.len()).into(),
        );

        // By course, a chip per category keeps only theirs: the place stays the one of the course.
        let mut categories: Vec<&str> = if self.by_course {
            ranking.entries.iter().map(|(i, _)| event.runners[*i].category.as_str()).collect()
        } else {
            Vec::new()
        };
        categories.sort_unstable();
        categories.dedup();
        let mut category_chips = Vec::new();
        if !categories.is_empty() {
            category_chips.push(LiveFilter { label: "Toutes".into(), tone: -1, detail: SharedString::new() });
            category_chips.extend(categories.iter().map(|c| LiveFilter { label: (*c).into(), tone: -1, detail: SharedString::new() }));
        }
        page.set_category_chips(model(category_chips));
        page.set_active_category(self.category as i32);
        // The same filter in the Filtres button: by course only, as the categories.
        page.set_filter_groups(model(if categories.is_empty() {
            Vec::new()
        } else {
            vec![FilterGroup {
                search: SharedString::new(),
                searchable: false,
                key: 0,
                title: "Catégorie".into(),
                choices: model(
                    categories
                        .iter()
                        .enumerate()
                        .map(|(c, name)| FilterChoice { label: (*name).into(), tone: -1, active: self.category == c + 1 })
                        .collect(),
                ),
            }]
        }));
        page.set_active_filters(model(
            self.category
                .checked_sub(1)
                .and_then(|c| categories.get(c).map(|name| ActiveFilter { label: (*name).into(), tone: -1, group: 0, choice: c as i32 }))
                .into_iter()
                .collect(),
        ));
        let category = self.category.checked_sub(1).and_then(|i| categories.get(i)).copied();

        // Valid results first, from the fastest, then those that do not count.
        let mut finished: Vec<(usize, u32, bool)> = ranking
            .entries
            .iter()
            .filter_map(|(i, n)| event.runners[*i].runs[*n].result.map(|(seconds, valid)| (*i, seconds, valid)))
            .collect();
        finished.sort_by_key(|(_, seconds, valid)| (!*valid, *seconds));
        let best = finished.first().filter(|(_, _, valid)| *valid).map_or(0, |(_, seconds, _)| *seconds);

        let mut rows = Vec::new();
        for (position, (index, seconds, valid)) in finished.iter().copied().enumerate() {
            let runner = &event.runners[index];
            let text = format!("{} {} {}", runner.name(), runner.club, runner.card.unwrap_or_default()).to_lowercase();
            if category.is_some_and(|c| c != runner.category) || !self.search.split_whitespace().all(|word| text.contains(word)) {
                continue;
            }
            rows.push(ResultRow {
                runner: index as i32,
                place: if valid { (position + 1).to_string().into() } else { SharedString::new() },
                place_note: if valid { place_text(position + 1).into() } else { SharedString::new() },
                family: runner.family.into(),
                first: runner.first.into(),
                club: runner.club.into(),
                category: runner.category.as_str().into(),
                time: format_time(seconds).into(),
                gap: if valid && seconds > best { format!("+{}", format_time(seconds - best)).into() } else { SharedString::new() },
                badge: badge(&if valid { Label::new("OK", Kind::Ok) } else { Label::new("PM", Kind::Pm) }),
                crossed: !valid,
                selected: selected_runner == Some(index),
            });
        }
        page.set_empty(
            if !rows.is_empty() {
                SharedString::new()
            } else if self.search.is_empty() {
                "Personne n'est encore arrivé".into()
            } else {
                "Aucun résultat".into()
            },
        );
        page.set_rows(model(rows));
    }

    // The columns, one per ranking, side by side (brief Cowork, 11/11): place, name, time,
    // gap, state. Valid results first, then the rest; the runners still out last.
    fn show_columns(&self, page: &Results, event: &Event, all: &[Ranking], selected_runner: Option<usize>) {
        // What a run says: classed (with its time), out, or a result that does not count.
        let state_of = |i: usize, n: usize| -> (usize, Label) {
            let runner = &event.runners[i];
            match (runner.outcome, runner.runs[n].result) {
                (Outcome::Retired, _) => (3, Label::new("Abandon", Kind::Inactive)),
                (Outcome::Disqualified, _) => (4, Label::new("Disqualifié", Kind::Pm)),
                (_, Some((_, true))) => (0, Label::new("OK", Kind::Ok)),
                (_, Some((_, false))) => (2, Label::new("PM", Kind::Pm)),
                (_, None) if runner.is_racing() => (1, Label::new("En course", Kind::Racing)),
                _ => (9, Label::default()),
            }
        };
        let words: Vec<&str> = self.search.split_whitespace().collect();
        let mut focus = -1;
        let mut columns = Vec::new();
        for (r, ranking) in all.iter().enumerate() {
            if self.column.is_some_and(|c| c != r) {
                continue;
            }
            let mut entries: Vec<(usize, usize, usize, u32)> = ranking
                .entries
                .iter()
                .map(|(i, n)| {
                    let (state, _) = state_of(*i, *n);
                    (*i, *n, state, event.runners[*i].runs[*n].result.map_or(u32::MAX, |(s, _)| s))
                })
                .filter(|(_, _, state, _)| *state != 9)
                .collect();
            entries.sort_by_key(|(_, _, state, seconds)| (*state, *seconds));
            let best = entries.iter().find(|e| e.2 == 0).map_or(0, |e| e.3);
            let classed = entries.iter().filter(|e| e.2 == 0).count();
            let racing = entries.iter().filter(|e| e.2 == 1).count();
            let mut rows = Vec::new();
            for (place, (i, _, state, seconds)) in entries.iter().copied().enumerate() {
                if self.state.is_some_and(|s| s != state) {
                    continue;
                }
                let runner = &event.runners[i];
                let text = format!("{} {} {}", runner.name(), runner.club, runner.card.unwrap_or_default()).to_lowercase();
                let found = !words.is_empty() && words.iter().all(|w| text.contains(w));
                if found && focus < 0 {
                    focus = columns.len() as i32;
                }
                if selected_runner == Some(i) && focus < 0 {
                    focus = columns.len() as i32;
                }
                rows.push(ResultRow {
                    runner: i as i32,
                    place: if state == 0 { (place + 1).to_string().into() } else { SharedString::new() },
                    place_note: SharedString::new(),
                    family: runner.family.into(),
                    first: runner.first.into(),
                    club: runner.club.into(),
                    category: runner.category.as_str().into(),
                    time: if seconds == u32::MAX { SharedString::new() } else { format_time(seconds).into() },
                    gap: if state == 0 && seconds > best { format!("+{}", format_time(seconds - best)).into() } else { SharedString::new() },
                    badge: badge(&state_of(i, entries[place].1).1),
                    crossed: state >= 2,
                    selected: selected_runner == Some(i) || found,
                });
            }
            columns.push(ResultColumn {
                index: r as i32,
                title: ranking.title.as_str().into(),
                tone: ranking.tone,
                state: if racing > 0 {
                    format!("Provisoire · {classed} classé{} · {racing} en forêt", if classed > 1 { "s" } else { "" }).into()
                } else {
                    format!("{classed} classé{}", if classed > 1 { "s" } else { "" }).into()
                },
                rows: model(rows),
            });
        }
        page.set_columns(model(columns));
        page.set_focus_column(focus);

        // The Filtres of the columns: the rankings, then the states.
        let ranking_names: Vec<String> = all.iter().map(|r| r.label.clone()).collect();
        let groups: [(&str, Vec<String>, Option<usize>); 2] = [
            (if self.by_course { "Circuit" } else { "Catégorie" }, ranking_names, self.column),
            ("État", STATES.iter().map(|s| s.to_string()).collect(), self.state),
        ];
        page.set_column_filter_groups(model(
            groups
                .iter()
                .enumerate()
                .map(|(g, (title, names, chosen))| FilterGroup {
                    search: SharedString::new(),
                    searchable: false,
                    key: g as i32,
                    title: (*title).into(),
                    choices: model(
                        names
                            .iter()
                            .enumerate()
                            .map(|(c, n)| FilterChoice {
                                label: n.as_str().into(),
                                tone: if g == RANKING { all[c].tone } else { -1 },
                                active: *chosen == Some(c),
                            })
                            .collect(),
                    ),
                })
                .collect(),
        ));
        page.set_column_filters(model(
            groups
                .iter()
                .enumerate()
                .filter_map(|(g, (_, names, chosen))| {
                    let c = (*chosen)?;
                    Some(ActiveFilter {
                        label: names.get(c)?.as_str().into(),
                        tone: if g == RANKING { all[c].tone } else { -1 },
                        group: g as i32,
                        choice: c as i32,
                    })
                })
                .collect(),
        ));

        // The blocks of figures: during the race, who is classed, out, mispunched; after it,
        // what went wrong too (brief Cowork, 12/11). None before.
        let count = |s: usize| all.iter().flat_map(|r| &r.entries).filter(|(i, n)| state_of(*i, *n).0 == s).count();
        let racing = event.runners.iter().any(|r| r.is_racing());
        let started = event.runners.iter().any(|r| r.started);
        let wanted: &[usize] = if !started { &[] } else if racing { &[0, 1, 2] } else { &[0, 2, 3, 4] };
        page.set_stats(model(
            wanted
                .iter()
                .filter(|s| count(**s) > 0)
                .map(|s| StatBlock {
                    label: STATES[*s].into(),
                    count: count(*s).to_string().into(),
                    group: STATE as i32,
                    choice: *s as i32,
                    active: self.state == Some(*s),
                })
                .collect(),
        ));
    }
}
