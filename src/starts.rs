// The Départs tab: what is chosen, and the rows it shows. The draw is a simple one for now:
// at random inside each course, on the test data only.

use crate::home::badge;
use crate::test_data::{Event, GroupKind, Kind, Label, Runner, START_STEP, format_clock};
use crate::participants::card_label;
use crate::{ActiveFilter, App, ChangedStart, FilterChoice, FilterGroup, FriseCell, FriseRow, GroupMember, GroupRule, StartGroup, StartPoint, StartCourse, StartPointSetting, StartRow, StartSection, Starts, TabItem, model, StatBlock};
use slint::{ComponentHandle, SharedString};

const VIEWS: [&str; 2] = ["Liste", "Frise"];
// The État filter of the starts, and the blocks of figures (brief Cowork, 12/11).
const STATES: [&str; 5] = ["Sans horaire", "Vacants", "Verrouillés", "À partir", "Partis"];
const VACANTS: usize = 1;

// The Frise has one column per minute, and a time written every 2 minutes (prototype: depFrise).
const MINUTE: u32 = 60;
const FRISE_LABEL_EVERY: u32 = 2 * MINUTE;

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
    course: Option<usize>,   // the course chosen in the panel; none: all of them
    state: Option<usize>,    // Filtres › État, one of STATES
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
            course: None,
            state: None,
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

    // The panel: all the courses, or one (brief Cowork, 11/11).
    pub fn select_course(&mut self, course: i32) {
        self.course = usize::try_from(course).ok();
        self.group = None;
    }

    // A group is never a filter: it opens on the right, to be set and completed (Robin, 11/11).
    pub fn select_group(&mut self, group: usize) {
        self.group = Some(group);
        self.selected = None;
    }

    // The Filtres bubble: the start point only. The groups are in the panel, and a choice has
    // one place only (brief Cowork, 11/11). A second click on the chosen choice takes it away.
    pub fn toggle_filter(&mut self, group: usize, choice: usize) {
        if group == 1 {
            self.state = if self.state == Some(choice) { None } else { Some(choice) };
            return;
        }
        self.point = if self.point == choice + 1 { 0 } else { choice + 1 };
    }

    // A block of figures: its state, alone; again, it goes away.
    pub fn pick_stat(&mut self, choice: usize) {
        self.group = None;
        self.toggle_filter(1, choice);
    }

    pub fn clear_filters(&mut self) {
        self.state = None;
        self.point = 0;
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
        let in_state = match self.state {
            None => true,
            Some(0) => runner.start == 0,
            Some(VACANTS) => false,
            Some(2) => runner.locked,
            Some(3) => !runner.started,
            Some(_) => runner.started,
        };
        in_state && self.search.split_whitespace().all(|word| text.contains(word))
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

        page.set_chips(model(vec!["Avec horaires".into(), published.as_str().into(), "Clubs séparés".into(), "1 vacant par circuit".into()]));
        page.set_summary(
            format!("{} · {range} · {vacants} · {}", plural(starters.len(), "partant"), published.to_lowercase()).into(),
        );
        page.set_summary_short(format!("{} · {range}", plural(starters.len(), "partant")).into());

        let changed: Vec<&Runner> = runners.iter().filter(|r| r.changed).collect();
        page.set_alert_count(changed.len().to_string().into());
        page.set_alert_title(if changed.is_empty() {
            SharedString::new()
        } else if changed.len() > 1 {
            format!("{} coureurs ont changé d'heure depuis la publication", changed.len()).into()
        } else {
            "1 coureur a changé d'heure depuis la publication".into()
        });
        // Nothing to publish once published and unchanged, nor while a draw is not applied
        // (departs-tirage-v2.md, Publier); the count is the runners whose time changed.
        page.set_publish_enabled(self.preview.is_none() && (event.published.is_none() || !changed.is_empty()));
        page.set_publish_label(if changed.is_empty() { "Publier".into() } else { format!("Publier ({})", changed.len()).into() });
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
        page.set_active_group(self.group.filter(|g| *g < event.groups.len()).map_or(-1, |g| g as i32));
        if let Some(group) = self.group.and_then(|g| event.groups.get(g)) {
            page.set_group_name(group.name.as_str().into());
            page.set_group_kind(match group.kind {
                GroupKind::Range => 0,
                GroupKind::Together => 1,
                GroupKind::Apart => 2,
            });
            page.set_group_setting(group.setting.as_str().into());
            page.set_group_facts(format!("{} · {}", group.rule, plural(group.members.len(), "coureur")).into());
            // The tab sums up the settings: the type, then the rules, or « À la main ».
            let mut summary = vec![group.rule.clone()];
            summary.extend(group.rules.iter().cloned());
            if group.rules.is_empty() {
                summary.push("À la main".into());
            }
            page.set_group_summary(summary.join(" · ").into());
            page.set_group_rules(model(
                group
                    .rules
                    .iter()
                    .map(|r| GroupRule { text: r.as_str().into(), count: plural(group.members.iter().filter(|m| !group.by_hand.contains(m)).count(), "coureur").into() })
                    .collect(),
            ));
            let mut members = group.members.clone();
            members.sort_by_key(|i| self.time_of(event, *i));
            page.set_group_members(model(
                members
                    .iter()
                    .map(|&i| {
                        let runner = &runners[i];
                        let course = runner.runs.first().map(|r| r.course);
                        GroupMember {
                            index: i as i32,
                            time: format_clock(self.time_of(event, i)).into(),
                            family: runner.family.into(),
                            first: runner.first.into(),
                            category: runner.category.as_str().into(),
                            club: runner.club.into(),
                            course: course.map_or(SharedString::new(), |c| event.courses[c].name.as_str().into()),
                            tone: course.map_or(-1, |c| c as i32),
                            origin: if group.by_hand.contains(&i) {
                                "À la main".into()
                            } else {
                                group.rules.first().map_or(SharedString::new(), |r| r.as_str().into())
                            },
                        }
                    })
                    .collect(),
            ));
        }

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

        // The filters: the start point, when there are several.
        let point_names: Vec<String> = if points.len() > 1 { points.iter().map(|p| format!("Départ S{p}")).collect() } else { Vec::new() };
        let state_names: Vec<String> = STATES.iter().map(|s| s.to_string()).collect();
        let chosen = [self.point.checked_sub(1), self.state];
        let lists = [("Point de départ", &point_names), ("État", &state_names)];
        page.set_filter_groups(model(
            lists
                .iter()
                .enumerate()
                .filter(|(_, (_, names))| !names.is_empty())
                .map(|(g, (title, names))| FilterGroup {
                    search: SharedString::new(),
                    searchable: false,
                    key: g as i32,
                    title: (*title).into(),
                    choices: model(
                        names
                            .iter()
                            .enumerate()
                            .map(|(c, name)| FilterChoice { label: name.as_str().into(), tone: -1, active: chosen[g] == Some(c) })
                            .collect(),
                    ),
                })
                .collect(),
        ));
        page.set_active_filters(model(
            lists
                .iter()
                .enumerate()
                .filter_map(|(g, (_, names))| {
                    let c = chosen[g]?;
                    Some(ActiveFilter { label: names.get(c)?.as_str().into(), tone: -1, group: g as i32, choice: c as i32 })
                })
                .collect(),
        ));

        // One settings block per start point: its first start, its step, and how many leave together.
        let point_of = |index: usize| runners[index].runs.first().map(|r| event.courses[r.course].start_point);
        page.set_point_settings(model(
            points
                .iter()
                .map(|point| {
                    let times: Vec<u32> = starters.iter().filter(|i| point_of(**i) == Some(*point)).map(|i| self.time_of(event, *i)).collect();
                    let together = times.iter().map(|t| times.iter().filter(|o| *o / MINUTE == *t / MINUTE).count()).max().unwrap_or(0);
                    StartPointSetting {
                        name: format!("Départ S{point}").into(),
                        first_start: times.iter().min().map_or("—".into(), |t| format_clock(*t)).into(),
                        step: format!("Toutes les {} min", START_STEP / MINUTE).into(),
                        limit: format!("Au plus {} par minute", plural(together, "coureur")).into(),
                    }
                })
                .collect(),
        ));
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

        // The vacants show with no filter, or alone with theirs.
        let filtering = (self.group.is_some() || !self.search.is_empty() || self.state.is_some()) && self.state != Some(VACANTS);

        // The blocks of figures (brief Cowork, 12/11): before the race, what the draw left;
        // during, who is to start and who has; after, none.
        let started = runners.iter().any(|r| r.started);
        let racing = runners.iter().any(|r| r.is_racing() || !r.started);
        let count = |s: usize| -> usize {
            match s {
                0 => starters.iter().filter(|i| runners[**i].start == 0).count(),
                VACANTS => event.vacants.len(),
                2 => starters.iter().filter(|i| runners[**i].locked).count(),
                3 => starters.iter().filter(|i| !runners[**i].started).count(),
                _ => starters.iter().filter(|i| runners[**i].started).count(),
            }
        };
        let wanted: &[usize] = if !started { &[0, VACANTS, 2] } else if racing { &[3, 4] } else { &[] };
        page.set_stats(model(
            wanted
                .iter()
                .map(|s| (*s, count(*s)))
                .filter(|(_, n)| *n > 0)
                .map(|(s, n)| StatBlock {
                    label: STATES[s].into(),
                    count: n.to_string().into(),
                    group: 1,
                    choice: s as i32,
                    active: self.state == Some(s),
                })
                .collect(),
        ));
        let in_point = |course: usize| self.point == 0 || event.courses[course].start_point == points[self.point - 1];
        let entrants_of = |course: usize| runners.iter().filter(|r| r.runs.first().is_some_and(|run| run.course == course)).count();
        page.set_courses(model(
            event
                .courses
                .iter()
                .enumerate()
                .map(|(i, course)| StartCourse {
                    index: i as i32,
                    name: course.name.as_str().into(),
                    detail: format!("S{} · {}", course.start_point, plural(entrants_of(i), "coureur")).into(),
                })
                .collect(),
        ));
        page.set_active_course(self.course.map_or(-1, |c| c as i32));
        let shown_course = |course: usize| in_point(course) && self.course.is_none_or(|c| c == course);
        page.set_sections(model(
            event
                .courses
                .iter()
                .enumerate()
                .filter(|(i, _)| shown_course(*i))
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
                    let entrants = entrants_of(i);
                    StartSection {
                        index: i as i32,
                        name: course.name.as_str().into(),
                        tone: i as i32,
                        chips: model(vec!["Écart 2 min".into(), "Catégories mêlées".into(), "Ordre au hasard".into()]),
                        detail: format!("S{} · {} · {}", course.start_point, course.categories, plural(entrants, "coureur")).into(),
                        rows: model(lines),
                    }
                })
                .collect(),
        ));

        // The frise: one column per minute, rows sorted by start point.
        let columns = ((last.saturating_sub(first)) / MINUTE) as usize + 1;
        page.set_frise_times(model(
            (0..columns)
                .map(|c| first + c as u32 * MINUTE)
                .map(|t| if t % FRISE_LABEL_EVERY == 0 { format_clock(t).into() } else { SharedString::new() })
                .collect(),
        ));
        let mut order: Vec<usize> = (0..event.courses.len()).filter(|i| shown_course(*i)).collect();
        order.sort_by_key(|i| event.courses[*i].start_point);
        page.set_frise_rows(model(
            order
                .iter()
                .enumerate()
                .map(|(n, &i)| {
                    let course = &event.courses[i];
                    let empty = FriseCell {
                        index: -1,
                        family: SharedString::new(),
                        first: SharedString::new(),
                        vacant: false,
                        locked: false,
                        moved: false,
                        left: false,
                        dimmed: false,
                    };
                    let mut cells = vec![empty; columns];
                    for vacant in event.vacants.iter().filter(|v| v.course == i) {
                        cells[((vacant.time - first) / MINUTE) as usize].vacant = true;
                    }
                    for index in starters.iter().copied().filter(|r| runners[*r].runs[0].course == i) {
                        let time = self.time_of(event, index);
                        let moved = time != runners[index].start;
                        if moved {
                            let old = ((runners[index].start.saturating_sub(first)) / MINUTE) as usize;
                            if let Some(cell) = cells.get_mut(old).filter(|c| c.index < 0) {
                                cell.left = true;
                            }
                        }
                        cells[((time - first) / MINUTE) as usize] = FriseCell {
                            index: index as i32,
                            family: runners[index].family.into(),
                            first: runners[index].first.into(),
                            vacant: false,
                            locked: runners[index].locked,
                            moved,
                            left: false,
                            dimmed: !self.shown(event, index),
                        };
                    }
                    let first_of_point = n == 0 || event.courses[order[n - 1]].start_point != course.start_point;
                    FriseRow {
                        point: if points.len() > 1 && first_of_point {
                            format!("Départ S{}", course.start_point).into()
                        } else {
                            SharedString::new()
                        },
                        name: course.name.as_str().into(),
                        tone: i as i32,
                        cells: model(cells),
                    }
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
            let time = self.time_of(event, index);
            page.set_detail_time(format_clock(time).into());
            page.set_detail_old_time(if time != runner.start { format_clock(runner.start).into() } else { SharedString::new() });
            page.set_detail_point(course.map_or(SharedString::new(), |c| format!("S{}", c.start_point).into()));
            page.set_detail_locked(runner.locked);
            page.set_detail_club(runner.club.into());
            page.set_detail_card(card_label(runner).into());
            let groups: Vec<&str> = event.groups.iter().filter(|g| g.members.contains(&index)).map(|g| g.name.as_str()).collect();
            page.set_detail_groups(if groups.is_empty() { "—".into() } else { groups.join(", ").into() });
        }
    }
}
