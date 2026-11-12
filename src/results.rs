// The Résultats tab: one ranking at a time, by category or by course. Each ranking is worked
// out again from the test data at every reading.

use crate::home::badge;
use crate::test_data::{Event, Kind, Label, format_time};
use crate::{ActiveFilter, App, FilterChoice, FilterGroup, LiveFilter, ResultRow, Results, model};
use slint::{ComponentHandle, SharedString};
use std::collections::BTreeMap;

const MODES: [&str; 2] = ["Par catégorie", "Par circuit"];

#[derive(Default)]
pub struct ResultsState {
    by_course: bool,
    ranking: usize,
    category: usize,    // 0 all, otherwise a category of the course + 1
    search: String,
    ranking_search: String,
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
    pub fn select_mode(&mut self, mode: usize) {
        self.by_course = mode == 1;
        self.ranking = 0;
        self.category = 0;
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
        // The generator makes no withdrawal yet: there is none to count.
        page.set_summary_short(format!("{arrived} · {racing} en course").into());
        page.set_summary(format!("{arrived} · {racing} en course · 0 abandon").into());

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
}
