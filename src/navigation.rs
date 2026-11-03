// The navigation state, as in the HTML prototype: the home page leads to an event
// or to a club page, and every other view goes back to it.

use crate::test_data::Generated;
use crate::{App, Navigation};
use slint::ComponentHandle;

pub const HOME: &str = "Accueil";
pub const CLUB_PAGES: [&str; 3] = ["Puces de prêt", "Boîtiers", "Coureurs et clubs"];
pub const TABS: [&str; 6] = ["Circuits", "Participants", "Départs", "Boîtiers", "Direct", "Résultats"];

const APP_NAME: &str = "Chron'O";
const DIRECT_TAB: usize = 4;

#[derive(Default)]
enum View {
    #[default]
    Home,
    ClubPage(usize),
    Event { event: usize, tab: usize },
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

    pub fn open_event(&mut self, event: usize) {
        self.view = View::Event { event, tab: 0 };
    }

    pub fn open_direct(&mut self, event: usize) {
        self.view = View::Event { event, tab: DIRECT_TAB };
    }

    pub fn select_tab(&mut self, tab: usize) {
        if let View::Event { event, .. } = self.view {
            self.view = View::Event { event, tab };
        }
    }

    // Events only exist with test data: when it stops, leave the event that went away.
    pub fn leave_event(&mut self) {
        if matches!(self.view, View::Event { .. }) {
            self.view = View::Home;
        }
    }

    pub fn show(&self, window: &App, data: Option<&Generated>) {
        let navigation = window.global::<Navigation>();
        navigation.set_home(matches!(self.view, View::Home));
        navigation.set_in_event(matches!(self.view, View::Event { .. }));

        let (header, page, tab) = match self.view {
            View::Home => (APP_NAME, HOME, 0),
            View::ClubPage(page) => (APP_NAME, CLUB_PAGES[page], 0),
            View::Event { event, tab } => {
                let name = data.map_or(APP_NAME, |d| d.events[event].name.as_str());
                (name, TABS[tab], tab)
            }
        };
        navigation.set_header_title(header.into());
        navigation.set_page_title(page.into());
        navigation.set_active_tab(tab as i32);
    }
}
