// The Direct tab: every card read, the newest first, and what still needs someone.
// Settling a reading works on the test data only.

use crate::home::badge;
use crate::test_data::{Event, FINISH_READERS, Format, Issue, LIVE_SECONDS, Reader, Reading, Runner, format_clock, format_clock_seconds, format_time, missed_control};
use crate::{ActiveFilter, App, FilterChoice, FilterGroup, Live, LiveCandidate, LiveCount, LiveFilter, LiveLast, LiveListRow, LiveProblem, LiveRow, LiveSplit, LiveStop, LiveSummary, model};
use slint::{ComponentHandle, SharedString};

const COUNTS: [&str; 5] = ["inscrits", "arrivés", "en course", "PM", "non partis"];
const LIST_TITLES: [&str; 5] = ["Inscrits", "Arrivés", "En course", "PM", "Non partis"];

#[derive(Clone, Copy, PartialEq)]
enum Selected {
    Reading(usize),
    Runner(usize),
}

#[derive(Default)]
pub struct LiveState {
    course: usize,  // 0 all, otherwise a course + 1
    reader: usize,  // 0 all, otherwise a reader + 1
    search: String,
    list: Option<usize>,
    selected: Option<Selected>,
    by_course: bool,    // how the Résultats rank: the same places everywhere
    to_handle_only: bool,
}

fn place_text(place: usize) -> String {
    if place == 1 { "1er".into() } else { format!("{place}e") }
}

// The time and place a reading shows: a card nobody knows has none until it has a runner.
fn time_and_place(event: &Event, reading: &Reading, by_course: bool) -> (String, String) {
    let Some(runner) = reading.runner else { return ("—".into(), String::new()) };
    if !reading.valid {
        return (format_time(reading.seconds), "—".into());
    }
    let category = (!by_course).then_some(event.runners[runner].category.as_str());
    (format_time(reading.seconds), place_text(event.place(reading.course, category, reading.seconds)))
}

fn category(event: &Event, reading: &Reading) -> String {
    reading.runner.map_or("—".into(), |i| event.runners[i].category.clone())
}

fn action(reading: &Reading) -> &'static str {
    match reading.issue {
        _ if !reading.to_handle() => "",
        Some(Issue::UnknownCard) => "Attribuer",
        _ => "Régler",
    }
}

fn settled(reading: &Reading) -> String {
    reading.settled_at.map_or(String::new(), |at| format!("réglée à {}", format_clock(at)))
}

// How long ago the last reading was read, while it is live.
fn age(seconds: u32) -> String {
    if seconds < 2 { "à l'instant".into() } else { format!("il y a {seconds} s") }
}

fn crossed(reading: &Reading) -> bool {
    !reading.valid && reading.runner.is_some() && !reading.to_handle()
}

// When each control was punched, spread evenly along the time: the generator keeps no punches.
fn stop_time(seconds: u32, stop: usize, stops: usize) -> u32 {
    seconds * (stop as u32 + 1) / (stops as u32 + 1)
}

// The last reading is live for a short while after it was read (prototype: lastLive).
pub fn last_reading(event: &Event, readers: &[Reader], clock: u32, by_course: bool) -> LiveLast {
    let Some(index) = event.readings.len().checked_sub(1) else { return LiveLast::default() };
    let reading = &event.readings[index];
    let since = clock.saturating_sub(reading.at);
    if since >= LIVE_SECONDS {
        return LiveLast::default();
    }
    let (time, place) = time_and_place(event, reading, by_course);
    let course = &event.courses[reading.course];
    let ordered = course.format == Format::Imposed;
    let missed = (!reading.valid).then(|| missed_control(reading.seconds, course.controls.len()));
    let known = reading.runner.is_some();
    LiveLast {
        live: true,
        index: index as i32,
        badge: badge(&event.reading_label(reading)),
        name: event.reading_name(reading).into(),
        club: reading.runner.map_or(SharedString::new(), |i| event.runners[i].club.into()),
        category: category(event, reading).into(),
        tone: reading.runner.map_or(-1, |_| reading.course as i32),
        time: time.into(),
        place: place.into(),
        age: age(since).into(),
        // What is left of its live time, emptied by the drain bar.
        left: 1.0 - since as f32 / LIVE_SECONDS as f32,
        reader: readers.get(reading.reader).map_or(SharedString::new(), |r| r.name.as_str().into()),
        crossed: crossed(reading),
        stops: model(
            course
                .controls
                .iter()
                .enumerate()
                .map(|(n, code)| LiveStop {
                    rank: if ordered { (n + 1).to_string().into() } else { SharedString::new() },
                    code: code.to_string().into(),
                    time: if !known {
                        SharedString::new()
                    } else if missed == Some(n) {
                        "manque".into()
                    } else {
                        format_time(stop_time(reading.seconds, n, course.controls.len())).into()
                    },
                    missed: missed == Some(n),
                })
                .collect(),
        ),
    }
}

impl LiveState {
    pub fn set_ranked_by_course(&mut self, by_course: bool) {
        self.by_course = by_course;
    }

    pub fn select_course(&mut self, course: usize) {
        self.course = course;
    }

    pub fn select_reader(&mut self, reader: usize) {
        self.reader = reader;
    }

    pub fn set_search(&mut self, text: &str) {
        self.search = text.trim().to_lowercase();
    }

    pub fn select_row(&mut self, reading: usize) {
        let selected = Selected::Reading(reading);
        self.selected = if self.selected == Some(selected) { None } else { Some(selected) };
    }

    pub fn open_row(&mut self, reading: usize) {
        self.selected = Some(Selected::Reading(reading));
    }

    pub fn selected_runner(&self, event: &Event) -> Option<usize> {
        match self.selected? {
            Selected::Reading(i) => event.readings.get(i)?.runner,
            Selected::Runner(i) => Some(i),
        }
    }

    pub fn close_detail(&mut self) {
        self.selected = None;
    }

    // The drawer goes down: its detail and its « à traiter » filter go with it.
    pub fn leave(&mut self) {
        self.selected = None;
        self.to_handle_only = false;
    }

    pub fn show_to_handle(&mut self) {
        self.to_handle_only = true;
    }

    pub fn toggle_to_handle(&mut self) {
        self.to_handle_only = !self.to_handle_only;
    }

    // The Filtres bubble (brief zones-onglets § 5): 0 the course, 1 the reader, 2 what is to
    // handle. A second click on the chosen choice takes it away.
    pub fn toggle_filter(&mut self, group: usize, choice: usize) {
        match group {
            0 => self.course = if self.course == choice + 1 { 0 } else { choice + 1 },
            1 => self.reader = if self.reader == choice + 1 { 0 } else { choice + 1 },
            _ => self.to_handle_only = !self.to_handle_only,
        }
    }

    pub fn clear_filters(&mut self) {
        self.course = 0;
        self.reader = 0;
        self.to_handle_only = false;
    }

    pub fn toggle_list(&mut self, list: usize) {
        self.list = if self.list == Some(list) { None } else { Some(list) };
    }

    // A name in a list opens their last reading, or their runner when they have none.
    pub fn select_runner(&mut self, event: &Event, runner: usize) {
        let last = event.readings.iter().rposition(|r| r.runner == Some(runner));
        self.selected = Some(last.map_or(Selected::Runner(runner), Selected::Reading));
    }

    pub fn give_to(&mut self, event: &mut Event, runner: usize, clock: u32) {
        if let Some(Selected::Reading(reading)) = self.selected {
            event.give_reading_to(reading, runner, clock);
        }
    }

    pub fn keep_first(&mut self, event: &mut Event, clock: u32) {
        if let Some(Selected::Reading(reading)) = self.selected {
            event.keep_first_result(reading, clock);
        }
    }

    fn matches(&self, event: &Event, reading: &Reading) -> bool {
        let club = reading.runner.map_or("", |i| event.runners[i].club);
        let text = format!("{} {club} {} {}", event.reading_name(reading), reading.card, category(event, reading)).to_lowercase();
        (self.course == 0 || reading.course + 1 == self.course)
            && (self.reader == 0 || reading.reader + 1 == self.reader)
            && self.search.split_whitespace().all(|word| text.contains(word))
            && (!self.to_handle_only || reading.to_handle())
    }

    fn row(&self, event: &Event, index: usize) -> LiveRow {
        let reading = &event.readings[index];
        let (time, place) = time_and_place(event, reading, self.by_course);
        let (family, first) = reading.runner.map_or((event.reading_name(reading), String::new()), |i| {
            (event.runners[i].family.to_string(), event.runners[i].first.to_string())
        });
        LiveRow {
            index: index as i32,
            badge: badge(&event.reading_label(reading)),
            family: family.into(),
            first: first.into(),
            category: category(event, reading).into(),
            tone: reading.runner.map_or(-1, |_| reading.course as i32),
            time: time.into(),
            place: place.into(),
            clock: format_clock_seconds(reading.at).into(),
            action: action(reading).into(),
            settled: settled(reading).into(),
            crossed: crossed(reading),
            to_handle: reading.to_handle(),
            selected: self.selected == Some(Selected::Reading(index)),
        }
    }

    fn last(&self, event: &Event, readers: &[Reader], clock: u32) -> LiveLast {
        last_reading(event, readers, clock, self.by_course)
    }

    fn lists(&self, event: &Event, list: usize, clock: u32) -> Vec<LiveListRow> {
        let course_name = |runner: &Runner| {
            runner.runs.first().map_or(String::new(), |r| event.courses[r.course].name.replace("Circuit ", ""))
        };
        let row = |index: usize, detail: String| {
            let runner = &event.runners[index];
            LiveListRow {
                runner: index as i32,
                name: runner.name().into(),
                detail: detail.into(),
                tone: runner.runs.first().map_or(-1, |r| r.course as i32),
            }
        };
        let mut indexes: Vec<usize> = (0..event.runners.len()).collect();
        indexes.retain(|i| {
            let runner = &event.runners[*i];
            match list {
                1 => runner.is_back(),
                2 => runner.is_racing(),
                3 => runner.is_back() && runner.runs.iter().any(|r| matches!(r.result, Some((_, false)))),
                4 => !runner.started,
                _ => true,
            }
        });
        // On the course: the longest out comes first.
        if list == 2 {
            indexes.sort_by_key(|i| event.runners[*i].start);
        } else {
            indexes.sort_by_key(|i| event.runners[*i].name());
        }
        indexes
            .into_iter()
            .map(|i| {
                let runner = &event.runners[i];
                let result = runner.runs.iter().rev().find_map(|r| r.result).map(|(seconds, _)| format_time(seconds));
                let detail = match list {
                    2 => format!("{} · {} · depuis {}", course_name(runner), runner.category, format_time(clock.saturating_sub(runner.start))),
                    1 | 3 => format!("{} · {} · {}", course_name(runner), runner.category, result.unwrap_or_default()),
                    4 => format!("{} · {} · départ {}", course_name(runner), runner.category, format_clock(runner.start)),
                    _ => format!("{} · {}", course_name(runner), runner.category),
                };
                row(i, detail)
            })
            .collect()
    }

    fn show_detail(&self, page: &Live, event: &Event) {
        let selected = self.selected.and_then(|s| match s {
            Selected::Reading(i) => event.readings.get(i).map(|r| (Some(r), r.runner, r.course)),
            Selected::Runner(i) => event.runners.get(i).map(|r| (None, Some(i), r.runs.first().map_or(0, |run| run.course))),
        });
        page.set_detail_open(selected.is_some());
        let Some((reading, runner_index, course_index)) = selected else { return };
        let runner = runner_index.map(|i| &event.runners[i]);

        let (word, time, place) = match reading {
            Some(reading) => {
                let (time, place) = time_and_place(event, reading, self.by_course);
                (event.reading_label(reading), time, place)
            }
            None => (runner.map(Runner::label).unwrap_or_default(), "—".to_string(), String::new()),
        };
        page.set_detail_badge(badge(&word));
        page.set_detail_name(match (reading, runner) {
            (Some(reading), _) => event.reading_name(reading).into(),
            (None, Some(runner)) => runner.name().into(),
            _ => SharedString::new(),
        });
        let course = event.courses.get(course_index);
        page.set_detail_tone(if runner.is_some() { course_index as i32 } else { -1 });
        page.set_detail_course(match (course, runner) {
            (Some(course), Some(runner)) => format!("{} · {}", course.name, runner.category).into(),
            (Some(course), None) => format!("{} reconnu", course.name).into(),
            _ => SharedString::new(),
        });
        page.set_detail_club(runner.map_or("—".into(), |r| r.club.into()));
        let card = reading.map(|r| r.card).or(runner.and_then(|r| r.card));
        page.set_detail_card(card.map_or("—".into(), |c| c.to_string().into()));
        page.set_detail_start(runner.map_or("—".into(), |r| format_clock(r.start).into()));
        page.set_detail_time(time.as_str().into());
        page.set_detail_place(place.as_str().into());
        page.set_detail_crossed(reading.is_some_and(crossed));
        page.set_detail_settled(reading.map_or(String::new(), settled).as_str().into());
        page.set_detail_has_runner(runner.is_some());
        // No ticket for a reading still to settle (lecture-a-regler.md § 4).
        page.set_detail_can_print(reading.is_some_and(|r| r.runner.is_some() && !r.to_handle()));

        // What the reading asks for, in a band of its own.
        let issue = reading.filter(|r| r.to_handle()).and_then(|r| r.issue);
        page.set_detail_unknown(issue == Some(Issue::UnknownCard));
        page.set_detail_twice(issue == Some(Issue::AlreadyRead));
        let first = reading.and_then(|reading| {
            event.readings.iter().find(|r| r.runner == reading.runner && r.issue.is_none())
        });
        page.set_detail_problem(match issue {
            Some(Issue::UnknownCard) => match course {
                Some(course) => format!("Puce non inscrite. Ses poinçons correspondent à {}.", course.name).into(),
                None => "Puce non inscrite.".into(),
            },
            Some(Issue::AlreadyRead) => first.map_or("Cette puce a déjà été lue.".into(), |first| {
                format!(
                    "Cette puce a déjà été lue à {} (résultat : {}). Le premier résultat est gardé.",
                    format_clock_seconds(first.at),
                    format_time(first.seconds)
                )
                .into()
            }),
            None => SharedString::new(),
        });
        let candidates = if issue == Some(Issue::UnknownCard) { event.candidates(course_index) } else { Vec::new() };
        page.set_candidates(model(
            candidates
                .into_iter()
                .take(3)
                .map(|i| {
                    let runner = &event.runners[i];
                    LiveCandidate {
                        runner: i as i32,
                        name: runner.name().into(),
                        detail: format!("{} · parti à {}", runner.category, format_clock(runner.start)).into(),
                    }
                })
                .collect(),
        ));

        // Postes et temps.
        let Some(course) = course else {
            page.set_splits(model(Vec::new()));
            return;
        };
        let has_times = reading.is_some_and(|r| r.runner.is_some());
        let seconds = reading.map_or(0, |r| r.seconds);
        let missed = reading.filter(|r| !r.valid && has_times).map(|r| missed_control(r.seconds, course.controls.len()));
        let count = course.controls.len();
        let ordered = course.format == Format::Imposed;
        let start = runner.map(|r| r.start);
        // The time of day of a punch: the start, plus the time since.
        let clock_at = |at: u32| start.filter(|_| has_times).map_or("—".into(), |start| format_clock(start + at).into());
        let mut previous = 0;
        let mut splits: Vec<LiveSplit> = course
            .controls
            .iter()
            .enumerate()
            .map(|(n, code)| {
                let at = stop_time(seconds, n, count);
                let is_missed = missed == Some(n);
                let split = LiveSplit {
                    rank: if ordered { (n + 1).to_string().into() } else { SharedString::new() },
                    code: code.to_string().into(),
                    clock: if is_missed { "—".into() } else { clock_at(at) },
                    leg: if has_times && !is_missed { format_time(at - previous).into() } else { "—".into() },
                    total: if is_missed { "manque".into() } else if has_times { format_time(at).into() } else { "—".into() },
                    missed: is_missed,
                    finish: false,
                };
                if !is_missed {
                    previous = at;
                }
                split
            })
            .collect();
        // The finish: its leg from the last control, its total the run time.
        if has_times {
            splits.push(LiveSplit {
                rank: SharedString::new(),
                code: "Arrivée".into(),
                clock: clock_at(seconds),
                leg: format_time(seconds - previous).into(),
                total: format_time(seconds).into(),
                missed: false,
                finish: true,
            });
        }
        page.set_splits(model(splits));
    }

    pub fn show(&self, window: &App, event: Option<&Event>, readers: &[Reader], clock: u32) {
        let page = window.global::<Live>();
        let Some(event) = event else { return };
        let runners = &event.runners;
        let readers = &readers[..readers.len().min(FINISH_READERS)];
        let mut course_filters = vec![LiveFilter { label: "Tous".into(), tone: -1, detail: SharedString::new() }];
        course_filters.extend(event.courses.iter().enumerate().map(|(i, c)| LiveFilter {
            label: c.name.replace("Circuit ", "").into(),
            tone: i as i32,
            detail: SharedString::new(),
        }));
        page.set_course_filters(model(course_filters));
        page.set_active_course(self.course as i32);
        let mut reader_filters = vec![LiveFilter { label: format!("{} lecteurs", readers.len()).into(), tone: -1, detail: SharedString::new() }];
        reader_filters.extend(readers.iter().map(|reader| LiveFilter { label: reader.name.as_str().into(), tone: -1, detail: SharedString::new() }));
        page.set_reader_filters(model(reader_filters));
        page.set_active_reader(self.reader as i32);

        // The same filters, in the Filtres button and its bubble.
        let courses: Vec<(String, i32)> = event.courses.iter().enumerate().map(|(i, c)| (c.name.clone(), i as i32)).collect();
        let reader_names: Vec<(String, i32)> = readers.iter().map(|r| (r.name.clone(), -1)).collect();
        let to_handle = vec![("À traiter".to_string(), -1)];
        let chosen = [self.course.checked_sub(1), self.reader.checked_sub(1), self.to_handle_only.then_some(0)];
        let lists = [("Circuit", &courses), ("Lecteur", &reader_names), ("Lectures", &to_handle)];
        page.set_filter_groups(model(
            lists
                .iter()
                .enumerate()
                .filter(|(_, (_, choices))| !choices.is_empty())
                .map(|(g, (title, choices))| FilterGroup {
                    search: SharedString::new(),
                    searchable: false,
                    key: g as i32,
                    title: (*title).into(),
                    choices: model(
                        choices
                            .iter()
                            .enumerate()
                            .map(|(c, (label, tone))| FilterChoice { label: label.as_str().into(), tone: *tone, active: chosen[g] == Some(c) })
                            .collect(),
                    ),
                })
                .collect(),
        ));
        page.set_active_filters(model(
            lists
                .iter()
                .enumerate()
                .filter_map(|(g, (_, choices))| {
                    let c = chosen[g]?;
                    let (label, tone) = choices.get(c)?;
                    Some(ActiveFilter { label: label.as_str().into(), tone: *tone, group: g as i32, choice: c as i32 })
                })
                .collect(),
        ));
        page.set_search_text(self.search.as_str().into());

        let racing = runners.iter().filter(|r| r.is_racing()).count();
        let back = runners.iter().filter(|r| r.is_back()).count();
        let missing = runners.iter().filter(|r| r.is_back() && r.runs.iter().any(|run| matches!(run.result, Some((_, false))))).count();
        let not_started = runners.iter().filter(|r| !r.started).count();
        let values = [runners.len(), back, racing, missing, not_started];
        page.set_counts(model(
            COUNTS.iter().zip(values).map(|(label, value)| LiveCount { value: value.to_string().into(), label: (*label).into() }).collect(),
        ));
        page.set_active_list(self.list.map_or(-1, |l| l as i32));
        page.set_list_title(self.list.map_or("", |l| LIST_TITLES[l]).into());
        page.set_list_rows(model(self.list.map_or(Vec::new(), |list| self.lists(event, list, clock))));

        // The figures in bold, the words around them (prototype: drBar).
        let closing = if racing > 0 { format!("fermeture {}", format_clock(event.closing)) } else { String::new() };
        page.set_summary(LiveSummary {
            racing: racing.to_string().into(),
            back: back.to_string().into(),
            back_word: if back > 1 { "arrivés" } else { "arrivé" }.into(),
            closing: closing.into(),
        });

        let to_handle = event.to_handle();
        page.set_alert_count(to_handle.to_string().into());
        page.set_alert_title(if to_handle > 0 { format!("{to_handle} à traiter").into() } else { SharedString::new() });
        page.set_problems(model(
            event
                .readings
                .iter()
                .enumerate()
                .rev()
                .filter(|(_, r)| r.to_handle())
                .map(|(i, r)| {
                    let what = if r.issue == Some(Issue::UnknownCard) { "inconnue" } else { "déjà lue" };
                    let name = event.reading_name(r);
                    let label = if r.runner.is_some() { format!("{name} : puce {what}") } else { format!("{name} {what}") };
                    LiveProblem { index: i as i32, label: format!("{label} · {}", format_clock(r.at)).into(), action: action(r).into() }
                })
                .collect(),
        ));

        page.set_to_handle_only(self.to_handle_only);
        page.set_last(self.last(event, readers, clock));
        page.set_rows(model(
            (0..event.readings.len())
                .rev()
                .filter(|i| self.matches(event, &event.readings[*i]))
                .map(|i| self.row(event, i))
                .collect(),
        ));
        self.show_detail(&page, event);
    }
}
