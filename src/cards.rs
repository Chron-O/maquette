// The Puces de prêt page: the club's loan cards, park by park. Only the states and the
// returns change something, on the test data.

use crate::home::badge;
use crate::test_data::{CardPlace, CardState, Generated, Kind, Label, LoanCard};
use crate::{ActiveFilter, App, FilterChoice, FilterGroup, CardAction, CardEntry, CardParkItem, CardRow, Cards, LiveFilter, OverdueCard, model};
use slint::{ComponentHandle, SharedString};

// The states, then where a card is out (puces-pret-parc.md § 12): the same words as the band.
const FILTERS: [&str; 7] = ["Toutes", "En service", "À vérifier", "HS", "Perdues", "Prêtées", "Non rendues"];

// What the buttons of the detail do.
const ACT_IN_SERVICE: i32 = 0;
const ACT_TO_CHECK: i32 = 1;
const ACT_BROKEN: i32 = 2;
const ACT_LOST: i32 = 3;
const ACT_RETURN: i32 = 4;
const ACT_LEND: i32 = 5;    // shown, without effect for now

#[derive(Default)]
pub struct CardsState {
    park: usize,
    filter: usize,
    search: String,
    selected: Option<u32>,
    to_confirm: Option<CardState>,
    last_read: Option<u32>,
    unknown_read: Option<u32>,   // a card in none of the parks: the tile asks what to do
    reads: u32,
}

fn state_label(state: CardState) -> Label {
    match state {
        CardState::InService => Label::new("En service", Kind::Valid),
        CardState::ToCheck => Label::new("À vérifier", Kind::ToCheck),
        CardState::Broken => Label::new("HS", Kind::Pm),
        CardState::Lost => Label::new("Perdue", Kind::Inactive),
    }
}

// The place, and who has the card since when.
fn place_text(place: &CardPlace) -> (&'static str, String) {
    match place {
        CardPlace::Park => ("Au parc", String::new()),
        CardPlace::Lent { who, event, date } => ("Prêtée", format!("{who} · {event} · {date}")),
        CardPlace::NotReturned { who, event, since } => ("Non rendue", format!("{who} · {event} · depuis le {since}")),
    }
}

fn is_out(card: &LoanCard) -> bool {
    card.place != CardPlace::Park
}

fn matches_filter(card: &LoanCard, filter: usize) -> bool {
    match filter {
        1 => card.state == CardState::InService,
        2 => card.state == CardState::ToCheck,
        3 => card.state == CardState::Broken,
        4 => card.state == CardState::Lost,
        5 => matches!(card.place, CardPlace::Lent { .. }),
        6 => matches!(card.place, CardPlace::NotReturned { .. }),
        _ => true,
    }
}

fn actions_of(card: &LoanCard) -> Vec<CardAction> {
    let action = |id: i32, label: &str, danger: bool| CardAction { id, label: label.into(), danger };
    let mut actions = Vec::new();
    if card.place == CardPlace::Park && matches!(card.state, CardState::InService | CardState::ToCheck) {
        actions.push(action(ACT_LEND, "Prêter…", false));
    }
    if is_out(card) && card.state != CardState::Lost {
        actions.push(action(ACT_RETURN, "Rendue", false));
    }
    match card.state {
        CardState::InService => actions.push(action(ACT_TO_CHECK, "À vérifier", false)),
        CardState::ToCheck => actions.push(action(ACT_IN_SERVICE, "En service", false)),
        CardState::Broken | CardState::Lost => actions.push(action(ACT_IN_SERVICE, "Remise en service", false)),
    }
    if matches!(card.state, CardState::InService | CardState::ToCheck) {
        actions.push(action(ACT_BROKEN, "HS…", true));
    }
    if card.state != CardState::Lost {
        actions.push(action(ACT_LOST, "Perdue…", true));
    }
    actions
}

impl CardsState {
    pub fn select_park(&mut self, park: usize) {
        self.park = park;
        self.filter = 0;
        self.close_detail();
    }

    pub fn select_filter(&mut self, filter: usize) {
        self.filter = filter;
    }

    // The Filtres bubble (brief zones-onglets § 5): one group, the choices after « Tous ».
    pub fn toggle_filter(&mut self, _group: usize, choice: usize) {
        self.filter = if self.filter == choice + 1 { 0 } else { choice + 1 };
    }

    pub fn clear_filters(&mut self) {
        self.filter = 0;
    }

    pub fn set_search(&mut self, text: &str) {
        self.search = text.trim().to_lowercase();
    }

    pub fn select_row(&mut self, number: u32) {
        self.selected = if self.selected == Some(number) { None } else { Some(number) };
        self.to_confirm = None;
    }

    pub fn close_detail(&mut self) {
        self.selected = None;
        self.to_confirm = None;
    }

    pub fn cancel_confirm(&mut self) {
        self.to_confirm = None;
    }

    // From the alert: the card may be in another park than the one shown.
    pub fn open_card(&mut self, data: &Generated, number: u32) {
        if let Some(park) = data.card_parks.iter().position(|p| p.cards.iter().any(|c| c.number == number)) {
            self.park = park;
        }
        self.filter = 0;
        self.search.clear();
        self.selected = Some(number);
        self.to_confirm = None;
    }

    // A card put on the reader: its detail opens and its row is selected (encart-lecture.md § 5).
    // Every other read, the Démo panel puts an unknown card: the tile asks its question
    // (encart-lecture.md § 5, § 9), and nothing else changes.
    pub fn read(&mut self, data: &mut Generated) {
        self.reads += 1;
        if self.reads.is_multiple_of(2) {
            self.unknown_read = Some(data.read_unknown_card());
            return;
        }
        if let Some(number) = data.read_loan_card() {
            self.open_card(data, number);
            self.last_read = Some(number);
            self.unknown_read = None;
        }
    }

    pub fn act(&mut self, data: &mut Generated, id: i32) {
        let Some(number) = self.selected else { return };
        match id {
            ACT_IN_SERVICE => data.set_card_state(number, CardState::InService, "Remise en service"),
            ACT_TO_CHECK => data.set_card_state(number, CardState::ToCheck, "Marquée à vérifier"),
            ACT_BROKEN => self.to_confirm = Some(CardState::Broken),
            ACT_LOST => self.to_confirm = Some(CardState::Lost),
            ACT_RETURN => data.return_card(number),
            _ => {}
        }
    }

    pub fn confirm(&mut self, data: &mut Generated) {
        let (Some(number), Some(state)) = (self.selected, self.to_confirm.take()) else { return };
        let text = if state == CardState::Broken { "Marquée HS" } else { "Marquée perdue" };
        data.set_card_state(number, state, text);
    }

    pub fn show(&self, window: &App, data: Option<&Generated>) {
        let page = window.global::<Cards>();
        let parks = data.map_or(&[][..], |d| &d.card_parks[..]);

        page.set_parks(model(
            parks.iter().enumerate()
                .map(|(i, p)| CardParkItem { index: i as i32, name: p.name.into(), count: p.cards.len().to_string().into() })
                .collect(),
        ));
        page.set_park_chips(model(
            parks.iter().map(|p| LiveFilter { label: p.name.into(), tone: -1, detail: SharedString::new() }).collect(),
        ));
        page.set_active_park(self.park as i32);
        page.set_active_filter(self.filter as i32);
        page.set_filter_groups(model(vec![FilterGroup {
            key: 0,
            title: "État".into(),
            choices: model(
                FILTERS
                    .iter()
                    .enumerate()
                    .skip(1)
                    .map(|(i, label)| FilterChoice { label: (*label).into(), tone: -1, active: self.filter == i })
                    .collect(),
            ),
        }]));
        page.set_active_filters(model(
            FILTERS
                .get(self.filter)
                .filter(|_| self.filter > 0)
                .map(|label| ActiveFilter { label: (*label).into(), tone: -1, group: 0, choice: self.filter as i32 - 1 })
                .into_iter()
                .collect(),
        ));
        page.set_search_text(self.search.as_str().into());

        let park = parks.get(self.park);
        let cards = park.map_or(&[][..], |p| &p.cards[..]);
        // Each filter says how many cards of the park shown it keeps, as on the club Boîtiers page.
        page.set_filters(model(
            FILTERS.iter()
                .enumerate()
                .map(|(i, f)| {
                    let count = cards.iter().filter(|c| matches_filter(c, i)).count();
                    LiveFilter { label: format!("{f} · {count}").into(), tone: -1, detail: SharedString::new() }
                })
                .collect(),
        ));
        page.set_title(park.map_or("", |p| p.name).into());
        page.set_description(park.map_or("", |p| p.description).into());
        page.set_chips(model(park.map(|p| SharedString::from(p.description)).into_iter().collect()));

        let count = |wanted: fn(&LoanCard) -> bool| cards.iter().filter(|c| wanted(c)).count();
        let total = plural(cards.len(), "puce");
        page.set_state(total.as_str().into());
        let lent = count(|c| matches!(c.place, CardPlace::Lent { .. }));
        let not_returned = count(|c| matches!(c.place, CardPlace::NotReturned { .. }));
        let at_park = cards.len() - lent - not_returned;
        page.set_summary(
            format!("{total} · {at_park} au parc · {} · {}", plural(lent, "prêtée"), plural(not_returned, "non rendue")).into(),
        );
        page.set_summary_short(format!("{total} · {} · {}", plural(lent, "prêtée"), plural(not_returned, "non rendue")).into());

        let words: Vec<&str> = self.search.split_whitespace().collect();
        let rows: Vec<CardRow> = cards
            .iter()
            .filter(|c| matches_filter(c, self.filter))
            .filter(|c| {
                let (place, detail) = place_text(&c.place);
                let text = format!("{} {} {place} {detail} {} {}", c.number, c.kind, c.last_borrower.as_deref().unwrap_or(""), c.note).to_lowercase();
                words.iter().all(|word| text.contains(word))
            })
            .map(|c| {
                let (place, detail) = place_text(&c.place);
                CardRow {
                    number: c.number as i32,
                    kind: c.kind.into(),
                    badge: badge(&state_label(c.state)),
                    place: place.into(),
                    place_detail: detail.into(),
                    battery: c.battery.unwrap_or("—").into(),
                    borrower: c.last_borrower.as_deref().unwrap_or("—").into(),
                    loans: c.loans.to_string().into(),
                    note: c.note.into(),
                    to_handle: matches!(c.place, CardPlace::NotReturned { .. }) && c.state != CardState::Lost,
                    selected: self.selected == Some(c.number),
                }
            })
            .collect();
        page.set_empty(
            if !rows.is_empty() {
                SharedString::new()
            } else if self.filter == 0 && words.is_empty() {
                "Aucune puce".into()
            } else {
                "Aucun résultat".into()
            },
        );
        page.set_rows(model(rows));

        let overdue = data.map_or(Vec::new(), Generated::overdue_cards);
        page.set_alert_title(if overdue.is_empty() { SharedString::new() } else { plural(overdue.len(), "puce non rendue").replace("puce non rendues", "puces non rendues").into() });
        page.set_alert_count(overdue.len().to_string().into());
        page.set_overdue(model(
            overdue.iter()
                .map(|c| OverdueCard {
                    number: c.number as i32,
                    detail: place_text(&c.place).1.into(),
                })
                .collect(),
        ));

        self.show_reading(&page, data);
        self.show_detail(&page, data);
    }

    // Three lines (encart-lecture.md § 8): the number, the card, its state and where it is.
    fn show_reading(&self, page: &Cards, data: Option<&Generated>) {
        page.set_read_unknown(self.unknown_read.is_some());
        if let Some(number) = self.unknown_read {
            page.set_has_reading(true);
            page.set_read_code(number.to_string().into());
            page.set_read_identity("Dans aucun parc du club".into());
            page.set_read_verdict("Inconnue".into());
            return;
        }
        let found = self.last_read.and_then(|number| {
            let data = data?;
            let card = data.card(number)?;
            let park = data.card_parks.iter().find(|p| p.cards.iter().any(|c| c.number == number))?;
            Some((card, park.name))
        });
        page.set_has_reading(found.is_some());
        let Some((card, park)) = found else { return };
        page.set_read_code(card.number.to_string().into());
        page.set_read_identity(format!("{} · {park}", card.kind).into());
        page.set_read_verdict(format!("{} · {}", state_label(card.state).word, place_text(&card.place).0).into());
    }

    fn show_detail(&self, page: &Cards, data: Option<&Generated>) {
        let found = self.selected.and_then(|number| {
            let data = data?;
            let card = data.card(number)?;
            let park = data.card_parks.iter().find(|p| p.cards.iter().any(|c| c.number == number))?;
            Some((card, park.name))
        });
        page.set_detail_open(found.is_some());
        let Some((card, park)) = found else { return };

        let (place, detail) = place_text(&card.place);
        page.set_detail_number(card.number.to_string().into());
        page.set_detail_subtitle(format!("{} · {park}", card.kind).into());
        page.set_detail_kind(card.kind.into());
        page.set_detail_park(park.into());
        page.set_detail_badge(badge(&state_label(card.state)));
        page.set_detail_since(format!("depuis le {}", card.state_since).into());
        // A lost card is not somewhere any more: this is the last place known of it.
        page.set_detail_place_label(if card.state == CardState::Lost { "Dernier lieu connu" } else { "Emplacement" }.into());
        page.set_detail_place(place.into());
        page.set_detail_place_detail(detail.into());
        page.set_detail_has_battery(card.battery.is_some());
        page.set_detail_battery(card.battery.unwrap_or("").into());
        page.set_detail_note(if card.note.is_empty() { "Aucune note" } else { card.note }.into());
        page.set_detail_loans(plural(card.loans as usize, "prêt").into());
        page.set_actions(model(actions_of(card)));
        page.set_history(model(
            card.history.iter().map(|e| CardEntry { date: e.date.as_str().into(), text: e.text.as_str().into() }).collect(),
        ));
        let (question, button) = match self.to_confirm {
            Some(CardState::Broken) => ("Marquer cette puce HS ?", "Marquer HS"),
            Some(CardState::Lost) => ("Marquer cette puce perdue ?", "Marquer perdue"),
            _ => ("", ""),
        };
        page.set_confirm_question(question.into());
        page.set_confirm_button(button.into());
    }
}

pub fn plural(count: usize, noun: &str) -> String {
    format!("{count} {noun}{}", if count > 1 { "s" } else { "" })
}
