// The navigation state, as in the HTML prototype: the home page leads to an event
// or to a club page, and every other view goes back to it.

use crate::test_data::{CURRENT_EVENT, Generated};
use crate::{App, Navigation};
use slint::ComponentHandle;

pub const HOME: &str = "Accueil";
pub const CLUB_PAGES: [&str; 3] = ["Puces de prêt", "Boîtiers", "Coureurs et clubs"];
pub const TABS: [&str; 5] = ["Circuits", "Participants", "Départs", "Boîtiers", "Résultats"];

const PARTICIPANTS_TAB: usize = 1;
const APP_NAME: &str = "Chron'O";
const CARDS_PAGE: usize = 0;
const STATIONS_PAGE: usize = 1;
const RUNNERS_PAGE: usize = 2;

#[derive(Default)]
enum View {
    #[default]
    Home,
    ClubPage(usize),
    // The Direct is not a tab: a drawer over any tab, open or closed.
    Event { event: usize, tab: usize, live_open: bool },
}

#[derive(Default)]
pub struct Navigator {
    view: View,
}

impl Navigator {
    pub fn go_home(&mut self) {
        self.view = View::Home;
    }

    pub fn open_club_page(&mut self, page: usize) {
        self.view = View::ClubPage(page);
    }

    // An event opens on its Participants (prototype: openEv).
    pub fn open_event(&mut self, event: usize) {
        self.view = View::Event { event, tab: PARTICIPANTS_TAB, live_open: false };
    }

    pub fn open_direct(&mut self, event: usize) {
        self.view = View::Event { event, tab: PARTICIPANTS_TAB, live_open: true };
    }

    // Choosing a tab shows it: the Direct goes down.
    pub fn select_tab(&mut self, tab: usize) {
        if let View::Event { event, .. } = self.view {
            self.view = View::Event { event, tab, live_open: false };
        }
    }

    pub fn toggle_live(&mut self) {
        if let View::Event { live_open, .. } = &mut self.view {
            *live_open = !*live_open;
        }
    }

    // Events only exist with test data: when it stops, leave the event that went away.
    pub fn leave_event(&mut self) {
        if matches!(self.view, View::Event { .. }) {
            self.view = View::Home;
        }
    }

    pub fn on_stations_page(&self) -> bool {
        matches!(self.view, View::ClubPage(STATIONS_PAGE))
    }

    pub fn event(&self) -> Option<usize> {
        match self.view {
            View::Event { event, .. } => Some(event),
            _ => None,
        }
    }

    pub fn show(&self, window: &App, data: Option<&Generated>) {
        let navigation = window.global::<Navigation>();
        navigation.set_home(matches!(self.view, View::Home));
        navigation.set_in_event(matches!(self.view, View::Event { .. }));
        navigation.set_cards_page(matches!(self.view, View::ClubPage(CARDS_PAGE)));
        navigation.set_stations_page(self.on_stations_page());
        navigation.set_runners_page(matches!(self.view, View::ClubPage(RUNNERS_PAGE)));

        let (header, tab, live_open) = match self.view {
            View::Home => (APP_NAME, 0, false),
            View::ClubPage(page) => (CLUB_PAGES[page], 0, false),
            View::Event { event, tab, live_open } => {
                let name = data.map_or(APP_NAME, |d| d.events[event].name.as_str());
                (name, tab, live_open)
            }
        };
        navigation.set_header_title(header.into());
        // An event says when and where, above its name (prototype: header).
        let detail = match (self.event(), data) {
            (Some(event), Some(data)) => data.events[event].detail(),
            _ => String::new(),
        };
        navigation.set_header_detail(detail.into());
        navigation.set_active_tab(tab as i32);
        navigation.set_live_open(live_open);
        // The dot of the Direct only moves while the race is on: today's event.
        navigation.set_live_running(self.event() == Some(CURRENT_EVENT));
    }
}
