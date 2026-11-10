// Rust gives the data, Slint shows it. See CLAUDE.md.

slint::include_modules!();

#[cfg(not(any(target_os = "ios", target_arch = "wasm32")))]
mod capture;
mod cards;
mod club_stations;
mod courses;
mod device;
mod home;
mod live;
mod navigation;
mod participants;
mod readers;
mod results;
mod runner_base;
mod starts;
mod stations;
mod test_data;

use device::Device;
use navigation::Navigator;
use slint::{ComponentHandle, ModelRc, Timer, TimerMode, VecModel};
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::Duration;
use test_data::{CURRENT_EVENT, Generated};


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
    participants: participants::ParticipantsState,
    starts: starts::StartsState,
    stations: stations::StationsState,
    live: live::LiveState,
    results: results::ResultsState,
    cards: cards::CardsState,
    club_stations: club_stations::ClubStationsState,
    runner_base: runner_base::RunnerBaseState,
    data: Option<Generated>,
}

impl AppState {
    fn show(&self, window: &App) {
        self.navigator.show(window, self.data.as_ref());
        home::show(window, self.data.as_ref());
        readers::show(window, self.data.as_ref());
        let event = self.navigator.event().and_then(|e| self.data.as_ref().map(|d| &d.events[e]));
        self.courses.show(window, event, &self.stations);
        let cards_free = self.data.as_ref().map_or(0, Generated::cards_free);
        self.participants.show(window, event, cards_free);
        self.starts.show(window, event);
        self.stations.show(window, event);
        let (readers, clock) = self.data.as_ref().map_or((&[][..], 0), |d| (&d.readers[..], d.clock()));
        self.live.show(window, event, readers, clock);
        self.results.show(window, event, event.and_then(|e| self.live.selected_runner(e)));
        self.cards.show(window, self.data.as_ref());
        self.club_stations.show(window, self.data.as_ref());
        self.runner_base.show(window, self.data.as_ref());
    }

    fn start_test_data(&mut self, seed: u64) {
        self.data = Some(Generated::new(seed));
    }

    fn stop_test_data(&mut self) {
        self.data = None;
        self.navigator.leave_event();
        self.reset_event_states();
        self.reset_club_states();
    }

    // What the tabs remember belongs to one event: it must never reach another one.
    fn reset_event_states(&mut self) {
        self.courses = courses::CoursesState::default();
        self.participants = participants::ParticipantsState::default();
        self.starts = starts::StartsState::default();
        self.stations = stations::StationsState::default();
        self.live = live::LiveState::default();
        self.results = results::ResultsState::default();
    }

    fn reset_club_states(&mut self) {
        self.cards = cards::CardsState::default();
        self.club_stations = club_stations::ClubStationsState::default();
        self.runner_base = runner_base::RunnerBaseState::default();
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
    // Opening an event starts it from the beginning, on its Participants.
    let open_event = {
        let (weak, state) = (window.as_weak(), state.clone());
        move |i: i32| {
            let mut state = state.borrow_mut();
            state.navigator.open_event(i as usize);
            state.reset_event_states();
            if let Some(window) = weak.upgrade() {
                state.show(&window);
            }
        }
    };
    navigation.on_open_event(open_event);
    // Opening a club page starts it from the beginning.
    let (weak, state_for_page) = (window.as_weak(), state.clone());
    navigation.on_open_club_page(move |page| {
        let mut state = state_for_page.borrow_mut();
        state.navigator.open_club_page(page as usize);
        state.reset_club_states();
        if let Some(window) = weak.upgrade() {
            state.show(&window);
        }
    });
    // The Direct and the Résultats share their detail: it stays behind when leaving them.
    let (weak, state_for_tab) = (window.as_weak(), state.clone());
    navigation.on_select_tab(move |tab| {
        let mut state = state_for_tab.borrow_mut();
        state.navigator.select_tab(tab as usize);
        state.live.leave();
        if let Some(window) = weak.upgrade() {
            state.show(&window);
        }
    });
    // The Direct and the Résultats share their detail: it closes with the drawer too.
    let (weak, state_for_live) = (window.as_weak(), state.clone());
    navigation.on_toggle_live(move || {
        let mut state = state_for_live.borrow_mut();
        state.navigator.toggle_live();
        state.live.leave();
        if let Some(window) = weak.upgrade() {
            state.show(&window);
        }
    });
    // From the home page, « à traiter » opens the Direct on what is to handle (prototype: openEv(2)).
    let (weak, state_for_direct) = (window.as_weak(), state.clone());
    navigation.on_open_direct(move || {
        let mut state = state_for_direct.borrow_mut();
        state.navigator.open_direct(CURRENT_EVENT);
        state.reset_event_states();
        state.live.show_to_handle();
        if let Some(window) = weak.upgrade() {
            state.show(&window);
        }
    });
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
    courses.on_select_course_view(on_course_click(window, state, |c, i: i32| c.select_course_view(i as usize)));
    courses.on_select_stop(on_course_click(window, state, |c, i: i32| c.select_stop(i as usize)));
    let (weak, state_for_reserve) = (window.as_weak(), state.clone());
    courses.on_select_reserve(move |code| {
        let state = &mut *state_for_reserve.borrow_mut();
        let event = state.navigator.event().and_then(|e| state.data.as_ref().map(|d| &d.events[e]));
        state.courses.select_reserve(&code, event);
        if let Some(window) = weak.upgrade() {
            state.show(&window);
        }
    });
    courses.on_show_in_all_controls(on_course_click(window, state, |c, code: slint::SharedString| c.show_in_all_controls(&code)));
    let others = on_course_click(window, state, |c, _: ()| c.toggle_others());
    courses.on_toggle_others(move || others(()));
    let close = on_course_click(window, state, |c, _: ()| c.close_detail());
    courses.on_close_detail(move || close(()));
}

fn connect_participants(window: &App, state: &Rc<RefCell<AppState>>) {
    fn on_click<T: 'static>(
        window: &App,
        state: &Rc<RefCell<AppState>>,
        action: impl Fn(&mut participants::ParticipantsState, T) + 'static,
    ) -> impl Fn(T) + 'static {
        let (weak, state) = (window.as_weak(), state.clone());
        move |value| {
            action(&mut state.borrow_mut().participants, value);
            if let Some(window) = weak.upgrade() {
                state.borrow().show(&window);
            }
        }
    }
    let page = window.global::<Participants>();
    page.on_select_row(on_click(window, state, |p, i: i32| p.select_row(i as usize)));
    page.on_sort_by(on_click(window, state, |p, i: i32| p.sort_by(i)));
    let toggle = on_click(window, state, |p, (g, c): (i32, i32)| p.toggle_filter(g as usize, c as usize));
    page.on_toggle_filter(move |g, c| toggle((g, c)));
    let clear = on_click(window, state, |p, _: ()| p.clear_filters());
    page.on_clear_filters(move || clear(()));
    page.on_show_problem(on_click(window, state, |p, i: i32| p.show_problem(i as usize)));
    page.on_search(on_click(window, state, |p, text: slint::SharedString| p.set_search(&text)));
    let close = on_click(window, state, |p, _: ()| p.close_detail());
    page.on_close_detail(move || close(()));
}

fn connect_starts(window: &App, state: &Rc<RefCell<AppState>>) {
    fn on_click<T: 'static>(
        window: &App,
        state: &Rc<RefCell<AppState>>,
        action: impl Fn(&mut starts::StartsState, T) + 'static,
    ) -> impl Fn(T) + 'static {
        let (weak, state) = (window.as_weak(), state.clone());
        move |value| {
            action(&mut state.borrow_mut().starts, value);
            if let Some(window) = weak.upgrade() {
                state.borrow().show(&window);
            }
        }
    }
    // These change the test data of the open event.
    fn on_data_click(
        window: &App,
        state: &Rc<RefCell<AppState>>,
        action: impl Fn(&mut starts::StartsState, &mut Generated, usize) + 'static,
    ) -> impl Fn() + 'static {
        let (weak, state) = (window.as_weak(), state.clone());
        move || {
            {
                let app = &mut *state.borrow_mut();
                if let (Some(event), Some(data)) = (app.navigator.event(), app.data.as_mut()) {
                    action(&mut app.starts, data, event);
                }
            }
            if let Some(window) = weak.upgrade() {
                state.borrow().show(&window);
            }
        }
    }
    let page = window.global::<Starts>();
    page.on_select_row(on_click(window, state, |s, i: i32| s.select_row(i as usize)));
    page.on_sort_by(on_click(window, state, |s, i: i32| s.sort_by(i)));
    page.on_select_group(on_click(window, state, |s, i: i32| s.select_group(i as usize)));
    page.on_select_point(on_click(window, state, |s, i: i32| s.select_point(i as usize)));
    page.on_select_view(on_click(window, state, |s, i: i32| s.select_view(i as usize)));
    page.on_search(on_click(window, state, |s, text: slint::SharedString| s.set_search(&text)));
    let close = on_click(window, state, |s, _: ()| s.close_detail());
    page.on_close_detail(move || close(()));
    let abandon = on_click(window, state, |s, _: ()| s.preview = None);
    page.on_abandon(move || abandon(()));
    page.on_draw(on_data_click(window, state, |s, data, event| s.preview = Some(data.draw(event))));
    page.on_apply(on_data_click(window, state, |s, data, event| s.apply(&mut data.events[event])));
    page.on_publish(on_data_click(window, state, |_, data, event| data.publish(event)));
    page.on_toggle_lock(on_data_click(window, state, |s, data, event| s.toggle_lock(&mut data.events[event])));
}

fn connect_stations(window: &App, state: &Rc<RefCell<AppState>>) {
    fn on_click<T: 'static>(
        window: &App,
        state: &Rc<RefCell<AppState>>,
        action: impl Fn(&mut stations::StationsState, T) + 'static,
    ) -> impl Fn(T) + 'static {
        let (weak, state) = (window.as_weak(), state.clone());
        move |value| {
            action(&mut state.borrow_mut().stations, value);
            if let Some(window) = weak.upgrade() {
                state.borrow().show(&window);
            }
        }
    }
    // These change the test data of the open event.
    fn on_data_click<T: 'static>(
        window: &App,
        state: &Rc<RefCell<AppState>>,
        action: impl Fn(&mut stations::StationsState, &mut Generated, usize, T) + 'static,
    ) -> impl Fn(T) + 'static {
        let (weak, state) = (window.as_weak(), state.clone());
        move |value| {
            {
                let app = &mut *state.borrow_mut();
                if let (Some(event), Some(data)) = (app.navigator.event(), app.data.as_mut()) {
                    action(&mut app.stations, data, event, value);
                }
            }
            if let Some(window) = weak.upgrade() {
                state.borrow().show(&window);
            }
        }
    }
    let page = window.global::<Stations>();
    page.on_select_mode(on_click(window, state, |s, i: i32| s.select_mode(i as usize)));
    page.on_select_filter(on_click(window, state, |s, i: i32| s.select_filter(i as usize)));
    page.on_select_row(on_click(window, state, |s, i: i32| s.select_row(i as usize)));
    page.on_sort_by(on_click(window, state, |s, i: i32| s.sort_by(i)));
    page.on_search(on_click(window, state, |s, text: slint::SharedString| s.set_search(&text)));
    let close = on_click(window, state, |s, _: ()| s.close_detail());
    page.on_close_detail(move || close(()));
    let confirm = on_data_click(window, state, |s, data, event, _: ()| s.confirm_reading(&mut data.events[event]));
    page.on_confirm_reading(move || confirm(()));
    page.on_add_row(on_data_click(window, state, |s, data, event, i: i32| s.add_to_event(&mut data.events[event], i as usize)));
    let alert = on_data_click(window, state, |s, data, event, _: ()| s.alert_action(&mut data.events[event]));
    page.on_alert_action(move || alert(()));
    let read = on_data_click(window, state, |s, data, event, _: ()| {
        let punches = data.random_punches();
        s.read_next(&mut data.events[event], punches);
    });
    // The same button of the Démo panel reads a station of the club on its page.
    let (weak, state_for_read) = (window.as_weak(), state.clone());
    window.global::<Demo>().on_read_station(move || {
        if !state_for_read.borrow().navigator.on_stations_page() {
            read(());
            return;
        }
        {
            let app = &mut *state_for_read.borrow_mut();
            if let Some(data) = app.data.as_mut() {
                app.club_stations.read(data);
            }
        }
        if let Some(window) = weak.upgrade() {
            state_for_read.borrow().show(&window);
        }
    });
}

fn connect_live(window: &App, state: &Rc<RefCell<AppState>>) {
    fn on_click<T: 'static>(
        window: &App,
        state: &Rc<RefCell<AppState>>,
        action: impl Fn(&mut live::LiveState, T) + 'static,
    ) -> impl Fn(T) + 'static {
        let (weak, state) = (window.as_weak(), state.clone());
        move |value| {
            action(&mut state.borrow_mut().live, value);
            if let Some(window) = weak.upgrade() {
                state.borrow().show(&window);
            }
        }
    }
    // These read or change the test data of the open event.
    fn on_data_click<T: 'static>(
        window: &App,
        state: &Rc<RefCell<AppState>>,
        action: impl Fn(&mut live::LiveState, &mut test_data::Event, u32, T) + 'static,
    ) -> impl Fn(T) + 'static {
        let (weak, state) = (window.as_weak(), state.clone());
        move |value| {
            {
                let app = &mut *state.borrow_mut();
                if let (Some(event), Some(data)) = (app.navigator.event(), app.data.as_mut()) {
                    let clock = data.clock();
                    action(&mut app.live, &mut data.events[event], clock, value);
                }
            }
            if let Some(window) = weak.upgrade() {
                state.borrow().show(&window);
            }
        }
    }
    let page = window.global::<Live>();
    page.on_select_course(on_click(window, state, |l, i: i32| l.select_course(i as usize)));
    page.on_select_reader(on_click(window, state, |l, i: i32| l.select_reader(i as usize)));
    page.on_select_row(on_click(window, state, |l, i: i32| l.select_row(i as usize)));
    page.on_open_row(on_click(window, state, |l, i: i32| l.open_row(i as usize)));
    page.on_toggle_list(on_click(window, state, |l, i: i32| l.toggle_list(i as usize)));
    page.on_search(on_click(window, state, |l, text: slint::SharedString| l.set_search(&text)));
    let close = on_click(window, state, |l, _: ()| l.close_detail());
    page.on_close_detail(move || close(()));
    let to_handle = on_click(window, state, |l, _: ()| l.toggle_to_handle());
    page.on_toggle_to_handle(move || to_handle(()));
    let only_to_handle = on_click(window, state, |l, _: ()| l.show_to_handle());
    page.on_show_to_handle(move || only_to_handle(()));
    page.on_select_runner(on_data_click(window, state, |l, event, _, i: i32| l.select_runner(event, i as usize)));
    page.on_give_to(on_data_click(window, state, |l, event, clock, i: i32| l.give_to(event, i as usize, clock)));
    let keep = on_data_click(window, state, |l, event, clock, _: ()| l.keep_first(event, clock));
    page.on_keep_first(move || keep(()));
}

fn connect_results(window: &App, state: &Rc<RefCell<AppState>>) {
    fn on_click<T: 'static>(
        window: &App,
        state: &Rc<RefCell<AppState>>,
        action: impl Fn(&mut results::ResultsState, T) + 'static,
    ) -> impl Fn(T) + 'static {
        let (weak, state) = (window.as_weak(), state.clone());
        move |value| {
            action(&mut state.borrow_mut().results, value);
            if let Some(window) = weak.upgrade() {
                state.borrow().show(&window);
            }
        }
    }
    let page = window.global::<Results>();
    let (weak, state_for_mode) = (window.as_weak(), state.clone());
    page.on_select_mode(move |mode| {
        let mut state = state_for_mode.borrow_mut();
        state.results.select_mode(mode as usize);
        state.live.set_ranked_by_course(mode == 1);
        if let Some(window) = weak.upgrade() {
            state.show(&window);
        }
    });
    page.on_select_ranking(on_click(window, state, |r, i: i32| r.select_ranking(i as usize)));
    page.on_select_category(on_click(window, state, |r, i: i32| r.select_category(i as usize)));
    page.on_search(on_click(window, state, |r, text: slint::SharedString| r.set_search(&text)));
    page.on_search_ranking(on_click(window, state, |r, text: slint::SharedString| r.set_ranking_search(&text)));
}

fn connect_cards(window: &App, state: &Rc<RefCell<AppState>>) {
    fn on_click<T: 'static>(
        window: &App,
        state: &Rc<RefCell<AppState>>,
        action: impl Fn(&mut cards::CardsState, T) + 'static,
    ) -> impl Fn(T) + 'static {
        let (weak, state) = (window.as_weak(), state.clone());
        move |value| {
            action(&mut state.borrow_mut().cards, value);
            if let Some(window) = weak.upgrade() {
                state.borrow().show(&window);
            }
        }
    }
    // These read or change the test data.
    fn on_data_click<T: 'static>(
        window: &App,
        state: &Rc<RefCell<AppState>>,
        action: impl Fn(&mut cards::CardsState, &mut Generated, T) + 'static,
    ) -> impl Fn(T) + 'static {
        let (weak, state) = (window.as_weak(), state.clone());
        move |value| {
            {
                let app = &mut *state.borrow_mut();
                if let Some(data) = app.data.as_mut() {
                    action(&mut app.cards, data, value);
                }
            }
            if let Some(window) = weak.upgrade() {
                state.borrow().show(&window);
            }
        }
    }
    let page = window.global::<Cards>();
    page.on_select_park(on_click(window, state, |c, i: i32| c.select_park(i as usize)));
    page.on_select_filter(on_click(window, state, |c, i: i32| c.select_filter(i as usize)));
    page.on_select_row(on_click(window, state, |c, number: i32| c.select_row(number as u32)));
    page.on_search(on_click(window, state, |c, text: slint::SharedString| c.set_search(&text)));
    let close = on_click(window, state, |c, _: ()| c.close_detail());
    page.on_close_detail(move || close(()));
    let cancel = on_click(window, state, |c, _: ()| c.cancel_confirm());
    page.on_cancel_confirm(move || cancel(()));
    page.on_open_card(on_data_click(window, state, |c, data, number: i32| c.open_card(data, number as u32)));
    page.on_act(on_data_click(window, state, |c, data, id: i32| c.act(data, id)));
    let confirm = on_data_click(window, state, |c, data, _: ()| c.confirm(data));
    page.on_confirm(move || confirm(()));
    // The Démo panel puts a card of the stock on the reader.
    let read = on_data_click(window, state, |c, data, _: ()| c.read(data));
    window.global::<Demo>().on_read_card(move || read(()));
}

fn connect_club_stations(window: &App, state: &Rc<RefCell<AppState>>) {
    fn on_click<T: 'static>(
        window: &App,
        state: &Rc<RefCell<AppState>>,
        action: impl Fn(&mut club_stations::ClubStationsState, T) + 'static,
    ) -> impl Fn(T) + 'static {
        let (weak, state) = (window.as_weak(), state.clone());
        move |value| {
            action(&mut state.borrow_mut().club_stations, value);
            if let Some(window) = weak.upgrade() {
                state.borrow().show(&window);
            }
        }
    }
    // These read or change the test data.
    fn on_data_click<T: 'static>(
        window: &App,
        state: &Rc<RefCell<AppState>>,
        action: impl Fn(&mut club_stations::ClubStationsState, &mut Generated, T) + 'static,
    ) -> impl Fn(T) + 'static {
        let (weak, state) = (window.as_weak(), state.clone());
        move |value| {
            {
                let app = &mut *state.borrow_mut();
                if let Some(data) = app.data.as_mut() {
                    action(&mut app.club_stations, data, value);
                }
            }
            if let Some(window) = weak.upgrade() {
                state.borrow().show(&window);
            }
        }
    }
    let page = window.global::<ClubStations>();
    page.on_select_park(on_click(window, state, |c, i: i32| c.select_park(i as usize)));
    page.on_select_filter(on_click(window, state, |c, i: i32| c.select_filter(i as usize)));
    page.on_select_row(on_click(window, state, |c, serial: i32| c.select_row(serial as u32)));
    page.on_search(on_click(window, state, |c, text: slint::SharedString| c.set_search(&text)));
    let close = on_click(window, state, |c, _: ()| c.close_detail());
    page.on_close_detail(move || close(()));
    let cancel = on_click(window, state, |c, _: ()| c.cancel_confirm());
    page.on_cancel_confirm(move || cancel(()));
    page.on_open_station(on_data_click(window, state, |c, data, serial: i32| c.open_station(data, serial as u32)));
    page.on_act(on_data_click(window, state, |c, data, id: i32| c.act(data, id)));
    let confirm = on_data_click(window, state, |c, data, _: ()| c.confirm(data));
    page.on_confirm(move || confirm(()));
    let battery = on_data_click(window, state, |c, data, _: ()| c.change_battery(data));
    page.on_change_battery(move || battery(()));
}

fn connect_runner_base(window: &App, state: &Rc<RefCell<AppState>>) {
    fn on_click<T: 'static>(
        window: &App,
        state: &Rc<RefCell<AppState>>,
        action: impl Fn(&mut runner_base::RunnerBaseState, T) + 'static,
    ) -> impl Fn(T) + 'static {
        let (weak, state) = (window.as_weak(), state.clone());
        move |value| {
            action(&mut state.borrow_mut().runner_base, value);
            if let Some(window) = weak.upgrade() {
                state.borrow().show(&window);
            }
        }
    }
    let page = window.global::<RunnerBase>();
    page.on_select_filter(on_click(window, state, |r, i: i32| r.select_filter(i as usize)));
    page.on_select_row(on_click(window, state, |r, id: i32| r.select_row(id as u32)));
    page.on_open_runner(on_click(window, state, |r, id: i32| r.open_runner(id as u32)));
    page.on_sort_by(on_click(window, state, |r, column: i32| r.sort_by(column as usize)));
    page.on_search(on_click(window, state, |r, text: slint::SharedString| r.set_search(&text)));
    let close = on_click(window, state, |r, _: ()| r.close_detail());
    page.on_close_detail(move || close(()));
    let ask = on_click(window, state, |r, _: ()| r.ask_delete());
    page.on_ask_delete(move || ask(()));
    let cancel = on_click(window, state, |r, _: ()| r.cancel_delete());
    page.on_cancel_delete(move || cancel(()));
    let (weak, state_for_delete) = (window.as_weak(), state.clone());
    page.on_confirm_delete(move || {
        {
            let app = &mut *state_for_delete.borrow_mut();
            if let Some(data) = app.data.as_mut() {
                app.runner_base.confirm_delete(data);
            }
        }
        if let Some(window) = weak.upgrade() {
            state_for_delete.borrow().show(&window);
        }
    });
}

fn connect_readers(window: &App, state: &Rc<RefCell<AppState>>) {
    // Changes a reader of the test data, then refreshes the screen.
    fn on_reader(
        window: &App,
        state: &Rc<RefCell<AppState>>,
        action: impl Fn(&mut Generated, usize) + 'static,
    ) -> impl Fn(i32) + 'static {
        let (weak, state) = (window.as_weak(), state.clone());
        move |reader| {
            if let Some(data) = state.borrow_mut().data.as_mut() {
                action(data, reader as usize);
            }
            if let Some(window) = weak.upgrade() {
                state.borrow().show(&window);
            }
        }
    }
    window.global::<Readers>().on_toggle_pin(on_reader(window, state, Generated::toggle_pin));
    let unplug = on_reader(window, state, Generated::toggle_unplugged);
    window.global::<Demo>().on_toggle_unplugged(move || unplug(0));
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
        // Clicking the active choice must not draw new data over the current ones.
        if window.global::<Demo>().get_active_test_data() == i {
            return;
        }
        window.global::<Demo>().set_active_test_data(i);
        if i == 1 {
            state_for_switch.borrow_mut().start_test_data(random_seed());
            let (weak, state) = (window.as_weak(), state_for_switch.clone());
            // The race clock goes on second by second; cards come now and then.
            readings_for_switch.start(TimerMode::Repeated, Duration::from_secs(1), move || {
                let mut state = state.borrow_mut();
                if let Some(data) = state.data.as_mut() {
                    data.tick();
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
    connect_participants(&window, &state);
    connect_starts(&window, &state);
    connect_stations(&window, &state);
    connect_live(&window, &state);
    connect_results(&window, &state);
    connect_cards(&window, &state);
    connect_club_stations(&window, &state);
    connect_runner_base(&window, &state);
    connect_readers(&window, &state);
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
