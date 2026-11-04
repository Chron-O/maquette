// The Départs tab: what is chosen, and the rows it shows. The draw is a simple one for now:
// at random inside each course, on the test data only.

use crate::home::badge;
use crate::test_data::{Event, Kind, Label, Runner, START_STEP, format_clock};
use crate::{App, ChangedStart, FriseCell, FriseRow, StartGroup, StartPoint, StartRow, StartSection, Starts, TabItem, model};
use slint::{ComponentHandle, SharedString};

const VIEWS: [&str; 2] = ["Liste", "Frise"];

fn plural(count: usize, noun: &str) -> String {
    format!("{count} {noun}{}", if count > 1 { "s" } else { "" })
}

#[derive(Clone, Copy, PartialEq)]
enum SortKey {
    Time,
    Name,
    Category,
    Club,
    Card,
    State,
}

pub struct StartsState {
    view: usize,
    sort: SortKey,
    ascending: bool,
    point: usize,       // 0 all, otherwise a start point
    group: Option<usize>,
    search: String,
    selected: Option<usize>,
    pub preview: Option<Vec<u32>>,   // the proposed time of each runner, until applied
}

impl Default for StartsState {
    fn default() -> Self {
        StartsState {
            view: 0,
            sort: SortKey::Time,
            ascending: true,
            point: 0,
            group: None,
            search: String::new(),
            selected: None,
            preview: None,
        }
    }
}

fn state_label(runner: &Runner) -> Label {
    if runner.changed {
        Label::new("Nouvel horaire", Kind::ToCheck)
    } else if runner.is_racing() {
        Label::new("Parti", Kind::Racing)
    } else if runner.is_back() {
        Label::new("Parti · arrivé", Kind::Ok)
    } else {
        Label::new("Pas encore parti", Kind::Inactive)
    }
}

fn initials(runner: &Runner) -> String {
    format!("{}{}", runner.first.chars().next().unwrap_or(' '), runner.family.chars().next().unwrap_or(' '))
}

impl StartsState {
    pub fn select_view(&mut self, view: usize) {
        self.view = view;
    }

    pub fn select_row(&mut self, index: usize) {
        self.selected = if self.selected == Some(index) { None } else { Some(index) };
    }

    pub fn close_detail(&mut self) {
        self.selected = None;
    }

    pub fn select_point(&mut self, point: usize) {
        self.point = point;
    }

    pub fn select_group(&mut self, group: usize) {
        self.group = if self.group == Some(group) { None } else { Some(group) };
    }

    pub fn set_search(&mut self, text: &str) {
        self.search = text.trim().to_lowercase();
    }

    pub fn sort_by(&mut self, column: i32) {
        let key = match column {
            1 => SortKey::Name,
            2 => SortKey::Category,
            3 => SortKey::Club,
            4 => SortKey::Card,
            5 => SortKey::State,
            _ => SortKey::Time,
        };
        if self.sort == key {
            self.ascending = !self.ascending;
        } else {
            self.sort = key;
            self.ascending = true;
        }
    }

    pub fn toggle_lock(&mut self, event: &mut Event) {
        if let Some(runner) = self.selected.and_then(|i| event.runners.get_mut(i)) {
            runner.locked = !runner.locked;
        }
    }

    pub fn apply(&mut self, event: &mut Event) {
        let Some(times) = self.preview.take() else { return };
        let published = event.published.is_some();
        for (runner, time) in event.runners.iter_mut().zip(times) {
            if runner.start != time {
                runner.start = time;
                runner.changed = published;
            }
        }
    }

    fn time_of(&self, event: &Event, index: usize) -> u32 {
        self.preview.as_ref().map_or(event.runners[index].start, |times| times[index])
    }

    fn shown(&self, event: &Event, index: usize) -> bool {
        let runner = &event.runners[index];
        let text = format!("{} {}", runner.name(), runner.club).to_lowercase();
        let in_group = self.group.is_none_or(|g| event.groups.get(g).is_none_or(|g| g.members.contains(&index)));
        in_group && self.search.split_whitespace().all(|word| text.contains(word))
    }

    fn sorted(&self, event: &Event, course: usize) -> Vec<usize> {
        let mut rows: Vec<usize> = (0..event.runners.len())
            .filter(|i| event.runners[*i].runs.first().is_some_and(|r| r.course == course) && self.shown(event, *i))
            .collect();
        rows.sort_by_key(|i| {
            let runner = &event.runners[*i];
            let key = match self.sort {
                SortKey::Time => String::new(),
                SortKey::Name => runner.name(),
                SortKey::Category => runner.category.clone(),
                SortKey::Club => runner.club.to_string(),
                SortKey::Card => runner.card.map_or(String::new(), |c| c.to_string()),
                SortKey::State => state_label(runner).word,
            };
            (key, self.time_of(event, *i))
        });
        if !self.ascending {
            rows.reverse();
        }
        rows
    }

    fn row(&self, event: &Event, index: usize) -> StartRow {
        let runner = &event.runners[index];
        let time = self.time_of(event, index);
        let moved = time != runner.start;
        let note = match &self.preview {
            None => "",
            Some(_) if moved => "",
            Some(_) if runner.locked => "Verrouillé, gardé",
            Some(_) if runner.started => "Déjà parti, gardé",
            Some(_) => "Inchangé",
        };
        StartRow {
            index: index as i32,
            time: format_clock(time).into(),
            old_time: if moved { format_clock(runner.start).into() } else { SharedString::new() },
            locked: runner.locked,
            family: runner.family.into(),
            first: runner.first.into(),
            category: runner.category.as_str().into(),
            club: runner.club.into(),
            card: runner.card.map_or("—".into(), |c| c.to_string().into()),
            badge: badge(&if moved { Label::new("Proposé", Kind::ToCheck) } else { state_label(runner) }),
            note: note.into(),
            selected: self.selected == Some(index),
        }
    }

    pub fn show(&self, window: &App, event: Option<&Event>) {
        let page = window.global::<Starts>();
        let Some(event) = event else { return };
        let runners = &event.runners;

        let starters: Vec<usize> = (0..runners.len()).filter(|i| !runners[*i].runs.is_empty()).collect();
        let first = starters.iter().map(|i| self.time_of(event, *i)).min().unwrap_or(0);
        let vacant_times = event.vacants.iter().map(|v| v.time);
        let last = starters.iter().map(|i| self.time_of(event, *i)).chain(vacant_times).max().unwrap_or(0);
        let range = format!("{} → {}", format_clock(first), format_clock(last));
        let published = event.published.clone().unwrap_or("Non publiés".into());
        let vacants = plural(event.vacants.len(), "vacant");

        page.set_first_start(format_clock(first).into());
        page.set_chips(model(vec!["Avec horaires".into(), published.as_str().into(), "Clubs séparés".into(), "1 vacant par circuit".into()]));
        page.set_summary(
            format!("{} · {range} · {vacants} · {}", plural(starters.len(), "partant"), published.to_lowercase()).into(),
        );
        page.set_summary_short(format!("{} · {vacants}", plural(starters.len(), "partant")).into());

        let changed: Vec<&Runner> = runners.iter().filter(|r| r.changed).collect();
        page.set_alert_count(changed.len().to_string().into());
        page.set_alert_title(if changed.is_empty() {
            SharedString::new()
        } else if changed.len() > 1 {
            format!("{} coureurs ont changé d'heure", changed.len()).into()
        } else {
            "1 coureur a changé d'heure".into()
        });
        page.set_changed(model(
            changed.iter().map(|r| ChangedStart { name: r.name().into(), time: format_clock(r.start).into() }).collect(),
        ));

        page.set_groups(model(
            event
                .groups
                .iter()
                .map(|g| StartGroup {
                    name: g.name.as_str().into(),
                    detail: format!("{} · {}", g.rule, plural(g.members.len(), "coureur")).into(),
                })
                .collect(),
        ));
        page.set_active_group(self.group.map_or(-1, |g| g as i32));

        let points: Vec<u32> = {
            let mut list: Vec<u32> = event.courses.iter().map(|c| c.start_point).collect();
            list.sort_unstable();
            list.dedup();
            list
        };
        let mut chips = Vec::new();
        if points.len() > 1 {
            chips.push(StartPoint { label: "Tous".into() });
            chips.extend(points.iter().map(|p| StartPoint { label: format!("S{p}").into() }));
        }
        page.set_points(model(chips));
        page.set_active_point(self.point as i32);
        page.set_views(model(VIEWS.iter().map(|v| TabItem { label: (*v).into(), icon: Default::default() }).collect()));
        page.set_active_view(self.view as i32);
        page.set_search_text(self.search.as_str().into());
        page.set_sort_column(match self.sort {
            SortKey::Time => 0,
            SortKey::Name => 1,
            SortKey::Category => 2,
            SortKey::Club => 3,
            SortKey::Card => 4,
            SortKey::State => 5,
        });
        page.set_sort_ascending(self.ascending);

        let filtering = self.group.is_some() || !self.search.is_empty();
        let in_point = |course: usize| self.point == 0 || event.courses[course].start_point == points[self.point - 1];
        page.set_sections(model(
            event
                .courses
                .iter()
                .enumerate()
                .filter(|(i, _)| in_point(*i))
                .map(|(i, course)| {
                    let rows = self.sorted(event, i);
                    let mut lines: Vec<StartRow> = rows.iter().map(|r| self.row(event, *r)).collect();
                    if !filtering {
                        lines.extend(event.vacants.iter().filter(|v| v.course == i).map(|v| StartRow {
                            index: -1,
                            time: format_clock(v.time).into(),
                            old_time: SharedString::new(),
                            locked: false,
                            family: SharedString::new(),
                            first: SharedString::new(),
                            category: SharedString::new(),
                            club: SharedString::new(),
                            card: SharedString::new(),
                            badge: badge(&Label::default()),
                            note: SharedString::new(),
                            selected: false,
                        }));
                    }
                    let entrants = runners.iter().filter(|r| r.runs.first().is_some_and(|run| run.course == i)).count();
                    StartSection {
                        index: i as i32,
                        name: course.name.as_str().into(),
                        tone: i as i32,
                        chips: model(vec![
                            format!("S{}", course.start_point).into(),
                            course.categories.into(),
                            "écart 2 min".into(),
                            plural(entrants, "coureur").into(),
                        ]),
                        rows: model(lines),
                    }
                })
                .collect(),
        ));

        // The frise: one column per start.
        let columns = ((last.saturating_sub(first)) / START_STEP) as usize + 1;
        page.set_frise_times(model((0..columns).map(|c| format_clock(first + c as u32 * START_STEP).into()).collect()));
        page.set_frise_rows(model(
            event
                .courses
                .iter()
                .enumerate()
                .filter(|(i, _)| in_point(*i))
                .map(|(i, course)| {
                    let mut cells = vec![
                        FriseCell { index: -1, label: SharedString::new(), vacant: false, locked: false, moved: false, dimmed: false };
                        columns
                    ];
                    for vacant in event.vacants.iter().filter(|v| v.course == i) {
                        cells[((vacant.time - first) / START_STEP) as usize].vacant = true;
                    }
                    for index in starters.iter().copied().filter(|r| runners[*r].runs[0].course == i) {
                        let time = self.time_of(event, index);
                        cells[((time - first) / START_STEP) as usize] = FriseCell {
                            index: index as i32,
                            label: initials(&runners[index]).into(),
                            vacant: false,
                            locked: runners[index].locked,
                            moved: time != runners[index].start,
                            dimmed: !self.shown(event, index),
                        };
                    }
                    FriseRow { name: course.name.as_str().into(), tone: i as i32, cells: model(cells) }
                })
                .collect(),
        ));

        page.set_preview(self.preview.is_some());
        if self.preview.is_some() {
            let moved = starters.iter().filter(|i| self.time_of(event, **i) != runners[**i].start).count();
            let kept = starters.iter().filter(|i| runners[**i].locked || runners[**i].started).count();
            page.set_preview_detail(
                format!(
                    "{} · {range} · {vacants} · {}",
                    plural(moved, "coureur déplacé").replace("coureur déplacés", "coureurs déplacés"),
                    plural(kept, "heure gardée").replace("heure gardées", "heures gardées"),
                )
                .into(),
            );
        }

        let selected = self.selected.filter(|i| *i < runners.len());
        page.set_detail_open(selected.is_some());
        if let Some(index) = selected {
            let runner = &runners[index];
            let course = runner.runs.first().and_then(|r| event.courses.get(r.course));
            page.set_detail_name(runner.name().into());
            page.set_detail_subtitle(
                format!("{} · {}", runner.category, course.map_or("sans circuit", |c| c.name.as_str())).into(),
            );
            page.set_detail_time(format_clock(self.time_of(event, index)).into());
            page.set_detail_point(course.map_or(SharedString::new(), |c| format!("S{}", c.start_point).into()));
            page.set_detail_locked(runner.locked);
        }
    }
}
