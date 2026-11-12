// The Coureurs et clubs page: the club's local base of runners. Only the sorting, the search,
// the filters and the deletion of a record work, on the test data.

use crate::cards::plural;
use crate::test_data::{BaseRunner, CardPlace, Generated, LicenceKind};
use crate::{ActiveFilter, App, FilterChoice, FilterGroup, BaseIssue, BaseRunnerRow, LiveFilter, RunnerBase, model};
use slint::{ComponentHandle, Model, ModelRc, SharedString, VecModel};
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;

const FILTERS: [&str; 7] = ["Tous", "Licenciés", "Non licenciés", "Compétition", "Découverte", "Loisir", "Handisport"];

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
                .collect(),
        ));
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
        page.set_title("Base du club".into());
        page.set_state(plural(base.len(), "fiche").into());
        page.set_summary(
            format!("{warning} · {} · {} saisie{} à la main", plural(clubs, "club"), plural(hand, "fiche"), if hand > 1 { "s" } else { "" })
                .into(),
        );
        page.set_summary_short(format!("{warning} · {}", plural(clubs, "club")).into());

        let words: Vec<&str> = self.search.split_whitespace().collect();
        let mut shown: Vec<&BaseRunner> = base
            .iter()
            .filter(|r| matches_filter(r, self.filter))
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
