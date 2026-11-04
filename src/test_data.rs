// The test data generator, started from the Démo panel. It stands in for what connected
// devices and real events will provide. When it is off, the app has no data at all.
// All names, clubs and card numbers are invented.

#[derive(Clone, Copy, Default, PartialEq)]
pub enum Kind {
    #[default]
    None,
    Valid,
    Racing,
    ToCheck,
    Inactive,
    Ok,
    Pm,
    Unknown,
}

#[derive(Clone, Default)]
pub struct Label {
    pub word: String,
    pub kind: Kind,
}

impl Label {
    pub fn new(word: impl Into<String>, kind: Kind) -> Self {
        Label { word: word.into(), kind }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum Format {
    Imposed,
    Free,
    Score,
}

pub struct Course {
    pub name: String,
    pub format: Format,
    pub points: Vec<u32>,       // one per control, for a score course
    pub mandatory: Vec<bool>,   // one per control, for a score course
    pub categories: &'static str,
    pub distance: String,
    pub entrants: u32,
    pub controls: Vec<u32>,
    pub start_point: u32,
}

// A control on the map, in a 600 × 360 drawing.
#[derive(Clone, Copy)]
pub struct Control {
    pub code: u32,
    pub x: f32,
    pub y: f32,
}

// One course run by a runner: its time and whether it is valid, once back.
pub struct Run {
    pub course: usize,
    pub result: Option<(u32, bool)>,
}

pub struct Runner {
    pub family: &'static str,
    pub first: &'static str,
    pub club: &'static str,
    pub category: String,
    pub unknown_category: bool,
    pub card: Option<u32>,
    pub lent: bool,
    pub start: u32,
    pub started: bool,
    pub locked: bool,
    pub changed: bool,      // its start time moved since the publication
    pub runs: Vec<Run>,
}

#[derive(Clone, Copy, PartialEq)]
pub enum StationState {
    ToTakeOut,
    ToProgram,
    Missing,
    Checked,
    ClockGap(u32),
    Read(u32),      // its memory is read, not yet added to the event
    Back(u32),
}

// What the event needs: a control, a start, a finish, a clear or a check station.
pub struct Station {
    pub name: String,
    pub code: String,
    pub control: Option<u32>,
    pub serial: Option<u32>,
    pub park: &'static str,
    pub low_battery: bool,
    pub state: StationState,
}

pub struct Vacant {
    pub course: usize,
    pub time: u32,
}

pub struct StartGroup {
    pub name: String,
    pub rule: String,
    pub members: Vec<usize>,
}

impl Runner {
    pub fn name(&self) -> String {
        format!("{} {}", self.family, self.first)
    }

    pub fn is_racing(&self) -> bool {
        self.started && self.runs.iter().any(|r| r.result.is_none())
    }

    pub fn is_back(&self) -> bool {
        self.started && !self.is_racing()
    }

    pub fn label(&self) -> Label {
        if !self.started {
            Label::new("Inscrit", Kind::Inactive)
        } else if self.is_racing() {
            Label::new("En course", Kind::Racing)
        } else if self.runs.iter().all(|r| matches!(r.result, Some((_, true)))) {
            Label::new("Arrivé", Kind::Ok)
        } else {
            Label::new("PM", Kind::Pm)
        }
    }
}

pub struct Event {
    pub name: String,
    pub weekday: &'static str,
    pub day: u32,
    pub month: &'static str,
    pub month_short: &'static str,
    pub place: &'static str,
    pub participants: String,
    pub station: u32,
    pub runners: Vec<Runner>,
    pub stations: Vec<Station>,
    pub vacants: Vec<Vacant>,
    pub groups: Vec<StartGroup>,
    pub published: Option<String>,
    pub courses: Vec<Course>,
    pub controls: Vec<Control>,
    pub label: Label,
}

impl Event {
    pub fn detail(&self) -> String {
        format!("{} {} {} · {}", self.weekday, self.day, self.month, self.place)
    }
}

pub struct Reading {
    pub label: Label,
    pub name: String,
    pub time: String,
    pub clock: String,
}

pub struct ClubFigures {
    pub cards_free: u32,
    pub cards_lent: u32,
    pub cards_overdue: u32,
    pub stations: u32,
    pub station_parks: u32,
    pub low_batteries: u32,
    pub runners: u32,
    pub clubs: u32,
}

pub struct Generated {
    pub events: Vec<Event>,
    pub upcoming: Vec<usize>,
    pub past: Vec<usize>,
    pub to_handle: u32,
    pub readers: Vec<String>,
    pub readings: Vec<Reading>,
    pub club: ClubFigures,
    pub devices_asking: u32,
    pub screens: u32,
    today: u32,
    clock_seconds: u32,
    random: Random,
}

// Where courses start and finish, on the same 600 × 360 map as the controls.
pub const START: (f32, f32) = (40.0, 330.0);
pub const FINISH: (f32, f32) = (565.0, 335.0);

// Two runners of a course start this many seconds apart.
pub const START_STEP: u32 = 120;

// The current event is always the first of the list.
pub const CURRENT_EVENT: usize = 0;

// A small pseudo-random generator: enough for test data, without a library.
struct Random(u64);

impl Random {
    fn below(&mut self, max: u32) -> u32 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        ((self.0 >> 33) as u32) % max
    }

    fn between(&mut self, low: u32, high: u32) -> u32 {
        low + self.below(high - low + 1)
    }

    fn pick<T: Copy>(&mut self, list: &[T]) -> T {
        list[self.below(list.len() as u32) as usize]
    }
}

const FAMILY_NAMES: [&str; 12] = [
    "GIRAUD", "MARTIN", "ROUX", "DUPONT", "LEROY", "DUMAS", "NOËL", "BERNARD", "FAVRE", "PETIT", "ROBIN", "MOREL",
];
const FIRST_NAMES: [&str; 10] = ["Ana", "Léa", "Tom", "Marc", "Zoé", "Paul", "Anna", "Lou", "Luc", "Inès"];
const EVENT_NAMES: [&str; 8] = [
    "Régionale de Lyon",
    "Lab'O de la fête du club",
    "Entraînement du mercredi",
    "Nocturne de Fourvière",
    "Départementale Rhône",
    "Sprint des Pentes",
    "Relais de l'Ouest lyonnais",
    "Moyenne distance du Beaujolais",
];
const PLACES: [&str; 8] = [
    "Forêt de Brou",
    "Gymnase Jean-Moulin",
    "Parc de Parilly",
    "Lyon 5e",
    "Parc de la Tête d'Or",
    "Monts d'Or",
    "Bois de Serres",
    "Col de la Luère",
];
const WEEKDAYS: [&str; 7] = ["Lundi", "Mardi", "Mercredi", "Jeudi", "Vendredi", "Samedi", "Dimanche"];
const MONTHS: [(&str, &str, u32); 4] =
    [("octobre", "oct.", 31), ("novembre", "nov.", 30), ("décembre", "déc.", 31), ("janvier", "janv.", 31)];

// A date as a number of days after 1 October 2026, which is a Thursday.
fn date(days: u32) -> (&'static str, u32, &'static str, &'static str) {
    let weekday = WEEKDAYS[((days + 3) % 7) as usize];
    let mut left = days;
    for (month, short, length) in MONTHS {
        if left < length {
            return (weekday, left + 1, month, short);
        }
        left -= length;
    }
    (weekday, 31, "janvier", "janv.")
}

pub fn format_time(seconds: u32) -> String {
    format!("{}:{:02}", seconds / 60, seconds % 60)
}

pub fn format_clock(seconds: u32) -> String {
    format!("{}:{:02}", seconds / 3600, seconds / 60 % 60)
}

const COURSE_NAMES: [&str; 5] = ["Circuit A", "Circuit B", "Circuit C", "Circuit D", "Jaune"];
const CATEGORIES: [&str; 5] = ["H21E, H20", "D21E, H35", "H16, D16, D40", "H18, D18", "Open jaune"];
const CLUBS: [&str; 6] = ["ASO Lyon", "SMOC", "TAD", "COLE", "CO Brou", "OTB"];
const ALL_CATEGORIES: [&str; 6] = ["H21E", "D21E", "H35", "D40", "H16", "D18"];
const UNKNOWN_CATEGORY: &str = "H70";
const MAP_COLUMNS: u32 = 5;
const MAP_ROWS: u32 = 3;

// Controls sit on a grid, moved a little, so that they and their tags do not overlap.
fn new_controls(random: &mut Random) -> Vec<Control> {
    let mut cells: Vec<u32> = (0..MAP_COLUMNS * MAP_ROWS).collect();
    let wanted = random.between(9, 13) as usize;
    let mut codes: Vec<u32> = (31..=99).collect();
    let mut controls = Vec::new();
    for _ in 0..wanted {
        let cell = cells.remove(random.below(cells.len() as u32) as usize);
        let code = codes.remove(random.below(codes.len() as u32) as usize);
        controls.push(Control {
            code,
            x: 70.0 + (cell % MAP_COLUMNS * 105) as f32 + random.below(30) as f32,
            y: 45.0 + (cell / MAP_COLUMNS * 90) as f32 + random.below(25) as f32,
        });
    }
    controls.sort_by_key(|c| c.code);
    controls
}

fn new_courses(random: &mut Random, count: u32, controls: &[Control]) -> Vec<Course> {
    (0..count as usize)
        .map(|i| {
            let length = random.between(3, 7) as usize;
            let mut pool: Vec<u32> = controls.iter().map(|c| c.code).collect();
            // The last control of the list is sometimes used by no course, as in real life.
            pool.pop();
            let mut chosen = Vec::new();
            while chosen.len() < length && !pool.is_empty() {
                chosen.push(pool.remove(random.below(pool.len() as u32) as usize));
            }
            // The last courses show the other formats. Their controls go by code: no order to follow.
            let format = match count as usize - i {
                1 => Format::Score,
                2 => Format::Free,
                _ => Format::Imposed,
            };
            if format != Format::Imposed {
                chosen.sort_unstable();
            }
            let score = format == Format::Score;
            let points = chosen.iter().map(|_| if score { random.between(1, 3) * 10 } else { 0 }).collect();
            let mandatory = chosen.iter().map(|_| score && random.below(4) == 0).collect();
            let kilometers = random.between(20, 75);
            Course {
                name: COURSE_NAMES[i].into(),
                format,
                points,
                mandatory,
                categories: CATEGORIES[i],
                distance: format!("{},{} km · {} m D+", kilometers / 10, kilometers % 10, random.between(4, 22) * 10),
                entrants: random.between(10, 80),
                controls: chosen,
                start_point: 1,
            }
        })
        .collect()
}

#[derive(Clone, Copy, PartialEq)]
enum Moment {
    Past,
    Today,
    Upcoming,
}

fn new_runners(random: &mut Random, courses: &[Course], moment: Moment) -> Vec<Runner> {
    let count = random.between(30, 60) as usize;
    let mut runners: Vec<Runner> = Vec::new();
    for _ in 0..count {
        // Two different people: the same name only comes later, on purpose.
        let (mut family, mut first) = (random.pick(&FAMILY_NAMES), random.pick(&FIRST_NAMES));
        while runners.iter().any(|r| r.family == family && r.first == first) {
            family = random.pick(&FAMILY_NAMES);
            first = random.pick(&FIRST_NAMES);
        }
        let has_card = moment == Moment::Past || random.below(12) > 0;
        let lent = has_card && random.below(12) == 0;
        let started = has_card && (moment == Moment::Past || (moment == Moment::Today && random.below(5) > 0));
        let done = moment == Moment::Past || random.below(2) == 0;
        let mut runs = Vec::new();
        let mut passages = if random.below(8) == 0 { 2 } else { 1 };
        let mut course = if courses.is_empty() { 0 } else { random.below(courses.len() as u32) as usize };
        let categories: Vec<&str> =
            courses.get(course).map_or(ALL_CATEGORIES.to_vec(), |c| c.categories.split(", ").collect());
        while passages > 0 && !courses.is_empty() {
            let result = (started && done).then(|| (random.between(1500, 4200), random.below(8) > 0));
            runs.push(Run { course, result });
            course = (course + 1) % courses.len();
            passages -= 1;
        }
        runners.push(Runner {
            family,
            first,
            club: random.pick(&CLUBS),
            category: random.pick(&categories).to_string(),
            unknown_category: false,
            card: has_card.then(|| if lent { random.between(2_040_000, 2_049_999) } else { random.between(8_100_000, 8_199_999) }),
            lent,
            start: 0,
            started,
            locked: false,
            changed: false,
            runs,
        });
    }
    // Things to check after an import: a possible duplicate, an unknown category.
    if moment != Moment::Past {
        for _ in 0..random.below(3) {
            let original = random.below(runners.len() as u32) as usize;
            let (family, first) = (runners[original].family, runners[original].first);
            let category = runners[original].category.clone();
            let course = runners[original].runs.first().map(|r| r.course);
            runners.push(Runner {
                family,
                first,
                club: random.pick(&CLUBS),
                category,
                unknown_category: false,
                card: None,
                lent: false,
                start: 0,
                started: false,
                locked: false,
                changed: false,
                runs: course.map(|course| Run { course, result: None }).into_iter().collect(),
            });
        }
        if random.below(2) == 0 {
            let last = runners.len() - 1;
            runners[last].category = UNKNOWN_CATEGORY.into();
            runners[last].unknown_category = true;
        }
    }
    runners
}

// Each course starts at the same time, one runner every step, then one vacant start.
fn place_starts(random: &mut Random, runners: &mut [Runner], courses: usize, first_start: u32, published: bool) -> Vec<Vacant> {
    let mut next = vec![first_start; courses];
    let mut others = first_start;
    for runner in runners.iter_mut() {
        let slot = runner.runs.first().map_or(&mut others, |r| &mut next[r.course]);
        runner.start = *slot;
        *slot += START_STEP;
        if !runner.started {
            runner.locked = random.below(10) == 0;
            runner.changed = published && random.below(10) == 0;
        }
    }
    next.iter().enumerate().map(|(course, time)| Vacant { course, time: *time }).collect()
}

fn new_stations(random: &mut Random, event_controls: &[Control], points: u32, moment: Moment) -> Vec<Station> {
    let mut needs: Vec<(String, String, Option<u32>)> = Vec::new();
    for point in 1..=points {
        needs.push((format!("Départ S{point}"), format!("S{point}"), None));
    }
    needs.push(("Arrivée F1".into(), "F1".into(), None));
    needs.extend(event_controls.iter().map(|c| (format!("Poste {}", c.code), c.code.to_string(), Some(c.code))));
    needs.push(("Effacer E1".into(), "E1".into(), None));
    needs.push(("Contrôler C1".into(), "C1".into(), None));

    let count = needs.len() as u32;
    // A few needs still to handle before leaving, on controls only.
    let special = |random: &mut Random| random.between(points + 1, (count - 3).max(points + 1)) as usize;
    let (program, missing, gap) = (special(random), special(random), special(random));
    needs
        .into_iter()
        .enumerate()
        .map(|(i, (name, code, control))| {
            let mut state = match random.below(5) {
                0 => StationState::ToTakeOut,
                _ => StationState::Checked,
            };
            if moment == Moment::Past {
                state = StationState::Back(random.between(20, 120));
            } else if i == program {
                state = StationState::ToProgram;
            } else if i == missing {
                state = StationState::Missing;
            } else if i == gap {
                state = StationState::ClockGap(random.between(3, 40));
            }
            let has_station = !matches!(state, StationState::Missing | StationState::ToProgram);
            Station {
                name,
                code,
                control,
                serial: has_station.then(|| random.between(200_000, 209_999)),
                park: if random.below(6) == 0 { "Secours" } else { "Club" },
                low_battery: has_station && random.below(10) == 0,
                state,
            }
        })
        .collect()
}

fn new_groups(random: &mut Random, runners: &[Runner], first_start: u32) -> Vec<StartGroup> {
    let in_course: Vec<usize> = (0..runners.len()).filter(|i| !runners[*i].runs.is_empty()).collect();
    if in_course.is_empty() {
        return Vec::new();
    }
    let club = runners[random.pick(&in_course)].club;
    let family = runners[random.pick(&in_course)].family;
    let helpers = in_course.iter().copied().filter(|_| random.below(10) == 0).collect();
    vec![
        StartGroup {
            name: club.to_string(),
            rule: "Ensemble".into(),
            members: in_course.iter().copied().filter(|i| runners[*i].club == club).collect(),
        },
        StartGroup {
            name: format!("Famille {family}"),
            rule: "Séparés de 10 min".into(),
            members: in_course.iter().copied().filter(|i| runners[*i].family == family).collect(),
        },
        StartGroup { name: "Bénévoles".into(), rule: format!("Avant {}", format_clock(first_start + 3 * START_STEP)), members: helpers },
    ]
}

fn new_event(random: &mut Random, name: &str, days: u32, moment: Moment) -> Event {
    let past = moment == Moment::Past;
    let (weekday, day, month, month_short) = date(days);
    // An upcoming event sometimes has no course yet.
    let course_count = if !past && random.below(4) == 0 { 0 } else { random.between(3, 5) };
    let controls = if course_count == 0 { Vec::new() } else { new_controls(random) };
    let mut courses = new_courses(random, course_count, &controls);
    // Sometimes a second start, for the courses of the second half.
    if random.below(2) == 0 {
        let half = courses.len() / 2;
        courses.iter_mut().skip(half).for_each(|c| c.start_point = 2);
    }
    let mut runners = new_runners(random, &courses, moment);
    let first_start = random.between(9, 11) * 3600;
    let published = past || random.below(2) == 0;
    let vacants = place_starts(random, &mut runners, courses.len(), first_start, published && !past);
    let groups = new_groups(random, &runners, first_start);
    let points = courses.iter().map(|c| c.start_point).max().unwrap_or(1);
    let stations = new_stations(random, &controls, points, moment);
    let published = published.then(|| {
        let (_, day, _, month) = date(days.saturating_sub(3));
        format!("Publiés le {day} {month}")
    });
    for (i, course) in courses.iter_mut().enumerate() {
        course.entrants = runners.iter().filter(|r| r.runs.iter().any(|run| run.course == i)).count() as u32;
    }
    let entered = runners.len();
    let label = if past {
        Label::new("Résultats publiés", Kind::Inactive)
    } else if course_count == 0 {
        Label::new("À préparer", Kind::ToCheck)
    } else {
        Label::new(format!("{course_count} circuits prêts"), Kind::Valid)
    };
    Event {
        name: name.into(),
        weekday,
        day,
        month,
        month_short,
        place: random.pick(&PLACES),
        participants: format!("{entered} {}", if past { "participants" } else { "inscrits" }),
        station: random.between(100_000, 999_999),
        runners,
        stations,
        vacants,
        groups,
        published,
        courses,
        controls,
        label,
    }
}

impl Generated {
    pub fn new(seed: u64) -> Self {
        let mut random = Random(seed);
        let first = random.below(EVENT_NAMES.len() as u32) as usize;
        let name = |i: usize| EVENT_NAMES[(first + i) % EVENT_NAMES.len()];
        let today = 40 + random.below(30);

        // The current event always has courses, so that runners can be out on them.
        let mut events = vec![new_event(&mut random, name(0), today, Moment::Today)];
        while events[CURRENT_EVENT].courses.is_empty() {
            events[CURRENT_EVENT] = new_event(&mut random, name(0), today, Moment::Today);
        }
        events[CURRENT_EVENT].label = Label::default();
        let (mut upcoming, mut past) = (Vec::new(), Vec::new());
        let mut days = today;
        for i in 1..=3 {
            days += random.between(3, 12);
            upcoming.push(events.len());
            events.push(new_event(&mut random, name(i), days, Moment::Upcoming));
        }
        let mut days = today;
        for i in 4..=5 {
            days -= random.between(4, 16);
            past.push(events.len());
            events.push(new_event(&mut random, name(i), days, Moment::Past));
        }

        let club = ClubFigures {
            cards_free: random.between(10, 30),
            cards_lent: random.below(4),
            cards_overdue: random.below(3),
            stations: random.between(30, 60),
            station_parks: random.between(1, 3),
            low_batteries: random.below(4),
            runners: random.between(11_000, 14_000),
            clubs: random.between(180, 240),
        };
        Generated {
            events,
            upcoming,
            past,
            to_handle: 0,
            readers: vec!["Arrivée 1".into(), "Arrivée 2".into()],
            readings: Vec::new(),
            club,
            devices_asking: random.below(2),
            screens: random.below(3),
            today,
            clock_seconds: random.between(10, 14) * 3600,
            random,
        }
    }

    pub fn racing(&self) -> u32 {
        self.events[CURRENT_EVENT].runners.iter().filter(|r| r.is_racing()).count() as u32
    }

    pub fn back(&self) -> u32 {
        self.events[CURRENT_EVENT].runners.iter().filter(|r| r.is_back()).count() as u32
    }

    // Draws again at random inside each course, without touching who is locked or already gone.
    // Returns the proposed time of each runner: nothing changes until it is applied.
    pub fn draw(&mut self, event: usize) -> Vec<u32> {
        let random = &mut self.random;
        let runners = &self.events[event].runners;
        let mut times: Vec<u32> = runners.iter().map(|r| r.start).collect();
        for course in 0..self.events[event].courses.len() {
            let free: Vec<usize> = (0..runners.len())
                .filter(|i| runners[*i].runs.first().is_some_and(|r| r.course == course) && !runners[*i].locked && !runners[*i].started)
                .collect();
            let mut left: Vec<u32> = free.iter().map(|i| times[*i]).collect();
            for index in free {
                times[index] = left.remove(random.below(left.len() as u32) as usize);
            }
        }
        times
    }

    pub fn random_punches(&mut self) -> u32 {
        self.random.between(20, 120)
    }

    pub fn publish(&mut self, event: usize) {
        let today = date(self.today);
        let event = &mut self.events[event];
        event.published = Some(format!("Publiés le {} {}", today.1, today.3));
        event.runners.iter_mut().for_each(|r| r.changed = false);
    }

    // A runner out on a course comes back and reads their card. Now and then, a card nobody knows.
    pub fn read_a_card(&mut self) {
        let random = &mut self.random;
        let seconds = random.between(1500, 4200);
        self.clock_seconds += random.between(5, 40);
        let racing: Vec<usize> = self.events[CURRENT_EVENT]
            .runners
            .iter()
            .enumerate()
            .filter(|(_, r)| r.is_racing())
            .map(|(i, _)| i)
            .collect();
        if racing.is_empty() {
            return;
        }
        let (label, name) = if random.below(12) == 0 {
            self.to_handle += 1;
            (Label::new("Inconnue", Kind::Unknown), format!("Puce {}", random.between(2_000_000, 9_999_999)))
        } else {
            let valid = random.below(8) > 0;
            let runner = &mut self.events[CURRENT_EVENT].runners[random.pick(&racing)];
            if let Some(run) = runner.runs.iter_mut().find(|r| r.result.is_none()) {
                run.result = Some((seconds, valid));
            }
            (if valid { Label::new("OK", Kind::Ok) } else { Label::new("PM", Kind::Pm) }, runner.name())
        };
        self.readings.insert(
            0,
            Reading { label, name, time: format_time(seconds), clock: format_clock(self.clock_seconds) },
        );
        self.readings.truncate(READINGS_SHOWN);
    }
}

const READINGS_SHOWN: usize = 3;
