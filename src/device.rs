// What the device decides, independently of the width: touch or mouse sizes, reduced motion.

use crate::{App, Demo, Theme};
use slint::{ComponentHandle, Timer, TimerMode};
use std::time::Duration;

#[derive(Clone, Copy, PartialEq)]
pub enum Device {
    Auto,
    Touch,
    Mouse,
}

impl Device {
    pub const ALL: [Device; 3] = [Device::Auto, Device::Touch, Device::Mouse];

    pub fn label(self) -> &'static str {
        match self {
            Device::Auto => "Auto",
            Device::Touch => "Tactile",
            Device::Mouse => "Souris",
        }
    }
}

#[cfg(any(target_os = "ios", target_os = "android"))]
fn has_touch_screen() -> bool {
    true
}

#[cfg(target_arch = "wasm32")]
fn has_touch_screen() -> bool {
    media_matches("(pointer: coarse)")
}

#[cfg(not(any(target_os = "ios", target_os = "android", target_arch = "wasm32")))]
fn has_touch_screen() -> bool {
    false
}

#[cfg(target_arch = "wasm32")]
fn media_matches(query: &str) -> bool {
    web_sys::window()
        .and_then(|window| window.match_media(query).ok().flatten())
        .is_some_and(|media| media.matches())
}

pub fn apply(window: &App, choice: Device) {
    let touch = match choice {
        Device::Auto => has_touch_screen(),
        Device::Touch => true,
        Device::Mouse => false,
    };
    window.global::<Theme>().set_touch(touch);
    let position = Device::ALL.iter().position(|d| *d == choice).unwrap_or(0);
    window.global::<Demo>().set_active_device(position as i32);
}

// The system's reduced motion setting.
#[cfg(target_os = "ios")]
fn reduced_motion() -> bool {
    #[link(name = "UIKit", kind = "framework")]
    unsafe extern "C" {
        fn UIAccessibilityIsReduceMotionEnabled() -> bool;
    }
    unsafe { UIAccessibilityIsReduceMotionEnabled() }
}

#[cfg(target_os = "macos")]
fn reduced_motion() -> bool {
    std::process::Command::new("defaults")
        .args(["read", "com.apple.universalaccess", "reduceMotion"])
        .output()
        .is_ok_and(|output| output.stdout.starts_with(b"1"))
}

#[cfg(target_arch = "wasm32")]
fn reduced_motion() -> bool {
    media_matches("(prefers-reduced-motion: reduce)")
}

#[cfg(not(any(target_os = "ios", target_os = "macos", target_arch = "wasm32")))]
fn reduced_motion() -> bool {
    false
}

// Applies the setting at launch, then reads it again every 3 seconds.
// Keep the returned timer: following stops when it is dropped.
pub fn follow_reduced_motion(mut apply: impl FnMut(bool) + 'static) -> Timer {
    apply(reduced_motion());
    let timer = Timer::default();
    timer.start(TimerMode::Repeated, Duration::from_secs(3), move || {
        apply(reduced_motion());
    });
    timer
}
