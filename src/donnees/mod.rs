// Made-up data only: names, clubs and card numbers are invented.

pub struct Evenement {
    pub nom: &'static str,
    pub detail: &'static str,
}

pub const EVENEMENTS: [Evenement; 1] =
    [Evenement { nom: "Régionale de Lyon", detail: "Dimanche 16 novembre · Forêt de Brou" }];
