// Capture mode, used by scripts/captures.sh:
//   maquette --capture FILE clair|sombre WIDTH HEIGHT tactile|souris [rubrique|onglet] [INDEX]
// Without these arguments, the app starts normally.

use crate::appareil::{self, Appareil};
use crate::navigation::Navigateur;
use crate::{App, ModeTheme, Theme};
use slint::ComponentHandle;
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::Duration;

const ATTENTE: Duration = Duration::from_millis(1200);

pub fn preparer(fenetre: &App, navigateur: &Rc<RefCell<Navigateur>>, choix: &Rc<Cell<Appareil>>) {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 7 || args[1] != "--capture" {
        return;
    }
    let mode = if args[3] == "sombre" { ModeTheme::Sombre } else { ModeTheme::Clair };
    fenetre.global::<Theme>().set_mode(mode);

    let largeur: f32 = args[4].parse().expect("width");
    let hauteur: f32 = args[5].parse().expect("height");
    fenetre.window().set_size(slint::LogicalSize::new(largeur, hauteur));

    choix.set(if args[6] == "tactile" { Appareil::Tactile } else { Appareil::Souris });
    appareil::appliquer(fenetre, choix.get());

    let index: usize = args.get(8).map_or(0, |i| i.parse().expect("index"));
    match args.get(7).map(String::as_str) {
        Some("onglet") => {
            let mut navigateur = navigateur.borrow_mut();
            navigateur.ouvrir_evenement(0);
            navigateur.choisir_onglet(index);
        }
        Some("rubrique") => navigateur.borrow_mut().choisir_rubrique(index),
        _ => {}
    }
    navigateur.borrow().afficher(fenetre);

    // Waits for the window to be drawn, saves the image, then closes.
    let chemin = args[2].clone();
    let faible = fenetre.as_weak();
    slint::Timer::single_shot(ATTENTE, move || {
        let fenetre = faible.unwrap();
        let image = fenetre.window().take_snapshot().expect("snapshot failed");
        image::save_buffer(
            &chemin,
            image.as_bytes(),
            image.width(),
            image.height(),
            image::ColorType::Rgba8,
        )
        .expect("saving failed");
        slint::quit_event_loop().unwrap();
    });
}
