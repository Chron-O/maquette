// Capture mode, used by scripts/captures.sh:
//   maquette --capture FILE light|dark WIDTH HEIGHT touch|mouse [club|tab] [INDEX]
// Without these arguments, the app starts normally.

use crate::device::{self, Device};
use crate::navigation::Navigator;
use crate::{App, Theme, ThemeMode};
use slint::ComponentHandle;
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::Duration;

const WAIT: Duration = Duration::from_millis(1200);

pub fn prepare(window: &App, navigator: &Rc<RefCell<Navigator>>, choice: &Rc<Cell<Device>>) {
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

    let index: usize = args.get(8).map_or(0, |i| i.parse().expect("index"));
    match args.get(7).map(String::as_str) {
        Some("tab") => {
            let mut navigator = navigator.borrow_mut();
            navigator.open_event(0);
            navigator.select_tab(index);
        }
        Some("club") => navigator.borrow_mut().open_club_page(index),
        _ => {}
    }
    navigator.borrow().show(window);

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
