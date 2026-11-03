// The Circuits tab: what is chosen, and the rows it shows.
// Everything is display only for now: nothing here changes a course.

use crate::test_data::{Control, Course, Event, FINISH, START};
use crate::{App, ColumnTitle, ControlRow, CourseItem, CourseLine, Courses, DetailCourse, MapMarker, SequenceStop, TabItem, model};
use slint::{ComponentHandle, SharedString};

const VIEWS: [&str; 3] = ["Tableau", "Lignes", "Carte"];
const MAP_WIDTH: f32 = 600.0;
const MAP_HEIGHT: f32 = 360.0;

#[derive(Clone, Copy, Default, PartialEq)]
enum Selection {
    #[default]
    Nothing,
    AllControls,
    Course(usize),
}

#[derive(Clone, Copy, PartialEq)]
enum SortKey {
    Code,
    Usage,
    Course(usize),
}

pub struct CoursesState {
    selection: Selection,
    view: usize,
    sort: SortKey,
    ascending: bool,
    highlight: usize,
    control: Option<u32>,
    code_search: String,
    course_search: String,
}

impl Default for CoursesState {
    fn default() -> Self {
        CoursesState {
            selection: Selection::Nothing,
            view: 0,
            sort: SortKey::Code,
            ascending: true,
            highlight: 0,
            control: None,
            code_search: String::new(),
            course_search: String::new(),
        }
    }
}

fn plural(count: usize, noun: &str) -> String {
    format!("{count} {noun}{}", if count > 1 { "s" } else { "" })
}

fn word(label: &str) -> TabItem {
    TabItem { label: label.into(), icon: Default::default() }
}

fn rank_in(course: &Course, code: u32) -> i32 {
    course.controls.iter().position(|c| *c == code).map_or(0, |i| i as i32 + 1)
}

fn usage_count(event: &Event, code: u32) -> usize {
    event.courses.iter().filter(|c| c.controls.contains(&code)).count()
}

fn position(event: &Event, code: u32) -> Option<Control> {
    event.controls.iter().find(|c| c.code == code).copied()
}

fn trace(event: &Event, course: &Course) -> String {
    let mut commands = format!("M{} {}", START.0, START.1);
    for code in &course.controls {
        if let Some(control) = position(event, *code) {
            commands += &format!(" L{} {}", control.x, control.y);
        }
    }
    commands + &format!(" L{} {}", FINISH.0, FINISH.1)
}

impl CoursesState {
    pub fn select_course(&mut self, index: i32) {
        self.selection = match index {
            -2 => Selection::Nothing,
            -1 => Selection::AllControls,
            i => Selection::Course(i as usize),
        };
        self.control = None;
    }

    pub fn select_view(&mut self, view: usize) {
        self.view = view;
    }

    pub fn select_highlight(&mut self, highlight: usize) {
        self.highlight = highlight;
    }

    pub fn sort_by(&mut self, column: i32) {
        let key = match column {
            -1 => SortKey::Code,
            -2 => SortKey::Usage,
            i => SortKey::Course(i as usize),
        };
        if self.sort == key {
            self.ascending = !self.ascending;
        } else {
            self.sort = key;
            self.ascending = true;
        }
    }

    // Selecting the same control again closes its detail.
    pub fn select_control(&mut self, code: &str) {
        let code = code.parse().ok();
        self.control = if self.control == code { None } else { code };
    }

    pub fn close_detail(&mut self) {
        self.control = None;
    }

    pub fn set_code_search(&mut self, text: &str) {
        self.code_search = text.trim().into();
    }

    pub fn set_course_search(&mut self, text: &str) {
        self.course_search = text.trim().to_lowercase();
    }

    fn sorted_codes(&self, event: &Event) -> Vec<u32> {
        let mut codes: Vec<u32> = event
            .controls
            .iter()
            .map(|c| c.code)
            .filter(|code| code.to_string().starts_with(&self.code_search))
            .collect();
        let rank = |code: u32, course: usize| {
            let rank = rank_in(&event.courses[course], code);
            // Controls outside the course come last, by code.
            if rank > 0 { rank as u32 } else { 1000 + code }
        };
        codes.sort_by_key(|code| match self.sort {
            SortKey::Code => (*code, 0),
            SortKey::Usage => (usage_count(event, *code) as u32, *code),
            SortKey::Course(course) => (rank(*code, course), 0),
        });
        if !self.ascending {
            codes.reverse();
        }
        codes
    }

    pub fn show(&self, window: &App, event: Option<&Event>) {
        let courses = window.global::<Courses>();
        let Some(event) = event else { return };

        courses.set_empty(event.courses.is_empty());
        courses.set_list_only(self.selection == Selection::Nothing);
        courses.set_views(model(VIEWS.iter().map(|v| word(v)).collect()));
        courses.set_views_narrow(model(VIEWS[..2].iter().map(|v| word(v)).collect()));
        courses.set_active_view(self.view as i32);

        let mut highlights = vec![word("Tous")];
        highlights.extend(event.courses.iter().map(|c| word(&c.name)));
        courses.set_highlights(model(highlights));
        courses.set_active_highlight(self.highlight as i32);

        let used: Vec<u32> = event.controls.iter().map(|c| c.code).filter(|c| usage_count(event, *c) > 0).collect();
        let unused: Vec<u32> = event.controls.iter().map(|c| c.code).filter(|c| usage_count(event, *c) == 0).collect();
        let shared = used.iter().filter(|c| usage_count(event, **c) > 1).count();

        courses.set_items(model(
            event
                .courses
                .iter()
                .enumerate()
                .filter(|(_, c)| c.name.to_lowercase().contains(&self.course_search))
                .map(|(i, c)| CourseItem {
                    index: i as i32,
                    name: c.name.clone().into(),
                    summary: format!("{} · {}", plural(c.controls.len(), "poste"), c.categories).into(),
                    trace: trace(event, c).into(),
                })
                .collect(),
        ));
        courses.set_all_summary(format!("{} · {}", plural(event.controls.len(), "poste"), plural(shared, "partagé")).into());
        courses.set_all_trace("".into());

        let (active, selected_course) = match self.selection {
            Selection::Course(i) => (i as i32, event.courses.get(i)),
            _ => (-1, None),
        };
        courses.set_active_course(active);
        courses.set_course_selected(selected_course.is_some());
        courses.set_course_title(
            selected_course.map_or(SharedString::new(), |c| format!("{} · {} · {}", c.name, c.categories, c.distance).into()),
        );

        let count = event.controls.len();
        courses.set_chips(model(vec![
            "Analyse : rien".into(),
            format!("Positions : {count} / {count}").into(),
        ]));
        if let Some(code) = unused.first() {
            courses.set_alert_title(format!("{} dans aucun circuit", plural(unused.len(), "poste")).into());
            let codes = unused.iter().map(u32::to_string).collect::<Vec<_>>().join(", ");
            courses.set_alert_detail(
                if unused.len() > 1 {
                    format!("Postes {codes} : posés mais empruntés par aucun circuit")
                } else {
                    format!("Poste {codes} : posé mais emprunté par aucun circuit")
                }
                .into(),
            );
            courses.set_alert_code(code.to_string().into());
        } else {
            courses.set_alert_title("".into());
        }
        courses.set_summary(
            format!("{} · {} · {} dans aucun circuit", plural(count, "poste"), plural(shared, "partagé"), unused.len()).into(),
        );
        courses.set_all_courses_usage(plural(event.courses.len(), "circuit").into());

        // Table
        courses.set_columns(model(
            event
                .courses
                .iter()
                .enumerate()
                .map(|(i, c)| ColumnTitle { name: c.name.clone().into(), tone: i as i32 })
                .collect(),
        ));
        courses.set_sort_column(match self.sort {
            SortKey::Code => -1,
            SortKey::Usage => -2,
            SortKey::Course(i) => i as i32,
        });
        courses.set_sort_ascending(self.ascending);
        courses.set_rows(model(
            self.sorted_codes(event)
                .into_iter()
                .map(|code| {
                    let uses = usage_count(event, code);
                    ControlRow {
                        code: code.to_string().into(),
                        unused: uses == 0,
                        shared: uses > 1,
                        selected: self.control == Some(code),
                        ranks: model(event.courses.iter().map(|c| rank_in(c, code)).collect()),
                        usage: if uses == 0 { "Dans aucun circuit".into() } else { plural(uses, "circuit").into() },
                        usage_short: if uses == 0 { "aucun".into() } else { format!("{uses} circ.").into() },
                    }
                })
                .collect(),
        ));

        // Lines
        courses.set_lines(model(
            event
                .courses
                .iter()
                .enumerate()
                .map(|(i, c)| {
                    let missing = self.control.is_some_and(|code| !c.controls.contains(&code));
                    CourseLine {
                        name: c.name.clone().into(),
                        tone: i as i32,
                        count: plural(c.controls.len(), "poste").into(),
                        dimmed: missing || (self.highlight > 0 && self.highlight - 1 != i),
                        stops: model(
                            c.controls
                                .iter()
                                .enumerate()
                                .map(|(n, code)| SequenceStop {
                                    code: code.to_string().into(),
                                    rank: n as i32 + 1,
                                    selected: self.control == Some(*code),
                                })
                                .collect(),
                        ),
                    }
                })
                .collect(),
        ));
        courses.set_unused_codes(model(unused.iter().map(|c| SharedString::from(c.to_string())).collect()));

        // Map: a course is traced when highlighted, or when it goes through the chosen control.
        let traced: Vec<&Course> = event
            .courses
            .iter()
            .enumerate()
            .filter(|(i, c)| {
                if self.highlight > 0 {
                    self.highlight - 1 == *i
                } else {
                    self.control.is_some_and(|code| c.controls.contains(&code))
                }
            })
            .map(|(_, c)| c)
            .collect();
        courses.set_traces(model(traced.iter().map(|c| SharedString::from(trace(event, c))).collect()));
        courses.set_markers(model(
            event
                .controls
                .iter()
                .map(|c| MapMarker {
                    code: c.code.to_string().into(),
                    x: c.x / MAP_WIDTH,
                    y: c.y / MAP_HEIGHT,
                    unused: usage_count(event, c.code) == 0,
                    selected: self.control == Some(c.code),
                    dimmed: self.highlight > 0 && !event.courses[self.highlight - 1].controls.contains(&c.code),
                })
                .collect(),
        ));
        courses.set_start_x(START.0 / MAP_WIDTH);
        courses.set_start_y(START.1 / MAP_HEIGHT);
        courses.set_finish_x(FINISH.0 / MAP_WIDTH);
        courses.set_finish_y(FINISH.1 / MAP_HEIGHT);

        // Detail
        let detail = self.control.filter(|code| event.controls.iter().any(|c| c.code == *code));
        courses.set_detail_open(detail.is_some());
        if let Some(code) = detail {
            let using: Vec<(usize, &Course)> =
                event.courses.iter().enumerate().filter(|(_, c)| c.controls.contains(&code)).collect();
            courses.set_detail_code(code.to_string().into());
            courses.set_detail_subtitle(
                if using.is_empty() { "dans aucun circuit".to_string() } else { plural(using.len(), "circuit") }.into(),
            );
            courses.set_detail_courses(model(
                using
                    .iter()
                    .map(|(tone, c)| {
                        let n = c.controls.iter().position(|x| *x == code).unwrap_or(0);
                        DetailCourse {
                            name: c.name.clone().into(),
                            tone: *tone as i32,
                            rank: format!("{}{}", n + 1, if n == 0 { "er" } else { "e" }).into(),
                            before: if n == 0 { "D".into() } else { c.controls[n - 1].to_string().into() },
                            after: c.controls.get(n + 1).map_or("A".into(), |x| x.to_string().into()),
                        }
                    })
                    .collect(),
            ));
            let entrants: u32 = using.iter().map(|(_, c)| c.entrants).sum();
            courses.set_detail_entrants(format!("{entrants} inscrits passent par ce poste").into());
        }
    }
}
