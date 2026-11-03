// The test data generator, started from the Démo panel. It stands in for what connected
// devices and real events will provide. When it is off, the app has no data at all.
// All names, clubs and card numbers are invented.

#[derive(Clone, Copy, Default, PartialEq)]
pub enum Kind {
    #[default]
    None,
    Valid,
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
    fn new(word: impl Into<String>, kind: Kind) -> Self {
        Label { word: word.into(), kind }
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
    pub courses: u32,
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
    pub racing: u32,
    pub finished: u32,
    pub to_handle: u32,
    pub readers: Vec<String>,
    pub readings: Vec<Reading>,
    pub club: ClubFigures,
    pub devices_asking: u32,
    pub screens: u32,
    clock_seconds: u32,
    random: Random,
}

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

fn format_time(seconds: u32) -> String {
    format!("{}:{:02}", seconds / 60, seconds % 60)
}

fn format_clock(seconds: u32) -> String {
    format!("{}:{:02}", seconds / 3600, seconds / 60 % 60)
}

fn new_event(random: &mut Random, name: &str, days: u32, past: bool) -> Event {
    let (weekday, day, month, month_short) = date(days);
    let courses = random.below(7);
    let entered = random.between(30, 320);
    let label = if past {
        Label::new("Résultats publiés", Kind::Inactive)
    } else if courses == 0 {
        Label::new("À préparer", Kind::ToCheck)
    } else {
        Label::new(format!("{courses} circuits prêts"), Kind::Valid)
    };
    Event {
        name: name.into(),
        weekday,
        day,
        month,
        month_short,
        place: random.pick(&PLACES),
        participants: format!("{entered} {}", if past { "participants" } else { "inscrits" }),
        courses,
        label,
    }
}

impl Generated {
    pub fn new(seed: u64) -> Self {
        let mut random = Random(seed);
        let first = random.below(EVENT_NAMES.len() as u32) as usize;
        let name = |i: usize| EVENT_NAMES[(first + i) % EVENT_NAMES.len()];
        let today = 40 + random.below(30);

        let mut events = vec![new_event(&mut random, name(0), today, false)];
        events[CURRENT_EVENT].courses = random.between(3, 6);
        let (mut upcoming, mut past) = (Vec::new(), Vec::new());
        let mut days = today;
        for i in 1..=3 {
            days += random.between(3, 12);
            upcoming.push(events.len());
            events.push(new_event(&mut random, name(i), days, false));
        }
        let mut days = today;
        for i in 4..=5 {
            days -= random.between(4, 16);
            past.push(events.len());
            events.push(new_event(&mut random, name(i), days, true));
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
            racing: random.between(40, 120),
            finished: random.between(60, 200),
            to_handle: 0,
            readers: vec!["Arrivée 1".into(), "Arrivée 2".into()],
            readings: Vec::new(),
            club,
            devices_asking: random.below(2),
            screens: random.below(3),
            clock_seconds: random.between(10, 14) * 3600,
            random,
        }
    }

    // A runner comes back and reads their card.
    pub fn read_a_card(&mut self) {
        let random = &mut self.random;
        let seconds = random.between(1500, 4200);
        self.clock_seconds += random.between(5, 40);
        let (label, name) = match random.below(12) {
            0 => {
                self.to_handle += 1;
                (Label::new("Inconnue", Kind::Unknown), format!("Puce {}", random.between(2_000_000, 9_999_999)))
            }
            1 => (Label::new("PM", Kind::Pm), random_runner(random)),
            _ => (Label::new("OK", Kind::Ok), random_runner(random)),
        };
        self.racing = self.racing.saturating_sub(1);
        self.finished += 1;
        self.readings.insert(
            0,
            Reading { label, name, time: format_time(seconds), clock: format_clock(self.clock_seconds) },
        );
        self.readings.truncate(READINGS_SHOWN);
    }
}

const READINGS_SHOWN: usize = 3;

fn random_runner(random: &mut Random) -> String {
    format!("{} {}", random.pick(&FAMILY_NAMES), random.pick(&FIRST_NAMES))
}
