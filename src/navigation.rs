// The navigation state: two levels, the software and one event.

use crate::{App, EvenementAffiche, Navigation, donnees, modele};
use slint::ComponentHandle;

pub const RUBRIQUES: [&str; 6] = ["Accueil", "Événements", "Boîtiers", "Puces", "Base coureurs", "Réglages"];
pub const ONGLETS: [&str; 6] = ["Circuits", "Participants", "Départs", "Boîtiers", "Direct", "Résultats"];

const RUBRIQUE_EVENEMENTS: usize = 1;

#[derive(Default)]
pub struct Navigateur {
    rubrique: usize,
    onglet: usize,
    evenement: Option<usize>,
}

impl Navigateur {
    pub fn choisir_rubrique(&mut self, rubrique: usize) {
        self.rubrique = rubrique;
    }

    pub fn choisir_onglet(&mut self, onglet: usize) {
        self.onglet = onglet;
    }

    pub fn ouvrir_evenement(&mut self, evenement: usize) {
        self.evenement = Some(evenement);
        self.onglet = 0;
    }

    pub fn revenir(&mut self) {
        self.evenement = None;
        self.rubrique = RUBRIQUE_EVENEMENTS;
    }

    pub fn afficher(&self, fenetre: &App) {
        let navigation = fenetre.global::<Navigation>();
        navigation.set_rubrique_active(self.rubrique as i32);
        navigation.set_onglet_actif(self.onglet as i32);
        navigation.set_dans_evenement(self.evenement.is_some());
        navigation.set_retour(format!("‹ {}", RUBRIQUES[RUBRIQUE_EVENEMENTS]).into());
        navigation.set_nom_evenement(
            self.evenement.map_or("", |e| donnees::EVENEMENTS[e].nom).into(),
        );
        navigation.set_titre(
            match self.evenement {
                Some(_) => ONGLETS[self.onglet],
                None => RUBRIQUES[self.rubrique],
            }
            .into(),
        );

        // The event list only shows on the Événements page.
        let liste = self.evenement.is_none() && self.rubrique == RUBRIQUE_EVENEMENTS;
        navigation.set_evenements(modele(if liste {
            donnees::EVENEMENTS
                .iter()
                .map(|e| EvenementAffiche { nom: e.nom.into(), detail: e.detail.into() })
                .collect()
        } else {
            Vec::new()
        }));
    }
}
