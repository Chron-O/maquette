// Made-up data only: names, clubs and card numbers are invented.

pub struct Event {
    pub name: &'static str,
    pub detail: &'static str,
}

pub const EVENTS: [Event; 1] =
    [Event { name: "Régionale de Lyon", detail: "Dimanche 16 novembre · Forêt de Brou" }];
