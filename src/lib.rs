// Rust gives the data, Slint shows it. See CLAUDE.md.

slint::include_modules!();

mod appareil;
#[cfg(not(any(target_os = "ios", target_arch = "wasm32")))]
mod capture;
mod donnees;
mod navigation;

use appareil::Appareil;
use navigation::Navigateur;
use slint::{ComponentHandle, ModelRc, VecModel};
use std::cell::{Cell, RefCell};
use std::rc::Rc;

fn sans_icone(libelle: &str) -> OngletIcone {
    OngletIcone { libelle: libelle.into(), icone: Default::default() }
}

fn modele<T: Clone + 'static>(lignes: Vec<T>) -> ModelRc<T> {
    ModelRc::new(VecModel::from(lignes))
}

fn brancher_navigation(fenetre: &App) -> Rc<RefCell<Navigateur>> {
    let navigateur = Rc::new(RefCell::new(Navigateur::default()));
    let navigation = fenetre.global::<Navigation>();

    navigation.set_rubriques(modele(navigation::RUBRIQUES.iter().map(|r| sans_icone(r)).collect()));

    let icones = fenetre.global::<Icones>();
    let images = [
        icones.get_co_circuit(),
        icones.get_users(),
        icones.get_co_depart(),
        icones.get_co_boitier(),
        icones.get_radio(),
        icones.get_trophy(),
    ];
    navigation.set_onglets(modele(
        navigation::ONGLETS
            .iter()
            .zip(images)
            .map(|(libelle, icone)| OngletIcone { libelle: (*libelle).into(), icone })
            .collect(),
    ));

    // Every click changes the state in Rust, then the screen is refreshed from it.
    fn sur_clic(
        fenetre: &App,
        navigateur: &Rc<RefCell<Navigateur>>,
        action: impl Fn(&mut Navigateur, usize) + 'static,
    ) -> impl Fn(i32) + 'static {
        let (faible, navigateur) = (fenetre.as_weak(), navigateur.clone());
        move |i| {
            action(&mut navigateur.borrow_mut(), i as usize);
            if let Some(fenetre) = faible.upgrade() {
                navigateur.borrow().afficher(&fenetre);
            }
        }
    }
    navigation.on_choisir_rubrique(sur_clic(fenetre, &navigateur, Navigateur::choisir_rubrique));
    navigation.on_choisir_onglet(sur_clic(fenetre, &navigateur, Navigateur::choisir_onglet));
    navigation.on_ouvrir_evenement(sur_clic(fenetre, &navigateur, Navigateur::ouvrir_evenement));
    let revenir = sur_clic(fenetre, &navigateur, |n, _| n.revenir());
    navigation.on_revenir(move || revenir(0));

    navigateur.borrow().afficher(fenetre);
    navigateur
}

fn brancher_demo(fenetre: &App) -> Rc<Cell<Appareil>> {
    let choix = Rc::new(Cell::new(Appareil::Auto));
    let demo = fenetre.global::<Demo>();
    demo.set_appareils(modele(Appareil::TOUS.iter().map(|a| sans_icone(a.libelle())).collect()));

    let (faible, choix_clic) = (fenetre.as_weak(), choix.clone());
    demo.on_choisir_appareil(move |i| {
        choix_clic.set(Appareil::TOUS[i as usize]);
        if let Some(fenetre) = faible.upgrade() {
            appareil::appliquer(&fenetre, choix_clic.get());
        }
    });
    appareil::appliquer(fenetre, choix.get());
    choix
}

pub fn lancer() -> Result<(), slint::PlatformError> {
    let fenetre = App::new()?;
    let navigateur = brancher_navigation(&fenetre);
    let choix = brancher_demo(&fenetre);

    let faible = fenetre.as_weak();
    let _suivi_mouvements = appareil::suivre_moins_de_mouvements(move |actif| {
        if let Some(fenetre) = faible.upgrade() {
            fenetre.global::<Theme>().set_moins_de_mouvements(actif);
        }
    });

    #[cfg(not(any(target_os = "ios", target_arch = "wasm32")))]
    capture::preparer(&fenetre, &navigateur, &choix);
    #[cfg(any(target_os = "ios", target_arch = "wasm32"))]
    let _ = (&navigateur, &choix);

    fenetre.run()
}

// The web page calls this when the WebAssembly module loads.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn demarrer_web() {
    lancer().unwrap();
}
