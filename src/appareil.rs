// What the device decides, independently of the width: touch or mouse sizes, reduced motion.

use crate::{App, Demo, Theme};
use slint::{ComponentHandle, Timer, TimerMode};
use std::time::Duration;

#[derive(Clone, Copy, PartialEq)]
pub enum Appareil {
    Auto,
    Tactile,
    Souris,
}

impl Appareil {
    pub const TOUS: [Appareil; 3] = [Appareil::Auto, Appareil::Tactile, Appareil::Souris];

    pub fn libelle(self) -> &'static str {
        match self {
            Appareil::Auto => "Auto",
            Appareil::Tactile => "Tactile",
            Appareil::Souris => "Souris",
        }
    }
}

#[cfg(any(target_os = "ios", target_os = "android"))]
fn ecran_tactile() -> bool {
    true
}

#[cfg(target_arch = "wasm32")]
fn ecran_tactile() -> bool {
    correspond_au_media("(pointer: coarse)")
}

#[cfg(not(any(target_os = "ios", target_os = "android", target_arch = "wasm32")))]
fn ecran_tactile() -> bool {
    false
}

#[cfg(target_arch = "wasm32")]
fn correspond_au_media(requete: &str) -> bool {
    web_sys::window()
        .and_then(|fenetre| fenetre.match_media(requete).ok().flatten())
        .is_some_and(|media| media.matches())
}

pub fn appliquer(fenetre: &App, choix: Appareil) {
    let tactile = match choix {
        Appareil::Auto => ecran_tactile(),
        Appareil::Tactile => true,
        Appareil::Souris => false,
    };
    fenetre.global::<Theme>().set_tactile(tactile);
    let position = Appareil::TOUS.iter().position(|a| *a == choix).unwrap_or(0);
    fenetre.global::<Demo>().set_appareil_actif(position as i32);
}

// The system's reduced motion setting.
#[cfg(target_os = "ios")]
fn moins_de_mouvements() -> bool {
    #[link(name = "UIKit", kind = "framework")]
    unsafe extern "C" {
        fn UIAccessibilityIsReduceMotionEnabled() -> bool;
    }
    unsafe { UIAccessibilityIsReduceMotionEnabled() }
}

#[cfg(target_os = "macos")]
fn moins_de_mouvements() -> bool {
    std::process::Command::new("defaults")
        .args(["read", "com.apple.universalaccess", "reduceMotion"])
        .output()
        .is_ok_and(|sortie| sortie.stdout.starts_with(b"1"))
}

#[cfg(target_arch = "wasm32")]
fn moins_de_mouvements() -> bool {
    correspond_au_media("(prefers-reduced-motion: reduce)")
}

#[cfg(not(any(target_os = "ios", target_os = "macos", target_arch = "wasm32")))]
fn moins_de_mouvements() -> bool {
    false
}

// Applies the setting at launch, then reads it again every 3 seconds.
// Keep the returned timer: following stops when it is dropped.
pub fn suivre_moins_de_mouvements(mut appliquer: impl FnMut(bool) + 'static) -> Timer {
    appliquer(moins_de_mouvements());
    let minuteur = Timer::default();
    minuteur.start(TimerMode::Repeated, Duration::from_secs(3), move || {
        appliquer(moins_de_mouvements());
    });
    minuteur
}
