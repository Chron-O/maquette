// Capture mode, used by scripts/captures.sh:
//   maquette --capture FILE light|dark WIDTH HEIGHT touch|mouse [club|tab] [INDEX]
// With CAPTURE_TEST_DATA set, the test data comes from a fixed seed, so images stay comparable.
// Without these arguments, the app starts normally.

use crate::device::{self, Device};
use crate::{App, AppState, Theme, ThemeMode};
use slint::ComponentHandle;
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::Duration;

const WAIT: Duration = Duration::from_millis(1200);
const TEST_DATA_SEED: u64 = 42;
const READINGS: usize = 3;

pub fn prepare(window: &App, state: &Rc<RefCell<AppState>>, choice: &Rc<Cell<Device>>) {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 7 || args[1] != "--capture" {
        return;
    }
    let mode = if args[3] == "dark" { ThemeMode::Dark } else { ThemeMode::Light };
    window.global::<Theme>().set_mode(mode);

    let width: f32 = args[4].parse().expect("width");
    let height: f32 = args[5].parse().expect("height");
    window.window().set_size(slint::LogicalSize::new(width, height));

    choice.set(if args[6] == "touch" { Device::Touch } else { Device::Mouse });
    device::apply(window, choice.get());

    let mut state_now = state.borrow_mut();
    if std::env::var_os("CAPTURE_TEST_DATA").is_some() {
        state_now.start_test_data(TEST_DATA_SEED);
        if let Some(data) = state_now.data.as_mut() {
            for _ in 0..READINGS {
                data.read_a_card();
            }
        }
    }
    let index: usize = args.get(8).map_or(0, |i| i.parse().expect("index"));
    match args.get(7).map(String::as_str) {
        Some("tab") => {
            state_now.navigator.open_event(0);
            state_now.navigator.select_tab(index);
        }
        Some("club") => state_now.navigator.open_club_page(index),
        _ => {}
    }
    state_now.show(window);

    // Waits for the window to be drawn, saves the image, then closes.
    let path = args[2].clone();
    let weak = window.as_weak();
    slint::Timer::single_shot(WAIT, move || {
        let window = weak.unwrap();
        let image = window.window().take_snapshot().expect("snapshot failed");
        image::save_buffer(
            &path,
            image.as_bytes(),
            image.width(),
            image.height(),
            image::ColorType::Rgba8,
        )
        .expect("saving failed");
        slint::quit_event_loop().unwrap();
    });
}
