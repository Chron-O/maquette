// The Coureurs et clubs page: the club's local base of runners. Only the sorting, the search,
// the filters and the deletion of a record work, on the test data.

use crate::cards::plural;
use crate::test_data::{BaseRunner, CardPlace, Generated, LicenceKind};
use crate::{ActiveFilter, App, FilterChoice, FilterGroup, BaseIssue, BaseRunnerRow, LiveFilter, RunnerBase, model, ClubLine, PanelSet, StatBlock};
use slint::{ComponentHandle, Model, ModelRc, SharedString, VecModel};
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;

const FILTERS: [&str; 8] = ["Tous", "Licenciés", "Non licenciés", "Compétition", "Découverte", "Loisir", "Handisport", "Doublons à vérifier"];
const DUPLICATES: usize = 7;
// The filters of the clubs (Robin, 12/11).
const CLUB_FILTERS: [&str; 2] = ["Clubs à fusionner", "Sans licencié"];

// One wording for a record typed by hand, in the table and in the detail.
const NO_LICENCE: &str = "Pas de licence";

const LEAGUES: [(&str, &str); 5] = [
    ("AR", "Auvergne-Rhône-Alpes"),
    ("BR", "Bretagne"),
    ("HF", "Hauts-de-France"),
    ("OC", "Occitanie"),
    ("PL", "Pays de la Loire"),
];

#[derive(Default)]
pub struct RunnerBaseState {
    clubs_tab: bool,              // the panel: Coureurs or Clubs (Robin, 12/11)
    origin: Option<usize>,        // Coureurs › Origine: 0 the FFCO archive, 1 typed by hand
    league: Option<String>,       // Clubs › Ligues
    club: Option<String>,         // Filtres › Club, from « Voir ses coureurs ›»
    club_filter: Option<usize>,   // Filtres of the clubs, one of CLUB_FILTERS
    filter: usize,
    search: String,
    sort_column: usize,
    descending: bool,
    selected: Option<u32>,
    confirming: bool,
    // Kept from one refresh to the next: only the rows that changed are rebuilt, so that
    // a reading elsewhere neither costs hundreds of rows nor moves the list.
    rows: Rc<VecModel<BaseRunnerRow>>,
}

// The leagues of the clubs in the base, with how many clubs each.
fn leagues_of(base: &[BaseRunner]) -> Vec<(&'static str, usize)> {
    let clubs: BTreeMap<&str, &str> = base.iter().filter(|r| !r.club_code.is_empty()).map(|r| (r.club, league(r))).collect();
    let mut counts: BTreeMap<&'static str, usize> = BTreeMap::new();
    for code in clubs.values() {
        let code: &'static str = LEAGUES.iter().find(|(short, _)| short == code).map_or("", |(short, _)| short);
        if !code.is_empty() {
            *counts.entry(code).or_default() += 1;
        }
    }
    counts.into_iter().collect()
}

fn league(runner: &BaseRunner) -> &'static str {
    runner.club_code.get(runner.club_code.len().saturating_sub(2)..).unwrap_or("")
}

fn league_name(code: &str) -> &str {
    LEAGUES.iter().find(|(short, _)| *short == code).map_or(code, |(_, name)| *name)
}

fn kind_word(kind: Option<LicenceKind>) -> &'static str {
    match kind {
        Some(LicenceKind::Competition) => "Compétition",
        Some(LicenceKind::Discovery) => "Découverte",
        Some(LicenceKind::Leisure) => "Loisir",
        Some(LicenceKind::Handisport) => "Handisport",
        None => "",
    }
}

fn kind_full(kind: Option<LicenceKind>) -> &'static str {
    match kind {
        Some(LicenceKind::Discovery) => "Découverte Compétition",
        Some(LicenceKind::Leisure) => "Loisir Santé",
        other => kind_word(other),
    }
}

// For now, two records look alike when one typed by hand has the name of a licensed one; two
// clubs when their names are the same letters. The real rule is a spec to write (ECRANS.md).
fn letters(text: &str) -> String {
    text.chars().filter(|c| c.is_alphanumeric()).collect::<String>().to_lowercase()
}

fn is_duplicate(runner: &BaseRunner, base: &[BaseRunner]) -> bool {
    base.iter().any(|o| o.id != runner.id && o.family == runner.family && o.first == runner.first && (o.licence.is_some() != runner.licence.is_some()))
}

fn twin_club<'a>(club: &str, clubs: &[&'a str]) -> Option<&'a str> {
    clubs.iter().copied().find(|o| *o != club && letters(o) == letters(club))
}

fn matches_filter(runner: &BaseRunner, filter: usize) -> bool {
    match filter {
        1 => runner.licence.is_some(),
        2 => runner.licence.is_none(),
        3 => runner.licence_kind == Some(LicenceKind::Competition),
        4 => runner.licence_kind == Some(LicenceKind::Discovery),
        5 => runner.licence_kind == Some(LicenceKind::Leisure),
        6 => runner.licence_kind == Some(LicenceKind::Handisport),
        _ => true,
    }
}

// What a column sorts on, numbers kept in their order. An empty cell has no key.
fn sort_key(runner: &BaseRunner, column: usize) -> Option<String> {
    let text = match column {
        0 => runner.family.to_string(),
        1 => runner.first.to_string(),
        2 => runner.club.to_string(),
        3 => league(runner).to_string(),
        4 => runner.category.clone(),
        5 => runner.year.to_string(),
        6 => format!("{:08}", runner.licence?),
        7 => kind_word(runner.licence_kind).to_string(),
        8 => format!("{:08}", runner.card?),
        _ => format!("{:06}", runner.points.as_ref()?.0),
    };
    (!text.is_empty()).then_some(text)
}

// What deserves a look, in two groups (coureurs-clubs.md § 7): a record typed by hand with the
// name of a licensed runner, and a card on two records. Nothing is changed by these lists.
fn issues_of(base: &[BaseRunner]) -> (Vec<BaseIssue>, Vec<BaseIssue>) {
    let mut duplicates = Vec::new();
    for runner in base.iter().filter(|r| r.licence.is_none()) {
        if base.iter().any(|o| o.licence.is_some() && o.family == runner.family && o.first == runner.first) {
            duplicates.push(BaseIssue {
                id: runner.id as i32,
                label: runner.name().into(),
                detail: "fiche saisie ↔ licencié".into(),
            });
        }
    }
    let mut shared = Vec::new();
    let mut cards: BTreeMap<u32, Vec<&BaseRunner>> = BTreeMap::new();
    for runner in base {
        if let Some(card) = runner.card {
            cards.entry(card).or_default().push(runner);
        }
    }
    for (card, holders) in cards.iter().filter(|(_, holders)| holders.len() > 1) {
        let names: Vec<String> = holders.iter().map(|h| h.name()).collect();
        shared.push(BaseIssue {
            id: holders[0].id as i32,
            label: format!("Puce {card}").into(),
            detail: names.join(" · ").into(),
        });
    }
    (duplicates, shared)
}

impl RunnerBaseState {
    pub fn select_filter(&mut self, filter: usize) {
        self.filter = filter;
    }

    // The panel (Robin, 12/11): Coureurs or Clubs, then a choice in its list (-1: all).
    pub fn select_tab(&mut self, clubs: bool) {
        self.clubs_tab = clubs;
        self.selected = None;
    }

    pub fn select_set(&mut self, set: i32, data: Option<&Generated>) {
        if self.clubs_tab {
            self.league = usize::try_from(set).ok().and_then(|i| data.and_then(|d| leagues_of(&d.runner_base).get(i).map(|l| l.0.to_string())));
        } else {
            self.origin = usize::try_from(set).ok();
        }
    }

    // « Voir ses coureurs ›» on a club: all the runners, with that club as a filter.
    pub fn show_club(&mut self, name: &str) {
        self.clubs_tab = false;
        self.origin = None;
        self.club = Some(name.to_string());
    }

    // The Filtres bubble: group 0 the runners (choices after « Tous »), 1 the club, 2 the clubs.
    pub fn toggle_filter(&mut self, group: usize, choice: usize) {
        match group {
            1 => self.club = None,
            2 => self.club_filter = if self.club_filter == Some(choice) { None } else { Some(choice) },
            _ => self.filter = if self.filter == choice + 1 { 0 } else { choice + 1 },
        }
    }

    // A block of figures: its filter alone; again, it goes away.
    pub fn pick_stat(&mut self, group: usize, choice: usize) {
        if group == 2 {
            self.club_filter = if self.club_filter == Some(choice) { None } else { Some(choice) };
        } else {
            self.club = None;
            self.filter = if self.filter == choice + 1 { 0 } else { choice + 1 };
        }
    }

    pub fn clear_filters(&mut self) {
        self.filter = 0;
        self.club = None;
        self.club_filter = None;
    }

    pub fn set_search(&mut self, text: &str) {
        self.search = text.trim().to_lowercase();
    }

    pub fn sort_by(&mut self, column: usize) {
        if self.sort_column == column {
            self.descending = !self.descending;
        } else {
            self.sort_column = column;
            self.descending = false;
        }
    }

    pub fn select_row(&mut self, id: u32) {
        self.selected = if self.selected == Some(id) { None } else { Some(id) };
        self.confirming = false;
    }

    // From the alert: the row may be hidden by what is on screen.
    pub fn open_runner(&mut self, id: u32) {
        self.filter = 0;
        self.search.clear();
        self.selected = Some(id);
        self.confirming = false;
    }

    pub fn close_detail(&mut self) {
        self.selected = None;
        self.confirming = false;
    }

    pub fn ask_delete(&mut self) {
        self.confirming = true;
    }

    pub fn cancel_delete(&mut self) {
        self.confirming = false;
    }

    pub fn confirm_delete(&mut self, data: &mut Generated) {
        if let Some(id) = self.selected {
            data.runner_base.retain(|r| r.id != id);
        }
        self.close_detail();
    }

    pub fn show(&self, window: &App, data: Option<&Generated>) {
        let page = window.global::<RunnerBase>();
        let base = data.map_or(&[][..], |d| &d.runner_base[..]);

        page.set_filters(model(
            FILTERS.iter().map(|f| LiveFilter { label: (*f).into(), tone: -1, detail: SharedString::new() }).collect(),
        ));
        page.set_active_filter(self.filter as i32);
        page.set_clubs_tab(self.clubs_tab);
        let club_names: Vec<&str> = base.iter().map(|r| r.club).collect::<BTreeSet<_>>().into_iter().collect();
        if self.clubs_tab {
            page.set_filter_groups(model(vec![FilterGroup {
                search: SharedString::new(),
                searchable: false,
                key: 2,
                title: "Clubs".into(),
                choices: model(CLUB_FILTERS.iter().enumerate().map(|(i, l)| FilterChoice { label: (*l).into(), tone: -1, active: self.club_filter == Some(i) }).collect()),
            }]));
            page.set_active_filters(model(
                self.club_filter.map(|c| ActiveFilter { label: CLUB_FILTERS[c].into(), tone: -1, group: 2, choice: c as i32 }).into_iter().collect(),
            ));
        } else {
            page.set_filter_groups(model(vec![FilterGroup {
                search: SharedString::new(),
                searchable: false,
                key: 0,
                title: "Coureurs".into(),
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
                    .chain(self.club.as_ref().map(|c| ActiveFilter { label: format!("Club : {c}").into(), tone: -1, group: 1, choice: 0 }))
                    .collect(),
            ));
        }
        self.show_panel(&page, base, &club_names);
        self.show_clubs(&page, base, &club_names);
        page.set_search_text(self.search.as_str().into());
        page.set_sort_column(self.sort_column as i32);
        page.set_sort_ascending(!self.descending);

        let licensed = base.iter().filter(|r| r.licence.is_some()).count();
        let hand = base.len() - licensed;
        let clubs = base.iter().map(|r| r.club).collect::<BTreeSet<_>>().len();
        // The warning of coureurs-clubs.md § 5 opens the band, then the counts.
        let licensed_text = format!("{licensed} licencié{}", if licensed > 1 { "s" } else { "" });
        let warning = match data {
            Some(d) => format!("Base locale, pas forcément à jour · archive FFCO importée le {} ({licensed_text})", d.archive_date_numeric),
            None => "Base locale, pas forcément à jour · archive FFCO jamais importée".to_string(),
        };
        if self.clubs_tab {
            page.set_title("Clubs".into());
            page.set_state(plural(clubs, "club").into());
        } else {
            page.set_title("Base du club".into());
            page.set_state(plural(base.len(), "fiche").into());
        }
        page.set_summary(
            format!("{warning} · {} · {} saisie{} à la main", plural(clubs, "club"), plural(hand, "fiche"), if hand > 1 { "s" } else { "" })
                .into(),
        );
        page.set_summary_short(format!("{warning} · {}", plural(clubs, "club")).into());
        let warning_short = "base locale, pas forcément à jour";

        let words: Vec<&str> = self.search.split_whitespace().collect();
        let mut shown: Vec<&BaseRunner> = base
            .iter()
            .filter(|r| if self.filter == DUPLICATES { is_duplicate(r, base) } else { matches_filter(r, self.filter) })
            .filter(|r| self.origin.is_none_or(|o| (o == 0) == r.licence.is_some()))
            .filter(|r| self.club.as_deref().is_none_or(|c| r.club == c))
            .filter(|r| {
                let text = format!(
                    "{} {} {} {} {} {} {}",
                    r.name(),
                    r.club,
                    r.club_code,
                    r.category,
                    r.licence.map_or(String::new(), |l| l.to_string()),
                    kind_word(r.licence_kind),
                    r.card.map_or(String::new(), |c| c.to_string()),
                )
                .to_lowercase();
                words.iter().all(|word| text.contains(word))
            })
            .collect();
        // Empty cells stay at the end, whichever way the column is sorted.
        shown.sort_by(|a, b| {
            let order = match (sort_key(a, self.sort_column), sort_key(b, self.sort_column)) {
                (Some(x), Some(y)) if self.descending => y.cmp(&x),
                (Some(x), Some(y)) => x.cmp(&y),
                (Some(_), None) => Ordering::Less,
                (None, Some(_)) => Ordering::Greater,
                (None, None) => Ordering::Equal,
            };
            order.then_with(|| (a.family, a.first, a.id).cmp(&(b.family, b.first, b.id)))
        });
        // The band counts what is shown (Robin, 11/11).
        if shown.len() < base.len() || self.clubs_tab {
            let text = if self.clubs_tab {
                format!("{} · {}", plural(clubs, "club"), plural(base.len(), "fiche"))
            } else {
                format!("{} sur {} · {warning_short}", plural(shown.len(), "fiche"), base.len())
            };
            page.set_summary(text.as_str().into());
            page.set_summary_short(text.into());
        }
        page.set_empty(
            if !shown.is_empty() {
                SharedString::new()
            } else if self.filter == 0 && words.is_empty() {
                "Aucune fiche".into()
            } else {
                "Aucun résultat".into()
            },
        );

        let rows: Vec<BaseRunnerRow> = shown
            .iter()
            .map(|r| BaseRunnerRow {
                id: r.id as i32,
                family: r.family.into(),
                first: r.first.into(),
                club: r.club.into(),
                league: league(r).into(),
                category: r.category.as_str().into(),
                year: r.year.to_string().into(),
                licence: r.licence.map_or("—".into(), |l| l.to_string().into()),
                kind: if r.licence.is_some() { kind_word(r.licence_kind).into() } else { NO_LICENCE.into() },
                card: r.card.map_or("—".into(), |c| c.to_string().into()),
                points: r.points.as_ref().map_or("—".into(), |(points, _)| points.to_string().into()),
                duplicate: is_duplicate(r, base),
                selected: self.selected == Some(r.id),
            })
            .collect();
        if self.rows.row_count() == rows.len() {
            for (i, row) in rows.into_iter().enumerate() {
                if self.rows.row_data(i).as_ref() != Some(&row) {
                    self.rows.set_row_data(i, row);
                }
            }
        } else {
            self.rows.set_vec(rows);
        }
        page.set_rows(ModelRc::from(self.rows.clone()));

        let (duplicates, shared) = issues_of(base);
        let count = duplicates.len() + shared.len();
        page.set_alert_title(match count {
            0 => SharedString::new(),
            1 => "1 point à vérifier".into(),
            n => format!("{n} points à vérifier").into(),
        });
        page.set_alert_count(count.to_string().into());
        page.set_duplicates_title(format!("Doublons possibles · {}", duplicates.len()).into());
        page.set_shared_cards_title(format!("Puces sur deux fiches · {}", shared.len()).into());
        page.set_duplicates(model(duplicates));
        page.set_shared_cards(model(shared));

        self.show_detail(&page, data);
    }

    // The panel (Robin, 12/11): the main choice and its sub-zone, then the blocks of figures.
    fn show_panel(&self, page: &RunnerBase, base: &[BaseRunner], clubs: &[&str]) {
        let set = |name: String, count: usize| PanelSet { name: name.into(), count: count.to_string().into(), detail: SharedString::new() };
        if self.clubs_tab {
            let leagues = leagues_of(base);
            page.set_main_set(set("Tous les clubs".into(), clubs.len()));
            page.set_sets_title("Ligues".into());
            page.set_sets(model(leagues.iter().map(|(code, n)| set(format!("{code} · {}", league_name(code)), *n)).collect()));
            page.set_active_set(self.league.as_deref().and_then(|l| leagues.iter().position(|(c, _)| *c == l)).map_or(-1, |i| i as i32));
            let without = clubs.iter().filter(|c| !base.iter().any(|r| r.club == **c && r.licence.is_some())).count();
            let twins = clubs.iter().filter(|c| twin_club(c, clubs).is_some()).count();
            page.set_stats(model(
                [(0usize, twins), (1, without)]
                    .iter()
                    .filter(|(_, n)| *n > 0)
                    .map(|(c, n)| StatBlock { label: CLUB_FILTERS[*c].into(), count: n.to_string().into(), group: 2, choice: *c as i32, active: self.club_filter == Some(*c) })
                    .collect(),
            ));
        } else {
            let licensed = base.iter().filter(|r| r.licence.is_some()).count();
            page.set_main_set(set("Tous les coureurs".into(), base.len()));
            page.set_sets_title("Origine".into());
            page.set_sets(model(vec![set("Archive FFCO".into(), licensed), set("Saisis à la main".into(), base.len() - licensed)]));
            page.set_active_set(self.origin.map_or(-1, |o| o as i32));
            let duplicates = base.iter().filter(|r| is_duplicate(r, base)).count();
            page.set_stats(model(
                [(1usize, licensed), (2, base.len() - licensed), (DUPLICATES, duplicates)]
                    .iter()
                    .filter(|(_, n)| *n > 0)
                    .map(|(f, n)| StatBlock { label: FILTERS[*f].into(), count: n.to_string().into(), group: 0, choice: *f as i32 - 1, active: self.filter == *f })
                    .collect(),
            ));
        }
    }

    // The clubs: league, FFCO number, runners; a club that looks like another is to merge.
    fn show_clubs(&self, page: &RunnerBase, base: &[BaseRunner], clubs: &[&str]) {
        let rows: Vec<ClubLine> = clubs
            .iter()
            .filter_map(|club| {
                let members: Vec<&BaseRunner> = base.iter().filter(|r| r.club == *club).collect();
                let code = members.iter().map(|r| r.club_code).find(|c| !c.is_empty()).unwrap_or("");
                let league_code = code.get(code.len().saturating_sub(2)..).unwrap_or("");
                let twin = twin_club(club, clubs);
                let licensed = members.iter().any(|r| r.licence.is_some());
                let keep = match self.club_filter {
                    Some(0) => twin.is_some(),
                    Some(_) => !licensed,
                    None => true,
                } && self.league.as_deref().is_none_or(|l| l == league_code);
                keep.then(|| ClubLine {
                    name: (*club).into(),
                    league: if league_code.is_empty() { "—".into() } else { league_code.into() },
                    number: code.get(..code.len().saturating_sub(2)).filter(|n| !n.is_empty() && *n != "0000").unwrap_or("—").into(),
                    runners: members.len().to_string().into(),
                    twin: twin.map_or(SharedString::new(), |t| format!("Même club que {t} ?").into()),
                })
            })
            .collect();
        page.set_club_rows(model(rows));
    }

    fn show_detail(&self, page: &RunnerBase, data: Option<&Generated>) {
        let found = self.selected.and_then(|id| data.and_then(|d| d.runner_base.iter().find(|r| r.id == id).map(|r| (r, d))));
        page.set_detail_open(found.is_some());
        let Some((runner, data)) = found else { return };

        page.set_detail_name(runner.name().into());
        page.set_detail_subtitle(format!("{} · {}", runner.club, runner.category).into());
        page.set_detail_kind(kind_word(runner.licence_kind).into());
        page.set_detail_club(runner.club.into());
        page.set_detail_code(runner.club_code.into());
        page.set_detail_league(league_name(league(runner)).into());
        page.set_detail_has_licence(runner.licence.is_some());
        page.set_detail_licence(runner.licence.map_or(SharedString::new(), |l| l.to_string().into()));
        page.set_detail_licence_kind(if runner.licence.is_some() { kind_full(runner.licence_kind) } else { NO_LICENCE }.into());
        page.set_detail_season(if runner.renewed { "2026 · relicencié".into() } else { format!("non relicencié (absent de l'archive du {})", data.archive_date).into() });
        page.set_detail_not_renewed(!runner.renewed);
        page.set_detail_points(runner.points.as_ref().map_or("—".into(), |(points, date)| format!("{points} pts · CN du {date}").into()));
        page.set_detail_sex(runner.sex.into());
        page.set_detail_year(runner.year.to_string().into());
        page.set_detail_category(runner.category.as_str().into());
        page.set_detail_card(runner.card.map_or("—".into(), |c| c.to_string().into()));
        let others: Vec<String> = data.runner_base.iter()
            .filter(|o| o.id != runner.id && o.card.is_some() && o.card == runner.card)
            .map(BaseRunner::name)
            .collect();
        page.set_detail_shared(if others.is_empty() { SharedString::new() } else { format!("Aussi sur la fiche de {}", others.join(", ")).into() });
        let name = runner.name();
        let loan = data.card_parks.iter().flat_map(|p| p.cards.iter()).find_map(|card| match &card.place {
            CardPlace::NotReturned { who, since, .. } if *who == name => Some(format!("Puce {} prêtée le {since} · non rendue", card.number)),
            _ => None,
        });
        page.set_detail_loan(loan.unwrap_or_default().into());
        page.set_detail_origin(
            if runner.licence.is_some() {
                format!("Origine : archive FFCO du {}", data.archive_date).into()
            } else {
                format!("Saisie à la main le {}", runner.entered_on).into()
            },
        );
        page.set_confirming(self.confirming);
    }
}
