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
    pub map: Option<&'static str>,  // its map file, when one is given
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

// What a card reading still asks someone to decide.
#[derive(Clone, Copy, PartialEq)]
pub enum Issue {
    UnknownCard,
    AlreadyRead,
}

// One card read at the finish. The live feed keeps them all, oldest first.
pub struct Reading {
    pub runner: Option<usize>,  // none for a card nobody knows
    pub course: usize,
    pub card: u32,
    pub seconds: u32,
    pub valid: bool,
    pub at: u32,
    pub reader: usize,
    pub issue: Option<Issue>,
    pub settled_at: Option<u32>,
}

impl Reading {
    pub fn to_handle(&self) -> bool {
        self.issue.is_some() && self.settled_at.is_none()
    }
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
    pub closing: u32,
    pub readings: Vec<Reading>,
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
    pub fn to_handle(&self) -> usize {
        self.readings.iter().filter(|r| r.to_handle()).count()
    }

    pub fn reading_label(&self, reading: &Reading) -> Label {
        match reading.issue {
            Some(Issue::UnknownCard) if reading.settled_at.is_none() => Label::new("Inconnue", Kind::Unknown),
            Some(Issue::AlreadyRead) if reading.settled_at.is_none() => Label::new("Déjà lue", Kind::ToCheck),
            _ if reading.valid => Label::new("OK", Kind::Ok),
            _ => Label::new("PM", Kind::Pm),
        }
    }

    pub fn reading_name(&self, reading: &Reading) -> String {
        reading.runner.map_or(format!("Puce {}", reading.card), |i| self.runners[i].name())
    }

    // Among the valid results of a ranking, how many are better, plus one. A ranking is a
    // course, or one category on a course.
    pub fn place(&self, course: usize, category: Option<&str>, seconds: u32) -> usize {
        let better = self
            .runners
            .iter()
            .filter(|r| category.is_none_or(|c| r.category == c))
            .flat_map(|r| &r.runs)
            .filter(|run| run.course == course && matches!(run.result, Some((time, true)) if time < seconds));
        better.count() + 1
    }

    // Who a card nobody knows may belong to: runners still out on the course it matches.
    pub fn candidates(&self, course: usize) -> Vec<usize> {
        (0..self.runners.len())
            .filter(|i| {
                let runner = &self.runners[*i];
                runner.started && runner.runs.iter().any(|r| r.course == course && r.result.is_none())
            })
            .collect()
    }

    pub fn give_reading_to(&mut self, reading: usize, runner: usize, clock: u32) {
        let entry = &mut self.readings[reading];
        let result = (entry.seconds, entry.valid);
        entry.runner = Some(runner);
        entry.settled_at = Some(clock);
        entry.card = self.runners[runner].card.unwrap_or(entry.card);
        let course = entry.course;
        if let Some(run) = self.runners[runner].runs.iter_mut().find(|r| r.course == course && r.result.is_none()) {
            run.result = Some(result);
        }
    }

    pub fn keep_first_result(&mut self, reading: usize, clock: u32) {
        self.readings[reading].settled_at = Some(clock);
    }

    pub fn detail(&self) -> String {
        format!("{} {} {} · {}", self.weekday, self.day, self.month, self.place)
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum CardState {
    InService,
    ToCheck,
    Broken,
    Lost,
}

// Where a loan card is. A lost card keeps its last known place.
#[derive(Clone, PartialEq)]
pub enum CardPlace {
    Park,
    Lent { who: String, event: String, date: String },
    NotReturned { who: String, event: String, since: String },
}

pub struct CardEntry {
    pub date: String,
    pub text: String,
}

pub struct LoanCard {
    pub number: u32,
    pub kind: &'static str,
    pub state: CardState,
    pub state_since: String,
    pub place: CardPlace,
    pub battery: Option<&'static str>,
    pub last_borrower: Option<String>,
    pub loans: u32,
    pub note: &'static str,
    pub history: Vec<CardEntry>,    // the latest first
}

pub struct CardPark {
    pub name: &'static str,
    pub description: &'static str,
    pub cards: Vec<LoanCard>,
}

#[derive(Clone, Copy, PartialEq)]
pub enum StockState {
    InService,
    ToCheck,
    BatteryDue,
    InRepair,
    Lost,
    Destroyed,
}

// A station of the club, whichever event it goes to.
pub struct StockStation {
    pub serial: u32,
    pub code: String,
    pub kind: &'static str,
    pub state: StockState,
    pub state_since: String,
    pub battery_year: u32,
    pub charge: Option<u32>,        // in hundredths of a volt, at the last reading
    pub clock_gap: Option<u32>,     // in seconds, at the last reading
    pub measured: Option<String>,
    pub note: &'static str,
    pub history: Vec<CardEntry>,    // the latest first
}

impl StockStation {
    pub fn old_battery(&self) -> bool {
        BATTERY_YEAR - self.battery_year > BATTERY_MAX_YEARS
    }

    pub fn low_charge(&self) -> bool {
        self.charge.is_some_and(|charge| charge < LOW_CHARGE)
    }

    pub fn clock_off(&self) -> bool {
        self.clock_gap.is_some_and(|gap| gap > CLOCK_MAX_GAP)
    }

    // Only a battery changes a state by itself: the other states are chosen by someone.
    fn settle(&mut self) {
        if self.state == StockState::InService && (self.old_battery() || self.low_charge()) {
            self.state = StockState::BatteryDue;
        }
    }
}

pub struct StationPark {
    pub name: &'static str,
    pub description: &'static str,
    pub stations: Vec<StockStation>,
}

// One person of the club's local base: the federal archive, and the cards typed by hand.
pub struct BaseRunner {
    pub id: u32,
    pub family: &'static str,
    pub first: &'static str,
    pub club: &'static str,
    pub club_code: &'static str,
    pub sex: &'static str,
    pub year: u32,
    pub category: String,
    pub licence: Option<u32>,       // none: typed by hand
    pub licence_kind: Option<LicenceKind>,
    pub renewed: bool,
    pub card: Option<u32>,
    pub points: Option<(u32, String)>,
    pub entered_on: String,         // when it was typed by hand
}

#[derive(Clone, Copy, PartialEq)]
pub enum LicenceKind {
    Competition,
    Discovery,
    Leisure,
    Handisport,
}

impl BaseRunner {
    pub fn name(&self) -> String {
        format!("{} {}", self.family, self.first)
    }
}

// A reader plugged into this device (prototype: RD). Pinned, no window takes it by itself.
pub struct Reader {
    pub name: String,
    pub serial: u32,
    pub pinned: bool,
    pub unplugged: bool,
    pub usage: Option<usize>,   // in READER_USAGES; None: a free reader
    pub course: usize,          // in Generated::reader_courses
}

// A device linked to the course (prototype: DEVS).
#[derive(Clone, Copy, PartialEq)]
pub enum DeviceKind {
    Laptop,
    Phone,
    Tablet,
}

pub struct LinkedDevice {
    pub kind: DeviceKind,
    pub name: String,
    pub role: usize,              // in DEVICE_ROLES
    pub waiting: Option<u32>,     // offline: how many readings wait to be sent
}

// A device asking to join, with the word it shows, to check it is the right one.
pub struct DeviceRequest {
    pub kind: DeviceKind,
    pub name: String,
    pub role: usize,
    pub word: &'static str,
}

// A public screen plugged into this device, or a free window (prototype: W.E).
pub struct PublicScreen {
    pub name: String,
    pub size: Option<&'static str>,   // None: a free window
    pub on: bool,
    pub content: usize,               // in SCREEN_CONTENTS
    pub readers: [Option<usize>; 2],  // reading: the readers shown left and right
    pub provisional: bool,            // reading: the provisional ranking
    pub names: bool,                  // start clock: the starters' names
    pub beeps: bool,                  // start clock: beeps at −10 s, −5 s and the top
}

pub const SCREEN_CONTENTS: [&str; 4] = ["Lecture de puce", "Horloge de départ", "Entrée du sas", "Résultats"];

impl PublicScreen {
    fn new(name: String, size: Option<&'static str>, on: bool, content: usize) -> Self {
        PublicScreen { name, size, on, content, readers: [Some(0), Some(1)], provisional: true, names: true, beeps: true }
    }
}

pub const DEVICE_ROLES: [&str; 5] = ["GEC", "Départ", "Arrivée", "Poste radio", "Écran public"];
const CHECK_WORDS: [&str; 4] = ["SAPIN", "CHÊNE", "BOULEAU", "ÉRABLE"];

// What a reader is used for (prototype: USES); its name follows.
pub const READER_USAGES: [&str; 4] = ["Arrivée", "Accueil", "Départ", "Boîtiers"];

// What a reader is doing now.
#[derive(Clone, Copy, PartialEq)]
pub enum ReaderStatus {
    Ready,
    Reading,
    Unplugged,
}

// The readers, in this order: the two finish readers of the event, the entry desk, a free one.
pub const FINISH_READERS: usize = 2;
pub const ENTRY_READER: usize = 2;
pub const FREE_READER: usize = 3;
// A card stays « being read » this long on its reader, and the last reading « live » this long.
pub const READING_SECONDS: u32 = 2;
pub const LIVE_SECONDS: u32 = 15;

pub struct Generated {
    pub events: Vec<Event>,
    pub upcoming: Vec<usize>,
    pub past: Vec<usize>,
    pub readers: Vec<Reader>,
    pub runner_base: Vec<BaseRunner>,
    pub archive_date: String,
    pub archive_date_numeric: String,   // dd/mm/yyyy
    pub card_parks: Vec<CardPark>,
    pub station_parks: Vec<StationPark>,
    pub devices: Vec<LinkedDevice>,
    pub request: Option<DeviceRequest>,   // a device asking to join the course
    pub screens: Vec<PublicScreen>,
    today: u32,
    clock_seconds: u32,
    next_reading: u32,
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

pub fn format_clock_seconds(seconds: u32) -> String {
    format!("{}:{:02}:{:02}", seconds / 3600, seconds / 60 % 60, seconds % 60)
}

// The control a runner missed on a course without a valid result, always the same one.
pub fn missed_control(seconds: u32, controls: usize) -> usize {
    seconds as usize % controls.max(1)
}

// Of the punches read on a station back from the forest, those of cards not yet in the event:
// always one to three, so that the reading card has something to say.
pub fn punches_not_in_event(punches: u32) -> usize {
    punches as usize % 3 + 1
}

const COURSE_NAMES: [&str; 5] = ["Circuit A", "Circuit B", "Circuit C", "Circuit D", "Jaune"];
const CATEGORIES: [&str; 5] = ["H21E, H20", "D21E, H35", "H16, D16, D40", "H18, D18", "Open jaune"];
// The fifth course has no map yet: both cases are shown.
const MAP_FILES: [Option<&str>; 5] =
    [Some("Forêt de Brou.ocd"), Some("Forêt de Brou.ocd"), Some("Forêt de Brou.ocd"), Some("Forêt de Brou.ocd"), None];
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
                map: MAP_FILES[i],
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
    needs.push(("Effacer 1".into(), "E1".into(), None));
    needs.push(("Contrôler 1".into(), "C1".into(), None));

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

// Who is already back has been read: the feed starts with them, oldest first.
fn first_readings(random: &mut Random, runners: &[Runner], courses: &[Course], moment: Moment) -> Vec<Reading> {
    let mut readings = Vec::new();
    for (index, runner) in runners.iter().enumerate() {
        let mut at = runner.start;
        for run in &runner.runs {
            let Some((seconds, valid)) = run.result else { continue };
            at += seconds;
            readings.push(Reading {
                runner: Some(index),
                course: run.course,
                card: runner.card.unwrap_or_default(),
                seconds,
                valid,
                at,
                reader: random.below(2) as usize,
                issue: None,
                settled_at: None,
            });
        }
    }
    readings.sort_by_key(|r| r.at);
    // Today, two readings wait for someone: a card nobody knows, and one read twice.
    if moment == Moment::Today && !courses.is_empty() {
        let last = readings.last().map_or(0, |r| r.at);
        if let Some(twice) = readings.last().filter(|_| readings.len() > 1).map(|r| Reading {
            runner: r.runner,
            course: r.course,
            card: r.card,
            seconds: r.seconds,
            valid: r.valid,
            at: last + random.between(20, 60),
            reader: random.below(2) as usize,
            issue: Some(Issue::AlreadyRead),
            settled_at: None,
        }) {
            readings.push(twice);
        }
        readings.push(Reading {
            runner: None,
            course: random.below(courses.len() as u32) as usize,
            card: random.between(2_000_000, 9_999_999),
            seconds: random.between(1500, 4200),
            valid: random.below(8) > 0,
            at: last + random.between(70, 120),
            reader: random.below(2) as usize,
            issue: Some(Issue::UnknownCard),
            settled_at: None,
        });
    }
    readings
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
    // Today, at least one result that does not count: the feed must show one.
    if moment == Moment::Today
        && let Some(run) = runners.iter_mut().filter(|r| r.is_back()).flat_map(|r| &mut r.runs).nth(3)
    {
        run.result = run.result.map(|(seconds, _)| (seconds, false));
    }
    let readings = first_readings(random, &runners, &courses, moment);
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
        closing: first_start + 5 * 3600,
        readings,
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

const CARD_KINDS: [&str; 6] = ["SI-Card 8", "SI-Card 9", "SI-Card 10", "SI-Card 10", "SI-Card 11", "SIAC"];
const CARD_BATTERIES: [&str; 5] = ["09/2024", "03/2025", "11/2025", "03/2026", "06/2026"];
const CARD_NOTES: [&str; 3] = ["Bouton qui accroche", "Rangée à part", "Pile faible"];
const STOCK_SEED: u64 = 0x5EED;

// The same date as 14/10/2026.
fn numeric_date(days: u32) -> String {
    let (_, day, month, _) = date(days);
    let index = MONTHS.iter().position(|(m, _, _)| *m == month).unwrap_or(0);
    let (month, year) = if index == 3 { (1, 2027) } else { (index + 10, 2026) };
    format!("{day:02}/{month:02}/{year}")
}

fn short_date(days: u32) -> String {
    let (_, day, _, short) = date(days);
    format!("{day} {short}")
}

fn new_card(random: &mut Random, number: u32, today: u32) -> LoanCard {
    let kind = random.pick(&CARD_KINDS);
    let state = match random.below(16) {
        0 => CardState::Broken,
        1 => CardState::Lost,
        2 | 3 => CardState::ToCheck,
        _ => CardState::InService,
    };
    let loans = random.below(19);
    let mut history = Vec::new();
    let mut last_borrower = None;
    let mut days = today;
    for i in 0..loans.min(3) {
        days = days.saturating_sub(random.between(3, 9));
        let who = format!("{} {}", random.pick(&FAMILY_NAMES), random.pick(&FIRST_NAMES));
        let text = format!("Prêtée à {who} · {} · rendue", random.pick(&EVENT_NAMES));
        history.push(CardEntry { date: short_date(days), text });
        if i == 0 {
            last_borrower = Some(who);
        }
    }
    let troubled = state != CardState::InService;
    let battery = if kind == "SI-Card 8" || kind == "SI-Card 9" { None } else { Some(random.pick(&CARD_BATTERIES)) };
    // The notes that speak of a battery stay on the cards that have one.
    let notes = if battery.is_some() { &CARD_NOTES[..] } else { &CARD_NOTES[..2] };
    LoanCard {
        number,
        kind,
        state,
        state_since: short_date(today.saturating_sub(random.below(30))),
        place: CardPlace::Park,
        battery,
        last_borrower,
        loans,
        note: if troubled || random.below(8) == 0 { random.pick(notes) } else { "" },
        history,
    }
}

// A card that went out with a runner: it can be lent, so it is not broken or lost.
fn lent_card(random: &mut Random, number: u32, today: u32, place: CardPlace, who: String, entry: CardEntry) -> LoanCard {
    let mut card = new_card(random, number, today);
    if matches!(card.state, CardState::Broken | CardState::Lost) {
        card.state = CardState::InService;
        card.note = "";
    }
    card.place = place;
    card.last_borrower = Some(who);
    card.loans += 1;
    card.history.insert(0, entry);
    card
}

// The stock comes after the events: it holds the cards lent to their runners, so all
// the screens agree. Only the current event and a past one or two still have cards out.
fn new_card_parks(seed: u64, events: &mut [Event], past: &[usize], today: u32) -> Vec<CardPark> {
    let mut random = Random(seed ^ STOCK_SEED);
    let mut club: Vec<LoanCard> = Vec::new();
    let mut overdue = random.between(1, 3);

    for (index, event) in events.iter_mut().enumerate() {
        let name = event.name.clone();
        let day = format!("{} {}", event.day, event.month_short);
        let is_past = past.contains(&index);
        let is_current = index == CURRENT_EVENT;
        for runner in event.runners.iter_mut().filter(|r| r.lent) {
            let Some(number) = runner.card else { continue };
            if club.iter().any(|c| c.number == number) {
                runner.lent = false;
                continue;
            }
            let who = runner.name();
            if is_current {
                let place = CardPlace::Lent { who: who.clone(), event: name.clone(), date: day.clone() };
                let entry = CardEntry { date: day.clone(), text: format!("Prêtée à {who} · {name}") };
                club.push(lent_card(&mut random, number, today, place, who, entry));
            } else if is_past && overdue > 0 {
                overdue -= 1;
                let place = CardPlace::NotReturned { who: who.clone(), event: name.clone(), since: day.clone() };
                let entry = CardEntry { date: day.clone(), text: format!("Prêtée à {who} · {name} · non rendue") };
                club.push(lent_card(&mut random, number, today, place, who, entry));
            } else {
                runner.lent = false;
            }
        }
    }
    for _ in 0..random.between(22, 32) {
        let number = random.between(2_040_000, 2_049_999);
        if club.iter().all(|c| c.number != number) {
            club.push(new_card(&mut random, number, today));
        }
    }
    club.sort_by_key(|c| c.number);

    let mut league: Vec<LoanCard> = (0..random.between(8, 14))
        .map(|i| new_card(&mut random, 2_051_000 + i * 7, today))
        .collect();
    league.sort_by_key(|c| c.number);

    // Cards put on a reader that belong to no park: nothing is known about them yet.
    let unknown = (0..2)
        .map(|i| LoanCard {
            number: random.between(7_000_000, 7_999_999) + i,
            kind: random.pick(&CARD_KINDS),
            state: CardState::ToCheck,
            state_since: short_date(today),
            place: CardPlace::Park,
            battery: None,
            last_borrower: None,
            loans: 0,
            note: "",
            history: vec![CardEntry { date: short_date(today), text: "Posée sur un lecteur, hors parc".into() }],
        })
        .collect();

    vec![
        CardPark { name: "Parc du club", description: "Mallette rouge, rangée au local du club.", cards: club },
        CardPark { name: "Ligue AURA (prêt)", description: "Puces prêtées au club par la ligue.", cards: league },
        CardPark { name: "Non identifiées", description: "Puces posées sur un lecteur, sans parc connu.", cards: unknown },
    ]
}

const BASE_SEED: u64 = 0xBA5E;
const BASE_FAMILY_NAMES: [&str; 30] = [
    "ARNAUD", "BLANC", "BONNET", "BRUN", "CARON", "CHEVALIER", "CLÉMENT", "COLIN", "DAVID", "DUBOIS", "FABRE", "FONTAINE",
    "GARNIER", "GAUTIER", "GUÉRIN", "HENRY", "JOLY", "LAMBERT", "LEFEBVRE", "MARCHAND", "MEUNIER", "MICHEL", "PERRIN", "PICARD",
    "RENARD", "RICHARD", "RIVIÈRE", "ROUSSEL", "SIMON", "VIDAL",
];
const NO_CLUB: &str = "Individuel (sans club)";
// The clubs of the base with their code: its last two letters are the league.
const BASE_CLUBS: [(&str, &str); 11] = [
    ("ASO Lyon", "6905AR"),
    ("SMOC", "6912AR"),
    ("TAD", "3804AR"),
    ("COLE", "0102AR"),
    ("CO Brou", "0106AR"),
    ("OTB", "4207AR"),
    ("CO des Vallons", "2903BR"),
    ("Balise 44", "4401PL"),
    ("Raid du Nord", "5908HF"),
    ("Orient'Sud", "3402OC"),
    (NO_CLUB, "0000AR"),
];

fn club_code(club: &str) -> &'static str {
    BASE_CLUBS.iter().find(|(name, _)| *name == club).map_or("", |(_, code)| *code)
}

// A category says the sex and the age, when it is a known one.
fn set_base_category(random: &mut Random, person: &mut BaseRunner, category: &str) {
    person.category = category.to_string();
    if category.starts_with('D') {
        person.sex = "F";
    } else if category.starts_with('H') {
        person.sex = "H";
    }
    let age: String = category.chars().filter(char::is_ascii_digit).collect();
    if let Ok(age) = age.parse::<u32>() {
        person.year = BATTERY_YEAR - age - random.below(3);
    }
}

fn new_base_runner(random: &mut Random, id: u32, family: &'static str, first: &'static str, club: &'static str, today: u32) -> BaseRunner {
    let kind = match random.below(12) {
        0 | 1 => LicenceKind::Discovery,
        2 => LicenceKind::Leisure,
        3 if random.below(4) == 0 => LicenceKind::Handisport,
        _ => LicenceKind::Competition,
    };
    let scored = random.below(5) < 2;
    let category = random.pick(&ALL_CATEGORIES);
    let mut person = BaseRunner {
        id,
        family,
        first,
        club,
        club_code: club_code(club),
        sex: "H",
        year: random.between(1950, 2012),
        category: String::new(),
        licence: Some(random.between(100_000, 999_999)),
        licence_kind: Some(kind),
        renewed: random.below(30) > 0,
        card: (random.below(5) > 0).then(|| random.between(8_100_000, 8_199_999)),
        points: scored.then(|| (random.between(800, 1500), short_date(today.saturating_sub(random.between(5, 25))))),
        entered_on: String::new(),
    };
    set_base_category(random, &mut person, category);
    person
}

// The base holds the runners of today's event and those who have a card of the club, so
// that the screens agree. Some cards typed by hand, some shared cards: things to check.
fn new_runner_base(seed: u64, events: &[Event], today: u32) -> Vec<BaseRunner> {
    let mut random = Random(seed ^ BASE_SEED);
    let mut base: Vec<BaseRunner> = Vec::new();
    let known = |base: &[BaseRunner], family: &str, first: &str| base.iter().any(|b| b.family == family && b.first == first);

    for (index, event) in events.iter().enumerate() {
        for runner in event.runners.iter().filter(|r| index == CURRENT_EVENT || r.lent) {
            if known(&base, runner.family, runner.first) {
                continue;
            }
            let id = base.len() as u32;
            let mut person = new_base_runner(&mut random, id, runner.family, runner.first, runner.club, today);
            set_base_category(&mut random, &mut person, &runner.category);
            person.card = runner.card;
            base.push(person);
        }
    }
    let wanted = random.between(280, 340) as usize;
    while base.len() < wanted {
        let family = if random.below(4) == 0 { random.pick(&FAMILY_NAMES) } else { random.pick(&BASE_FAMILY_NAMES) };
        let first = random.pick(&FIRST_NAMES);
        if !known(&base, family, first) {
            let club = random.pick(&BASE_CLUBS).0;
            let id = base.len() as u32;
            base.push(new_base_runner(&mut random, id, family, first, club, today));
        }
    }
    // Two licensed runners sharing a card, twice.
    for _ in 0..2 {
        let (from, to) = (random.below(base.len() as u32) as usize, random.below(base.len() as u32) as usize);
        if from != to && base[from].card.is_some() {
            base[to].card = base[from].card;
        }
    }
    // Cards typed by hand: some are probably someone who has a licence, some are not.
    for i in 0..6 {
        let id = base.len() as u32;
        let (family, first) = if i < 3 {
            let original = random.below(id) as usize;
            (base[original].family, base[original].first)
        } else {
            let mut name = (random.pick(&BASE_FAMILY_NAMES), random.pick(&FIRST_NAMES));
            while known(&base, name.0, name.1) {
                name = (random.pick(&BASE_FAMILY_NAMES), random.pick(&FIRST_NAMES));
            }
            name
        };
        let mut person = new_base_runner(&mut random, id, family, first, NO_CLUB, today);
        person.licence = None;
        person.licence_kind = None;
        person.points = None;
        person.card = None;
        person.entered_on = short_date(today.saturating_sub(random.below(30)));
        base.push(person);
    }
    base
}

const STATION_KINDS: [&str; 3] = ["BSF7", "BSF8", "BSM8"];
const STATION_NOTES: [&str; 3] = ["Étiquette à refaire", "Bip faible", "Support fendu"];
const STATION_STOCK_SEED: u64 = 0xB01E;
pub const UNIDENTIFIED_PARK: &str = "Non identifiés";
const BATTERY_YEAR: u32 = 2026;
const BATTERY_MAX_YEARS: u32 = 5;
const LOW_CHARGE: u32 = 320;
pub const CLOCK_MAX_GAP: u32 = 30;

fn new_stock_station(random: &mut Random, serial: u32, code: String, today: u32) -> StockStation {
    let state = match random.below(24) {
        0 => StockState::Lost,
        1 => StockState::InRepair,
        2 => StockState::Destroyed,
        3 | 4 => StockState::ToCheck,
        _ => StockState::InService,
    };
    let measured = random.below(10) > 0;
    let days = today.saturating_sub(random.below(30));
    let mut station = StockStation {
        serial,
        code,
        kind: random.pick(&STATION_KINDS),
        state,
        state_since: short_date(today.saturating_sub(random.below(30))),
        battery_year: if random.below(12) == 0 { 2019 } else { random.between(2021, BATTERY_YEAR) },
        charge: measured.then(|| if random.below(10) == 0 { random.between(300, LOW_CHARGE - 1) } else { random.between(LOW_CHARGE, 335) }),
        clock_gap: measured.then(|| if random.below(12) == 0 { random.between(CLOCK_MAX_GAP + 1, 60) } else { random.below(4) }),
        measured: measured.then(|| short_date(days)),
        note: if state != StockState::InService || random.below(8) == 0 { random.pick(&STATION_NOTES) } else { "" },
        history: vec![CardEntry { date: short_date(days), text: format!("Utilisé · {}", random.pick(&EVENT_NAMES)) }],
    };
    station.settle();
    station
}

// The stations of the current event are in the stock, in the park the event says.
fn new_station_parks(seed: u64, event: &Event, today: u32) -> Vec<StationPark> {
    let mut random = Random(seed ^ STATION_STOCK_SEED);
    let (mut club, mut spare): (Vec<StockStation>, Vec<StockStation>) = (Vec::new(), Vec::new());
    for station in &event.stations {
        let Some(serial) = station.serial else { continue };
        if club.iter().chain(spare.iter()).any(|s| s.serial == serial) {
            continue;
        }
        let mut stock = new_stock_station(&mut random, serial, station.code.clone(), today);
        // It went out for this event: it can be neither lost nor out of order.
        stock.state = StockState::InService;
        stock.note = "";
        if station.low_battery {
            stock.charge = Some(random.between(300, LOW_CHARGE - 1));
        } else if stock.low_charge() {
            stock.charge = Some(LOW_CHARGE);
        }
        if let StationState::ClockGap(gap) = station.state {
            stock.clock_gap = Some(gap);
            stock.measured = Some(short_date(today));
        }
        stock.settle();
        if station.park == "Secours" { spare.push(stock) } else { club.push(stock) }
    }
    for _ in 0..random.between(6, 12) {
        let serial = random.between(210_000, 219_999);
        let code = random.between(31, 99).to_string();
        if club.iter().all(|s| s.serial != serial) {
            club.push(new_stock_station(&mut random, serial, code, today));
        }
    }
    for _ in 0..random.between(4, 8) {
        let serial = random.between(220_000, 229_999);
        let code = random.between(31, 99).to_string();
        if spare.iter().all(|s| s.serial != serial) {
            spare.push(new_stock_station(&mut random, serial, code, today));
        }
    }
    club.sort_by_key(|s| s.serial);
    spare.sort_by_key(|s| s.serial);

    // A station put on a reader that belongs to no park stays in service: only someone
    // can say it has to be checked.
    let unknown = vec![StockStation {
        serial: random.between(300_000, 309_999),
        code: random.between(31, 99).to_string(),
        kind: random.pick(&STATION_KINDS),
        state: StockState::InService,
        state_since: short_date(today),
        battery_year: 2024,
        charge: Some(325),
        clock_gap: Some(1),
        measured: Some(short_date(today)),
        note: "",
        history: vec![CardEntry { date: short_date(today), text: "Posé sur un lecteur, hors parc".into() }],
    }];

    vec![
        StationPark { name: "Club", description: "Caisses du local, pour toutes les courses du club.", stations: club },
        StationPark { name: "Secours", description: "Boîtiers de remplacement, à emporter en plus.", stations: spare },
        StationPark { name: UNIDENTIFIED_PARK, description: "Boîtiers posés sur un lecteur, sans parc connu.", stations: unknown },
    ]
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

        let card_parks = new_card_parks(seed, &mut events, &past, today);
        let station_parks = new_station_parks(seed, &events[CURRENT_EVENT], today);
        let archive_date = short_date(today.saturating_sub(4));
        let archive_date_numeric = numeric_date(today.saturating_sub(4));
        let runner_base = new_runner_base(seed, &events, today);
        let clock_seconds = events[CURRENT_EVENT].readings.last().map(|r| r.at + 30);
        let clock_seconds = clock_seconds.unwrap_or_else(|| random.between(10, 14) * 3600);
        // Invented serial numbers, close to the event's own station.
        let serial = events[CURRENT_EVENT].station;
        let reader = |usage: Option<usize>, serial: u32, pinned: bool| Reader {
            name: String::new(),
            serial,
            pinned,
            unplugged: false,
            usage,
            course: 0,
        };
        let mut readers = vec![
            reader(Some(0), serial, true),
            reader(Some(0), serial + 6, true),
            reader(Some(1), serial + 11, false),
            reader(None, serial + 2, false),
        ];
        name_readers(&mut readers);
        // Two devices linked to the course, one offline; sometimes a tablet asks to join.
        let devices = vec![
            LinkedDevice { kind: DeviceKind::Laptop, name: format!("Ordinateur de {}", random.pick(&FIRST_NAMES)), role: 1, waiting: None },
            LinkedDevice {
                kind: DeviceKind::Phone,
                name: format!("Téléphone de {}", random.pick(&FIRST_NAMES)),
                role: 3,
                waiting: Some(random.between(2, 8)),
            },
        ];
        // Two screens plugged in (sometimes only one showing), and a free window, closed.
        let screens = vec![
            PublicScreen::new("Télé HDMI".into(), Some("1920 × 1080"), true, 0),
            PublicScreen::new("Écran USB-C".into(), Some("1280 × 720"), random.below(2) == 1, 1),
            PublicScreen::new("Fenêtre 1".into(), None, false, 2),
        ];
        let request = (random.below(2) == 1).then(|| DeviceRequest {
            kind: DeviceKind::Tablet,
            name: format!("Tablette de {}", random.pick(&FIRST_NAMES)),
            role: 4,
            word: random.pick(&CHECK_WORDS),
        });
        Generated {
            events,
            upcoming,
            past,
            readers,
            runner_base,
            archive_date,
            archive_date_numeric,
            card_parks,
            station_parks,
            devices,
            request,
            screens,
            today,
            clock_seconds,
            next_reading: clock_seconds + 3,
            random,
        }
    }

    fn stock_stations(&self) -> impl Iterator<Item = &StockStation> {
        self.station_parks.iter().flat_map(|p| p.stations.iter())
    }

    fn stock_station_mut(&mut self, serial: u32) -> Option<&mut StockStation> {
        self.station_parks.iter_mut().flat_map(|p| p.stations.iter_mut()).find(|s| s.serial == serial)
    }

    pub fn stock_station(&self, serial: u32) -> Option<&StockStation> {
        self.stock_stations().find(|s| s.serial == serial)
    }

    // What the club counts as its own: its parks, without the stations nobody identified.
    pub fn club_parks(&self) -> impl Iterator<Item = &StationPark> {
        self.station_parks.iter().filter(|p| p.name != UNIDENTIFIED_PARK)
    }

    pub fn batteries_due(&self) -> Vec<&StockStation> {
        self.stock_stations().filter(|s| s.state == StockState::BatteryDue).collect()
    }

    pub fn set_stock_state(&mut self, serial: u32, state: StockState, text: &str) {
        let date = short_date(self.today);
        if let Some(station) = self.stock_station_mut(serial) {
            station.state = state;
            station.settle();
            station.state_since = date.clone();
            station.history.insert(0, CardEntry { date, text: text.into() });
        }
    }

    // The new battery is measured at the next reading.
    pub fn change_battery(&mut self, serial: u32) {
        let date = short_date(self.today);
        if let Some(station) = self.stock_station_mut(serial) {
            station.battery_year = BATTERY_YEAR;
            station.charge = None;
            if station.state == StockState::BatteryDue {
                station.state = StockState::InService;
                station.state_since = date.clone();
            }
            station.history.insert(0, CardEntry { date, text: "Pile changée".into() });
        }
    }

    // A reading measures the station, and never changes a state chosen by someone.
    pub fn toggle_pin(&mut self, reader: usize) {
        if let Some(reader) = self.readers.get_mut(reader) {
            reader.pinned = !reader.pinned;
        }
    }

    // A reader takes another use: the readers are named again (Arrivée 1, Arrivée 2…).
    pub fn set_reader_usage(&mut self, reader: usize, usage: usize) {
        if let Some(reader) = self.readers.get_mut(reader) {
            reader.usage = (usage < READER_USAGES.len()).then_some(usage);
        }
        name_readers(&mut self.readers);
    }

    pub fn set_reader_course(&mut self, reader: usize, course: usize) {
        if let Some(reader) = self.readers.get_mut(reader) {
            reader.course = course;
        }
    }

    // The courses a reader may read for: today's, then the next one.
    pub fn reader_courses(&self) -> Vec<String> {
        let mut courses = vec![self.events[CURRENT_EVENT].name.clone()];
        courses.extend(self.upcoming.first().map(|&e| self.events[e].name.clone()));
        courses
    }

    // The device asking to join: accepted, it is linked with the role it asked for.
    pub fn accept_request(&mut self) {
        if let Some(request) = self.request.take() {
            self.devices.push(LinkedDevice { kind: request.kind, name: request.name, role: request.role, waiting: None });
        }
    }

    pub fn refuse_request(&mut self) {
        self.request = None;
    }

    pub fn set_device_role(&mut self, device: usize, role: usize) {
        if let Some(device) = self.devices.get_mut(device) {
            device.role = role.min(DEVICE_ROLES.len() - 1);
        }
    }

    pub fn remove_device(&mut self, device: usize) {
        if device < self.devices.len() {
            self.devices.remove(device);
        }
    }

    pub fn toggle_screen(&mut self, screen: usize) {
        if let Some(screen) = self.screens.get_mut(screen) {
            screen.on = !screen.on;
        }
    }

    // Choosing a content shows it at once (prototype: « Choisir un contenu le diffuse aussitôt »).
    pub fn set_screen_content(&mut self, screen: usize, content: usize) {
        if let Some(screen) = self.screens.get_mut(screen) {
            screen.content = content.min(SCREEN_CONTENTS.len() - 1);
            screen.on = true;
        }
    }

    // « + Fenêtre »: a free window, showing the reading.
    pub fn add_window(&mut self) -> usize {
        let number = self.screens.iter().filter(|s| s.size.is_none()).count() + 1;
        self.screens.push(PublicScreen::new(format!("Fenêtre {number}"), None, true, 0));
        self.screens.len() - 1
    }

    pub fn remove_window(&mut self, screen: usize) {
        if self.screens.get(screen).is_some_and(|s| s.size.is_none()) {
            self.screens.remove(screen);
        }
    }

    // The reader shown on one side of a reading screen: the next one, then none.
    pub fn next_screen_reader(&mut self, screen: usize, side: usize) {
        let readers = self.readers.len();
        if let Some(slot) = self.screens.get_mut(screen).and_then(|s| s.readers.get_mut(side)) {
            *slot = match *slot {
                None => Some(0),
                Some(r) if r + 1 < readers => Some(r + 1),
                Some(_) => None,
            };
        }
    }

    // A switch of a screen: 0 provisional ranking, 1 starters' names, 2 beeps.
    pub fn toggle_screen_setting(&mut self, screen: usize, setting: usize) {
        if let Some(screen) = self.screens.get_mut(screen) {
            match setting {
                0 => screen.provisional = !screen.provisional,
                1 => screen.names = !screen.names,
                _ => screen.beeps = !screen.beeps,
            }
        }
    }

    pub fn toggle_unplugged(&mut self, reader: usize) {
        if let Some(reader) = self.readers.get_mut(reader) {
            reader.unplugged = !reader.unplugged;
        }
    }

    // A finish reader is reading the card just put on it, for a short while.
    pub fn reader_status(&self, reader: usize) -> ReaderStatus {
        if self.readers.get(reader).is_some_and(|r| r.unplugged) {
            return ReaderStatus::Unplugged;
        }
        let last = self.events[CURRENT_EVENT].readings.last();
        if last.is_some_and(|r| r.reader == reader && self.clock_seconds.saturating_sub(r.at) < READING_SECONDS) {
            ReaderStatus::Reading
        } else {
            ReaderStatus::Ready
        }
    }

    // A card of the stock put on that reader: nothing changes, its detail opens.
    pub fn read_loan_card(&mut self) -> Option<u32> {
        let numbers: Vec<u32> = self.cards().map(|c| c.number).collect();
        if numbers.is_empty() {
            return None;
        }
        Some(self.random.pick(&numbers))
    }

    // A card that is in none of the club's parks: a runner's own card put on the reader.
    pub fn read_unknown_card(&mut self) -> u32 {
        loop {
            let number = self.random.between(2_000_000, 8_999_999);
            if self.card(number).is_none() {
                return number;
            }
        }
    }

    pub fn read_stock_station(&mut self) -> Option<u32> {
        let serials: Vec<u32> = self.stock_stations().filter(|s| s.state != StockState::Destroyed).map(|s| s.serial).collect();
        if serials.is_empty() {
            return None;
        }
        let serial = self.random.pick(&serials);
        let charge = if self.random.below(8) == 0 { self.random.between(300, LOW_CHARGE - 1) } else { self.random.between(LOW_CHARGE, 335) };
        let gap = if self.random.below(8) == 0 { self.random.between(CLOCK_MAX_GAP + 1, 60) } else { self.random.below(4) };
        let date = short_date(self.today);
        let station = self.stock_station_mut(serial)?;
        station.charge = Some(charge);
        station.clock_gap = Some(gap);
        station.measured = Some(date.clone());
        // A lost station read again is back: this is the one change a reading makes alone.
        let text = if station.state == StockState::Lost {
            station.state = StockState::InService;
            station.state_since = date.clone();
            "Retrouvé sur la station"
        } else {
            "Lu sur la station"
        };
        station.settle();
        station.history.insert(0, CardEntry { date, text: text.into() });
        Some(serial)
    }

    fn cards(&self) -> impl Iterator<Item = &LoanCard> {
        self.card_parks.iter().flat_map(|p| p.cards.iter())
    }

    pub fn card(&self, number: u32) -> Option<&LoanCard> {
        self.cards().find(|c| c.number == number)
    }

    // The cards that can be lent right now.
    pub fn cards_free(&self) -> u32 {
        self.cards()
            .filter(|c| c.place == CardPlace::Park && matches!(c.state, CardState::InService | CardState::ToCheck))
            .count() as u32
    }

    pub fn cards_lent(&self) -> u32 {
        self.cards().filter(|c| matches!(c.place, CardPlace::Lent { .. })).count() as u32
    }

    pub fn overdue_cards(&self) -> Vec<&LoanCard> {
        self.cards().filter(|c| matches!(c.place, CardPlace::NotReturned { .. }) && c.state != CardState::Lost).collect()
    }

    pub fn set_card_state(&mut self, number: u32, state: CardState, text: &str) {
        let date = short_date(self.today);
        if let Some(card) = self.card_parks.iter_mut().flat_map(|p| p.cards.iter_mut()).find(|c| c.number == number) {
            card.state = state;
            card.state_since = date.clone();
            card.history.insert(0, CardEntry { date, text: text.into() });
        }
    }

    pub fn return_card(&mut self, number: u32) {
        let date = short_date(self.today);
        if let Some(card) = self.card_parks.iter_mut().flat_map(|p| p.cards.iter_mut()).find(|c| c.number == number) {
            card.place = CardPlace::Park;
            card.history.insert(0, CardEntry { date, text: "Rendue".into() });
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

    pub fn clock(&self) -> u32 {
        self.clock_seconds
    }

    // One second of the race goes by; now and then, a card is read.
    pub fn tick(&mut self) {
        self.clock_seconds += 1;
        if self.clock_seconds >= self.next_reading {
            self.read_card_now();
            self.next_reading = self.clock_seconds + self.random.between(4, 20);
        }
    }

    // Time goes by, then a card is read. Only the captures need it.
    #[cfg(not(any(target_os = "ios", target_arch = "wasm32")))]
    pub fn read_a_card(&mut self) {
        self.clock_seconds += self.random.between(5, 40);
        self.read_card_now();
    }

    // Lets time go by without any reading. Only the captures need it.
    #[cfg(not(any(target_os = "ios", target_arch = "wasm32")))]
    pub fn wait(&mut self, seconds: u32) {
        self.clock_seconds += seconds;
        self.next_reading = self.next_reading.max(self.clock_seconds + 1);
    }

    // A runner out on a course comes back and reads their card. Now and then, a card nobody
    // knows or one read a second time. An unplugged reader reads nothing.
    fn read_card_now(&mut self) {
        let random = &mut self.random;
        let at = self.clock_seconds;
        let event = &mut self.events[CURRENT_EVENT];
        let racing: Vec<usize> = (0..event.runners.len()).filter(|i| event.runners[*i].is_racing()).collect();
        if racing.is_empty() {
            return;
        }
        let plugged: Vec<usize> = (0..FINISH_READERS).filter(|i| !self.readers[*i].unplugged).collect();
        if plugged.is_empty() {
            return;
        }
        let reader = random.pick(&plugged);
        let draw = random.below(12);
        let reading = if draw == 0 {
            Reading {
                runner: None,
                course: event.runners[racing[0]].runs[0].course,
                card: random.between(2_000_000, 9_999_999),
                seconds: random.between(1500, 4200),
                valid: random.below(8) > 0,
                at,
                reader,
                issue: Some(Issue::UnknownCard),
                settled_at: None,
            }
        } else if draw == 1 && event.readings.iter().any(|r| r.issue.is_none()) {
            let known: Vec<usize> = (0..event.readings.len()).filter(|i| event.readings[*i].issue.is_none()).collect();
            let first = &event.readings[random.pick(&known)];
            Reading { runner: first.runner, course: first.course, card: first.card, seconds: first.seconds, valid: first.valid, at, reader, issue: Some(Issue::AlreadyRead), settled_at: None }
        } else {
            let index = random.pick(&racing);
            let runner = &mut event.runners[index];
            // Their time follows from the clock when it makes sense: the generator's start times can be late.
            let since_start = at.saturating_sub(runner.start);
            let seconds = if (900..=9000).contains(&since_start) { since_start } else { random.between(1500, 4200) };
            let valid = random.below(8) > 0;
            let run = runner.runs.iter_mut().find(|r| r.result.is_none()).expect("a racing runner has a run left");
            run.result = Some((seconds, valid));
            Reading { runner: Some(index), course: run.course, card: runner.card.unwrap_or_default(), seconds, valid, at, reader, issue: None, settled_at: None }
        };
        event.readings.push(reading);
    }
}

// A reader is named by its use, numbered when several share it; without one, « Lecteur libre ».
fn name_readers(readers: &mut [Reader]) {
    let usages: Vec<Option<usize>> = readers.iter().map(|r| r.usage).collect();
    for (i, reader) in readers.iter_mut().enumerate() {
        reader.name = match reader.usage {
            None => "Lecteur libre".to_string(),
            Some(usage) => {
                let same: Vec<usize> = (0..usages.len()).filter(|&j| usages[j] == Some(usage)).collect();
                let word = READER_USAGES[usage];
                if same.len() > 1 {
                    let rank = same.iter().position(|&j| j == i).map_or(1, |p| p + 1);
                    format!("{word} {rank}")
                } else {
                    word.to_string()
                }
            }
        };
    }
}
