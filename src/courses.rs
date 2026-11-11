// The Circuits tab: what is chosen, and the rows it shows.
// Everything is display only for now: nothing here changes a course.

use crate::home::badge;
use crate::stations::StationsState;
use crate::test_data::{Control, Course, Event, FINISH, Format, Kind, Label, START};
use crate::{
    App, ColumnTitle, ControlRow, CourseItem, CourseLine, CourseMarker, CourseStop, Courses, DetailCourse, LegendItem, MapMarker, ReserveItem,
    SequenceStop, TabItem, TracedCourse, model,
};
use slint::{ComponentHandle, SharedString};

const VIEWS: [&str; 3] = ["Tableau", "Lignes", "Carte"];
const COURSE_VIEWS: [&str; 3] = ["Liste", "Métro", "Géographique"];
const COURSE_VIEWS_NARROW: [&str; 3] = ["Liste", "Métro", "Géo"];
const GEOGRAPHIC_VIEW: usize = 2;
const MAP_VIEW: usize = 2;
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
    course_view: usize,
    show_others: bool,
    stop: Option<usize>,
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
            course_view: 0,
            show_others: false,
            stop: None,
        }
    }
}

fn plural(count: usize, noun: &str) -> String {
    format!("{count} {noun}{}", if count > 1 { "s" } else { "" })
}

fn rank_text(n: usize) -> String {
    format!("{}{}", n + 1, if n == 0 { "er" } else { "e" })
}

fn word(label: &str) -> TabItem {
    TabItem { label: label.into(), icon: Default::default() }
}

fn rank_in(course: &Course, code: u32) -> i32 {
    course.controls.iter().position(|c| *c == code).map_or(0, |i| i as i32 + 1)
}

// What the table shows for a control: its rank, or -1 when the course has no order.
fn shown_rank(course: &Course, code: u32) -> i32 {
    match rank_in(course, code) {
        0 => 0,
        _ if course.format != Format::Imposed => -1,
        rank => rank,
    }
}

fn usage_count(event: &Event, code: u32) -> usize {
    event.courses.iter().filter(|c| c.controls.contains(&code)).count()
}

fn position(event: &Event, code: u32) -> Option<Control> {
    event.controls.iter().find(|c| c.code == code).copied()
}

// A course without an order has no line between its controls.
fn trace(event: &Event, course: &Course) -> String {
    if course.format != Format::Imposed {
        return String::new();
    }
    let mut commands = format!("M{} {}", START.0, START.1);
    for code in &course.controls {
        if let Some(control) = position(event, *code) {
            commands += &format!(" L{} {}", control.x, control.y);
        }
    }
    commands + &format!(" L{} {}", FINISH.0, FINISH.1)
}

fn traced(event: &Event, course: &Course, tone: usize) -> TracedCourse {
    TracedCourse { trace: trace(event, course).into(), tone: tone as i32 }
}

impl CoursesState {
    pub fn select_course(&mut self, index: i32) {
        self.selection = match index {
            -2 => {
                self.code_search.clear();
                self.course_search.clear();
                Selection::Nothing
            }
            -1 => Selection::AllControls,
            i => Selection::Course(i as usize),
        };
        self.control = None;
        self.stop = None;
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

    pub fn select_course_view(&mut self, view: usize) {
        self.course_view = view;
    }

    pub fn toggle_others(&mut self) {
        self.show_others = !self.show_others;
    }

    // Selecting the same stop again closes its detail.
    pub fn select_stop(&mut self, stop: usize) {
        self.stop = if self.stop == Some(stop) { None } else { Some(stop) };
    }

    // A control of the course selects its stop. Adding one is for later.
    pub fn select_reserve(&mut self, code: &str, event: Option<&Event>) {
        let Selection::Course(index) = self.selection else { return };
        let course = event.and_then(|e| e.courses.get(index));
        let code = code.parse::<u32>().ok();
        if let Some(stop) = course.and_then(|c| c.controls.iter().position(|x| Some(*x) == code)) {
            self.select_stop(stop);
        }
    }

    pub fn show_in_all_controls(&mut self, code: &str) {
        self.select_course(-1);
        self.control = code.parse().ok();
    }

    pub fn close_detail(&mut self) {
        self.control = None;
        self.stop = None;
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

    pub fn show(&self, window: &App, event: Option<&Event>, stations: &StationsState) {
        let courses = window.global::<Courses>();
        let Some(event) = event else { return };

        courses.set_empty(event.courses.is_empty());
        courses.set_list_only(self.selection == Selection::Nothing);
        courses.set_views(model(VIEWS.iter().map(|v| word(v)).collect()));
        courses.set_views_narrow(model(VIEWS[..2].iter().map(|v| word(v)).collect()));
        courses.set_active_view(self.view as i32);
        // The phone has no map: it shows the table as a list instead.
        courses.set_active_view_narrow(if self.view == MAP_VIEW { 0 } else { self.view as i32 });
        courses.set_code_search_text(self.code_search.as_str().into());
        courses.set_course_search_text(self.course_search.as_str().into());

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

        let (active, selected_course) = match self.selection {
            Selection::Course(i) => (i as i32, event.courses.get(i)),
            _ => (-1, None),
        };
        courses.set_active_course(active);
        courses.set_course_selected(selected_course.is_some());
        courses.set_zone_title(if selected_course.is_some() { "Tous les circuits" } else { "Tous les postes" }.into());
        let count = event.controls.len();
        let categories: usize = event.courses.iter().map(|c| c.categories.split(", ").count()).sum();
        courses.set_chips(model(if selected_course.is_some() {
            vec!["Circuit reconnu à la lecture : non".into(), "Ordre imposé".into(), format!("Catégories : {categories}").into()]
        } else {
            vec!["Analyse : rien".into(), format!("Positions : {count} / {count}").into()]
        }));
        courses.set_alert_count(unused.len().to_string().into());
        if unused.is_empty() {
            courses.set_alert_title("".into());
        } else {
            courses.set_alert_title(format!("{} dans aucun circuit", plural(unused.len(), "poste")).into());
            courses.set_alert_detail(
                if unused.len() > 1 { "Posés mais empruntés par aucun circuit" } else { "Posé mais emprunté par aucun circuit" }.into(),
            );
        }
        courses.set_summary(
            if selected_course.is_some() {
                format!("{} · {} · {}", plural(event.courses.len(), "circuit"), plural(count, "poste"), plural(categories, "catégorie"))
            } else {
                format!("{} · {} · {} dans aucun circuit", plural(count, "poste"), plural(shared, "partagé"), unused.len())
            }
            .into(),
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
                        ranks: model(event.courses.iter().map(|c| shown_rank(c, code)).collect()),
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
                        ordered: c.format == Format::Imposed,
                        stops: model(
                            c.controls
                                .iter()
                                .enumerate()
                                .map(|(n, code)| SequenceStop {
                                    code: code.to_string().into(),
                                    rank: if c.format == Format::Imposed { n as i32 + 1 } else { -1 },
                                })
                                .collect(),
                        ),
                    }
                })
                .collect(),
        ));
        courses.set_unused_codes(model(unused.iter().map(|c| SharedString::from(c.to_string())).collect()));

        // Map: a course is traced when highlighted, or when it goes through the chosen control.
        courses.set_traces(model(
            event
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
                .map(|(i, c)| traced(event, c, i))
                .filter(|t| !t.trace.is_empty())
                .collect(),
        ));
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

        if let Some(course) = selected_course {
            self.show_course(&courses, event, course, active as usize, stations);
        }

        // Detail
        let detail = self.control.filter(|code| event.controls.iter().any(|c| c.code == *code));
        let stop = selected_course.and_then(|c| self.stop.filter(|s| *s < c.controls.len()));
        courses.set_detail_open(detail.is_some() || stop.is_some());
        if let Some(code) = detail {
            let using: Vec<(usize, &Course)> =
                event.courses.iter().enumerate().filter(|(_, c)| c.controls.contains(&code)).collect();
            courses.set_detail_code(code.to_string().into());
            courses.set_detail_unused(using.is_empty());
            courses.set_detail_subtitle(
                if using.is_empty() { "dans aucun circuit".to_string() } else { plural(using.len(), "circuit") }.into(),
            );
            // A new code applies wherever the control is used (prototype: tFiche).
            courses.set_detail_code_scope(
                match using.len() {
                    0 => String::new(),
                    1 => "change dans tous les circuits".to_string(),
                    n => format!("change dans les {n} circuits"),
                }
                .into(),
            );
            courses.set_detail_courses(model(
                using
                    .iter()
                    .map(|(tone, c)| {
                        let n = c.controls.iter().position(|x| *x == code).unwrap_or(0);
                        let ordered = c.format == Format::Imposed;
                        DetailCourse {
                            name: c.name.clone().into(),
                            tone: *tone as i32,
                            ordered,
                            rank: if ordered { rank_text(n).into() } else { "libre".into() },
                            // Empty: the start or the finish, drawn with its symbol.
                            before: if n == 0 { SharedString::new() } else { c.controls[n - 1].to_string().into() },
                            after: c.controls.get(n + 1).map_or(SharedString::new(), |x| x.to_string().into()),
                        }
                    })
                    .collect(),
            ));
            let entrants: u32 = using.iter().map(|(_, c)| c.entrants).sum();
            let entrants = match entrants {
                0 => "Aucun coureur inscrit ne passe par ce poste".to_string(),
                1 => "1 coureur inscrit passe par ce poste".to_string(),
                n => format!("{n} coureurs inscrits passent par ce poste"),
            };
            courses.set_detail_entrants(entrants.into());
        }
    }

    fn show_course(&self, courses: &Courses<'_>, event: &Event, course: &Course, index: usize, stations: &StationsState) {
        courses.set_course_name(course.name.as_str().into());
        courses.set_course_map(course.map.unwrap_or("—").into());
        courses.set_course_categories(course.categories.into());
        let ordered = course.format == Format::Imposed;
        let scored = course.format == Format::Score;
        let mandatory = course.mandatory.iter().filter(|m| **m).count();
        let format_label = match course.format {
            Format::Imposed => "Ordre imposé".to_string(),
            Format::Free => "Ordre libre".to_string(),
            Format::Score => format!("Score · {}", plural(mandatory, "obligatoire")),
        };
        courses.set_course_facts(format!("{} · {}", course.categories, course.distance).into());
        let map = format!("Carte : {}", course.map.unwrap_or("—"));
        courses.set_course_chips(model(vec![format_label.as_str().into(), map.as_str().into()]));
        courses.set_course_ordered(ordered);
        courses.set_course_scored(scored);
        courses.set_free_title(
            if scored { format!("Ordre libre · au score · {}", plural(mandatory, "obligatoire")) } else { "Ordre libre".to_string() }.into(),
        );
        courses.set_course_views(model(COURSE_VIEWS.iter().map(|v| word(v)).collect()));
        courses.set_course_views_narrow(model(COURSE_VIEWS_NARROW.iter().map(|v| word(v)).collect()));
        courses.set_active_course_view(self.course_view as i32);
        courses.set_show_others(self.show_others);

        let selected = self.stop.and_then(|s| course.controls.get(s).copied());
        courses.set_stops(model(
            course
                .controls
                .iter()
                .enumerate()
                .map(|(n, code)| CourseStop {
                    code: code.to_string().into(),
                    mandatory: course.mandatory[n],
                    points: if scored { format!("{} pts", course.points[n]).into() } else { SharedString::new() },
                    label: match course.format {
                        Format::Imposed => rank_text(n).into(),
                        Format::Free => SharedString::new(),
                        Format::Score if course.mandatory[n] => format!("{} pts · obligatoire", course.points[n]).into(),
                        Format::Score => format!("{} pts", course.points[n]).into(),
                    },
                    selected: self.stop == Some(n),
                    others: model(
                        event
                            .courses
                            .iter()
                            .enumerate()
                            .filter(|(i, c)| self.show_others && *i != index && c.controls.contains(code))
                            .map(|(i, _)| i as i32)
                            .collect(),
                    ),
                })
                .collect(),
        ));

        let mut codes: Vec<u32> = event.controls.iter().map(|c| c.code).collect();
        codes.sort_unstable();
        courses.set_reserve(model(
            codes
                .iter()
                .map(|code| ReserveItem {
                    code: code.to_string().into(),
                    in_course: course.controls.contains(code),
                    selected: selected == Some(*code),
                })
                .collect(),
        ));

        courses.set_course_trace(trace(event, course).into());
        courses.set_other_traces(model(
            event
                .courses
                .iter()
                .enumerate()
                .filter(|(i, _)| self.show_others && *i != index)
                .map(|(i, c)| traced(event, c, i))
                .filter(|t| !t.trace.is_empty())
                .collect(),
        ));
        courses.set_course_markers(model(
            event
                .controls
                .iter()
                .map(|c| CourseMarker {
                    code: c.code.to_string().into(),
                    x: c.x / MAP_WIDTH,
                    y: c.y / MAP_HEIGHT,
                    in_course: course.controls.contains(&c.code),
                    mandatory: course.controls.iter().position(|x| *x == c.code).is_some_and(|n| course.mandatory[n]),
                    selected: selected == Some(c.code),
                })
                .collect(),
        ));
        courses.set_start_x(START.0 / MAP_WIDTH);
        courses.set_start_y(START.1 / MAP_HEIGHT);
        courses.set_finish_x(FINISH.0 / MAP_WIDTH);
        courses.set_finish_y(FINISH.1 / MAP_HEIGHT);

        let mut legend = vec![(0, "Départ"), (1, "Arrivée"), (2, "Poste du circuit"), (3, "Poste hors circuit")];
        if mandatory > 0 {
            legend.push((5, "Poste obligatoire"));
        }
        if self.show_others {
            legend.push(if self.course_view == GEOGRAPHIC_VIEW { (6, "Tracé d'un autre circuit") } else { (4, "Dans un autre circuit") });
        }
        courses.set_legend(model(legend.into_iter().map(|(kind, label)| LegendItem { kind, label: label.into() }).collect()));

        if let (Some(n), Some(code)) = (self.stop, selected) {
            courses.set_stop_points(format!("{} pts", course.points[n]).into());
            courses.set_stop_mandatory(course.mandatory[n]);
            // Empty: the start or the finish, drawn with its symbol in the mini-trace.
            let before = if n == 0 { String::new() } else { course.controls[n - 1].to_string() };
            let after = course.controls.get(n + 1).map_or(String::new(), u32::to_string);
            courses.set_stop_code(code.to_string().into());
            courses.set_stop_subtitle(
                if ordered {
                    format!("{} · {} sur {}", course.name, rank_text(n), course.controls.len())
                } else {
                    format!("{} · ordre libre", course.name)
                }
                .into(),
            );
            courses.set_stop_before(before.into());
            courses.set_stop_after(after.as_str().into());
            let next = if after.is_empty() { "Arrivée" } else { after.as_str() };
            courses.set_stop_section(format!("Tronçon {code} → {next} non compté pour ce circuit").into());
            let station = event.stations.iter().find(|s| s.control == Some(code));
            courses.set_stop_station(badge(&station.map_or(Label::new("Pas de boîtier", Kind::Inactive), |s| stations.state_label(s))));
            courses.set_stop_others(model(
                event
                    .courses
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| *i != index)
                    .filter_map(|(i, c)| c.controls.iter().position(|x| *x == code).map(|n| (i, c, n)))
                    .map(|(i, c, n)| {
                        let ordered = c.format == Format::Imposed;
                        DetailCourse {
                            name: c.name.clone().into(),
                            tone: i as i32,
                            ordered,
                            rank: if ordered { format!("{} poste", rank_text(n)).into() } else { "libre".into() },
                            before: SharedString::new(),
                            after: SharedString::new(),
                        }
                    })
                    .collect(),
            ));
        }
    }
}
