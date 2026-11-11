// Capture mode, used by scripts/captures.sh:
//   maquette --capture FILE light|dark WIDTH HEIGHT touch|mouse [club|tab|live] [INDEX]
// « live » opens the Direct drawer over the tab INDEX.
// With CAPTURE_TEST_DATA set, the test data comes from a fixed seed, so images stay comparable.
// CAPTURE_COURSE and CAPTURE_COURSE_VIEW open a course and one of its views.
// CAPTURE_CLICKS="x,y;x,y" clicks these points first, to test the navigation without a mouse;
// "~x,y,dy" scrolls by dy points there instead (a negative dy goes down the page).
// Without these arguments, the app starts normally.

use crate::device::{self, Device};
use crate::{App, AppState, Theme, ThemeMode};
use slint::platform::{PointerEventButton, WindowEvent};
use slint::{ComponentHandle, LogicalPosition};
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::Duration;

const WAIT: Duration = Duration::from_millis(1200);
const TEST_DATA_SEED: u64 = 42;
const READINGS: usize = 3;
const LIVE_AGE: u32 = 4;

pub fn prepare(window: &App, state: &Rc<RefCell<AppState>>, choice: &Rc<Cell<Device>>) {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 7 || args[1] != "--capture" {
        return;
    }
    let mode = if args[3] == "dark" { ThemeMode::Dark } else { ThemeMode::Light };
    window.global::<Theme>().set_mode(mode);

    let width: f32 = args[4].parse().expect("width");
    let height: f32 = args[5].parse().expect("height");
    set_logical_size(window, width, height);

    choice.set(if args[6] == "touch" { Device::Touch } else { Device::Mouse });
    device::apply(window, choice.get());

    let mut state_now = state.borrow_mut();
    if std::env::var_os("CAPTURE_TEST_DATA").is_some() {
        state_now.start_test_data(TEST_DATA_SEED);
        if let Some(data) = state_now.data.as_mut() {
            for _ in 0..READINGS {
                data.read_a_card();
            }
            // A few seconds after the last reading: still live, the reader ready again.
            data.wait(LIVE_AGE);
        }
    }
    let index: usize = args.get(8).map_or(0, |i| i.parse().expect("index"));
    match args.get(7).map(String::as_str) {
        Some(kind @ ("tab" | "live")) => {
            state_now.navigator.open_event(0);
            state_now.navigator.select_tab(index);
            if kind == "live" {
                state_now.navigator.toggle_live();
            }
            state_now.courses.select_course(-1);
            if let Ok(view) = std::env::var("CAPTURE_COURSES_VIEW") {
                state_now.courses.select_view(view.parse().expect("view"));
            }
            if let Ok(course) = std::env::var("CAPTURE_COURSE") {
                state_now.courses.select_course(course.parse().expect("course"));
            }
            if let Ok(view) = std::env::var("CAPTURE_COURSE_VIEW") {
                state_now.courses.select_course_view(view.parse().expect("view"));
            }
        }
        Some("club") => state_now.navigator.open_club_page(index),
        _ => {}
    }
    state_now.show(window);

    // Clicks one point after the other, then leaves time for the screen to update.
    // Each step: a point, and how far to scroll there (None: a click).
    let clicks: Vec<(LogicalPosition, Option<f32>)> = std::env::var("CAPTURE_CLICKS").map_or(Vec::new(), |list| {
        list.split(';')
            .map(|step| {
                let (scroll, point) = match step.strip_prefix('~') {
                    Some(rest) => {
                        let (point, dy) = rest.rsplit_once(',').expect("x,y,dy");
                        (Some(dy.parse().expect("dy")), point)
                    }
                    None => (None, step),
                };
                let (x, y) = point.split_once(',').expect("x,y");
                (LogicalPosition::new(x.parse().expect("x"), y.parse().expect("y")), scroll)
            })
            .collect()
    });
    // The system may shrink the window when it opens: its size is given again once it is shown.
    let resize = window.as_weak();
    slint::Timer::single_shot(WAIT / 2, move || {
        set_logical_size(&resize.unwrap(), width, height);
    });
    let click_time = WAIT * (clicks.len() as u32 + 2);
    for (i, (position, scroll)) in clicks.into_iter().enumerate() {
        let weak = window.as_weak();
        slint::Timer::single_shot(WAIT * (i as u32 + 1), move || {
            let window = weak.unwrap();
            window.window().dispatch_event(WindowEvent::PointerMoved { position });
            if let Some(delta_y) = scroll {
                window.window().dispatch_event(WindowEvent::PointerScrolled { position, delta_x: 0.0, delta_y });
                return;
            }
            let button = PointerEventButton::Left;
            window.window().dispatch_event(WindowEvent::PointerPressed { position, button });
            window.window().dispatch_event(WindowEvent::PointerReleased { position, button });
        });
    }

    // Waits for the window to be drawn, saves the image, then closes.
    let path = args[2].clone();
    let weak = window.as_weak();
    slint::Timer::single_shot(click_time, move || {
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

// The window in the app's own points, whatever scale the system gives the window: a forced scale
// factor (SLINT_SCALE_FACTOR) and the screen's own can differ.
fn set_logical_size(window: &App, width: f32, height: f32) {
    let scale = window.window().scale_factor();
    window.window().set_size(slint::PhysicalSize::new((width * scale).round() as u32, (height * scale).round() as u32));
}
