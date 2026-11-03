// Rust gives the data, Slint shows it. See CLAUDE.md.

slint::include_modules!();

#[cfg(not(any(target_os = "ios", target_arch = "wasm32")))]
mod capture;
mod data;
mod device;
mod navigation;

use device::Device;
use navigation::Navigator;
use slint::{ComponentHandle, ModelRc, SharedString, VecModel};
use std::cell::{Cell, RefCell};
use std::rc::Rc;

fn model<T: Clone + 'static>(rows: Vec<T>) -> ModelRc<T> {
    ModelRc::new(VecModel::from(rows))
}

fn connect_navigation(window: &App) -> Rc<RefCell<Navigator>> {
    let navigator = Rc::new(RefCell::new(Navigator::default()));
    let navigation = window.global::<Navigation>();

    navigation.set_back_label(navigation::HOME.into());
    navigation.set_events(model(
        data::EVENTS
            .iter()
            .map(|e| EventItem { name: e.name.into(), detail: e.detail.into() })
            .collect(),
    ));
    navigation.set_club_pages(model(
        navigation::CLUB_PAGES.iter().map(|page| SharedString::from(*page)).collect(),
    ));

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
        navigator: &Rc<RefCell<Navigator>>,
        action: impl Fn(&mut Navigator, usize) + 'static,
    ) -> impl Fn(i32) + 'static {
        let (weak, navigator) = (window.as_weak(), navigator.clone());
        move |i| {
            action(&mut navigator.borrow_mut(), i as usize);
            if let Some(window) = weak.upgrade() {
                navigator.borrow().show(&window);
            }
        }
    }
    navigation.on_open_event(on_click(window, &navigator, Navigator::open_event));
    navigation.on_open_club_page(on_click(window, &navigator, Navigator::open_club_page));
    navigation.on_select_tab(on_click(window, &navigator, Navigator::select_tab));
    let go_back = on_click(window, &navigator, |n, _| n.go_home());
    navigation.on_go_back(move || go_back(0));

    navigator.borrow().show(window);
    navigator
}

fn connect_demo(window: &App) -> Rc<Cell<Device>> {
    let choice = Rc::new(Cell::new(Device::Auto));
    let demo = window.global::<Demo>();
    demo.set_devices(model(
        Device::ALL
            .iter()
            .map(|d| TabItem { label: d.label().into(), icon: Default::default() })
            .collect(),
    ));

    let (weak, clicked_choice) = (window.as_weak(), choice.clone());
    demo.on_select_device(move |i| {
        clicked_choice.set(Device::ALL[i as usize]);
        if let Some(window) = weak.upgrade() {
            device::apply(&window, clicked_choice.get());
        }
    });
    device::apply(window, choice.get());
    choice
}

pub fn run() -> Result<(), slint::PlatformError> {
    let window = App::new()?;
    let navigator = connect_navigation(&window);
    let choice = connect_demo(&window);

    let weak = window.as_weak();
    let _reduced_motion = device::follow_reduced_motion(move |active| {
        if let Some(window) = weak.upgrade() {
            window.global::<Theme>().set_reduced_motion(active);
        }
    });

    #[cfg(not(any(target_os = "ios", target_arch = "wasm32")))]
    capture::prepare(&window, &navigator, &choice);
    #[cfg(any(target_os = "ios", target_arch = "wasm32"))]
    let _ = (&navigator, &choice);

    window.run()
}

// The web page calls this when the WebAssembly module loads.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn start_web() {
    run().unwrap();
}
