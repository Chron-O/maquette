// Rust gives the data, Slint shows it. See CLAUDE.md.

slint::include_modules!();

#[cfg(not(any(target_os = "ios", target_arch = "wasm32")))]
mod capture;
mod courses;
mod device;
mod home;
mod navigation;
mod test_data;

use device::Device;
use navigation::Navigator;
use slint::{ComponentHandle, ModelRc, Timer, TimerMode, VecModel};
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::Duration;
use test_data::{CURRENT_EVENT, Generated};

const SECONDS_BETWEEN_READINGS: u64 = 3;

fn model<T: Clone + 'static>(rows: Vec<T>) -> ModelRc<T> {
    ModelRc::new(VecModel::from(rows))
}

fn word(label: &str) -> TabItem {
    TabItem { label: label.into(), icon: Default::default() }
}

// All the app's state: where we are, and the test data if the generator is on.
#[derive(Default)]
pub struct AppState {
    navigator: Navigator,
    courses: courses::CoursesState,
    data: Option<Generated>,
}

impl AppState {
    fn show(&self, window: &App) {
        self.navigator.show(window, self.data.as_ref());
        home::show(window, self.data.as_ref());
        let event = self.navigator.event().and_then(|e| self.data.as_ref().map(|d| &d.events[e]));
        self.courses.show(window, event);
    }

    fn start_test_data(&mut self, seed: u64) {
        self.data = Some(Generated::new(seed));
    }

    fn stop_test_data(&mut self) {
        self.data = None;
        self.navigator.leave_event();
    }
}

#[cfg(target_arch = "wasm32")]
fn random_seed() -> u64 {
    js_sys::Date::now() as u64
}

#[cfg(not(target_arch = "wasm32"))]
fn random_seed() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(1, |d| d.as_secs())
}

fn connect_navigation(window: &App, state: &Rc<RefCell<AppState>>) {
    let navigation = window.global::<Navigation>();
    navigation.set_back_label(navigation::HOME.into());

    let icons = window.global::<Icons>();
    let images = [
        icons.get_co_course(),
        icons.get_users(),
        icons.get_co_start(),
        icons.get_co_station(),
        icons.get_radio(),
        icons.get_trophy(),
    ];
    navigation.set_tabs(model(
        navigation::TABS
            .iter()
            .zip(images)
            .map(|(label, icon)| TabItem { label: (*label).into(), icon })
            .collect(),
    ));

    // Every click changes the state in Rust, then the screen is refreshed from it.
    fn on_click(
        window: &App,
        state: &Rc<RefCell<AppState>>,
        action: impl Fn(&mut Navigator, usize) + 'static,
    ) -> impl Fn(i32) + 'static {
        let (weak, state) = (window.as_weak(), state.clone());
        move |i| {
            action(&mut state.borrow_mut().navigator, i as usize);
            if let Some(window) = weak.upgrade() {
                state.borrow().show(&window);
            }
        }
    }
    // Opening an event starts its Circuits tab from the beginning.
    let open_event = {
        let (weak, state) = (window.as_weak(), state.clone());
        move |i: i32| {
            let mut state = state.borrow_mut();
            state.navigator.open_event(i as usize);
            state.courses = courses::CoursesState::default();
            if let Some(window) = weak.upgrade() {
                state.show(&window);
            }
        }
    };
    navigation.on_open_event(open_event);
    navigation.on_open_club_page(on_click(window, state, Navigator::open_club_page));
    navigation.on_select_tab(on_click(window, state, Navigator::select_tab));
    let open_direct = on_click(window, state, |n, _| n.open_direct(CURRENT_EVENT));
    navigation.on_open_direct(move || open_direct(0));
    let go_back = on_click(window, state, |n, _| n.go_home());
    navigation.on_go_back(move || go_back(0));

    fn on_course_click<T: 'static>(
        window: &App,
        state: &Rc<RefCell<AppState>>,
        action: impl Fn(&mut courses::CoursesState, T) + 'static,
    ) -> impl Fn(T) + 'static {
        let (weak, state) = (window.as_weak(), state.clone());
        move |value| {
            action(&mut state.borrow_mut().courses, value);
            if let Some(window) = weak.upgrade() {
                state.borrow().show(&window);
            }
        }
    }
    let courses = window.global::<Courses>();
    courses.on_select_course(on_course_click(window, state, |c, i: i32| c.select_course(i)));
    courses.on_select_view(on_course_click(window, state, |c, i: i32| c.select_view(i as usize)));
    courses.on_select_highlight(on_course_click(window, state, |c, i: i32| c.select_highlight(i as usize)));
    courses.on_sort_by(on_course_click(window, state, |c, i: i32| c.sort_by(i)));
    courses.on_select_control(on_course_click(window, state, |c, code: slint::SharedString| c.select_control(&code)));
    courses.on_code_search(on_course_click(window, state, |c, text: slint::SharedString| c.set_code_search(&text)));
    courses.on_course_search(on_course_click(window, state, |c, text: slint::SharedString| c.set_course_search(&text)));
    let close = on_course_click(window, state, |c, _: ()| c.close_detail());
    courses.on_close_detail(move || close(()));
}

fn connect_demo(window: &App, state: &Rc<RefCell<AppState>>) -> (Rc<Cell<Device>>, Rc<Timer>) {
    let choice = Rc::new(Cell::new(Device::Auto));
    let demo = window.global::<Demo>();
    demo.set_devices(model(Device::ALL.iter().map(|d| word(d.label())).collect()));

    let (weak, clicked_choice) = (window.as_weak(), choice.clone());
    demo.on_select_device(move |i| {
        clicked_choice.set(Device::ALL[i as usize]);
        if let Some(window) = weak.upgrade() {
            device::apply(&window, clicked_choice.get());
        }
    });
    device::apply(window, choice.get());

    demo.set_test_data_options(model(vec![word("Arrêt"), word("Marche")]));
    let readings = Rc::new(Timer::default());
    let (weak, state_for_switch, readings_for_switch) = (window.as_weak(), state.clone(), readings.clone());
    demo.on_select_test_data(move |i| {
        let Some(window) = weak.upgrade() else { return };
        window.global::<Demo>().set_active_test_data(i);
        if i == 1 {
            state_for_switch.borrow_mut().start_test_data(random_seed());
            let (weak, state) = (window.as_weak(), state_for_switch.clone());
            readings_for_switch.start(TimerMode::Repeated, Duration::from_secs(SECONDS_BETWEEN_READINGS), move || {
                let mut state = state.borrow_mut();
                if let Some(data) = state.data.as_mut() {
                    data.read_a_card();
                }
                if let Some(window) = weak.upgrade() {
                    state.show(&window);
                }
            });
        } else {
            readings_for_switch.stop();
            state_for_switch.borrow_mut().stop_test_data();
        }
        state_for_switch.borrow().show(&window);
    });
    (choice, readings)
}

pub fn run() -> Result<(), slint::PlatformError> {
    let window = App::new()?;
    let state = Rc::new(RefCell::new(AppState::default()));
    connect_navigation(&window, &state);
    let (choice, _readings) = connect_demo(&window, &state);
    state.borrow().show(&window);

    let weak = window.as_weak();
    let _reduced_motion = device::follow_reduced_motion(move |active| {
        if let Some(window) = weak.upgrade() {
            window.global::<Theme>().set_reduced_motion(active);
        }
    });

    #[cfg(not(any(target_os = "ios", target_arch = "wasm32")))]
    capture::prepare(&window, &state, &choice);
    #[cfg(any(target_os = "ios", target_arch = "wasm32"))]
    let _ = &choice;

    window.run()
}

// The web page calls this when the WebAssembly module loads.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn start_web() {
    run().unwrap();
}
